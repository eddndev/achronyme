use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::native::NativeAsyncOutput;

use super::*;

const COMMAND_CAPACITY: usize = 256;
const MAX_TIMERS: usize = 65_535;
const CANCEL_POLL: Duration = Duration::from_millis(25);

enum Command {
    Submit {
        deadline: Instant,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    },
    Wake,
    Shutdown,
}

struct TimerEntry {
    deadline: Instant,
    sequence: u64,
    cancelled: Arc<AtomicBool>,
    result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline && self.sequence == other.sequence
    }
}

impl Eq for TimerEntry {}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        other
            .deadline
            .cmp(&self.deadline)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

pub(crate) struct TimerReactor {
    sender: Option<mpsc::SyncSender<Command>>,
    thread: Option<JoinHandle<()>>,
}

impl TimerReactor {
    pub(crate) fn new() -> Self {
        let (sender, commands) = mpsc::sync_channel(COMMAND_CAPACITY);
        let thread = thread::Builder::new()
            .name("achronyme-timers".into())
            .spawn(move || timer_loop(commands))
            .expect("timer reactor thread creation failed");
        Self {
            sender: Some(sender),
            thread: Some(thread),
        }
    }

    pub(crate) fn submit(
        &self,
        duration: Duration,
        cancelled: Arc<AtomicBool>,
    ) -> Result<JobReceiver, RuntimeError> {
        let deadline = Instant::now().checked_add(duration).ok_or_else(|| {
            RuntimeError::resource_limit_exceeded("timer duration exceeds platform range")
        })?;
        let (result, receiver) = mpsc::channel();
        let command = Command::Submit {
            deadline,
            cancelled,
            result,
        };
        match self
            .sender
            .as_ref()
            .expect("timer reactor sender exists")
            .try_send(command)
        {
            Ok(()) => Ok(receiver),
            Err(mpsc::TrySendError::Full(_)) => Err(RuntimeError::resource_limit_exceeded(
                format!("timer command queue is full (capacity {COMMAND_CAPACITY})"),
            )),
            Err(mpsc::TrySendError::Disconnected(_)) => {
                Err(RuntimeError::task_failed("timer reactor stopped"))
            }
        }
    }

    pub(crate) fn wake(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.try_send(Command::Wake);
        }
    }
}

impl Drop for TimerReactor {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(Command::Shutdown);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn timer_loop(commands: mpsc::Receiver<Command>) {
    let mut timers = BinaryHeap::<TimerEntry>::new();
    let mut sequence = 0u64;
    loop {
        complete_due(&mut timers);
        let wait = timers
            .peek()
            .map(|timer| timer.deadline.saturating_duration_since(Instant::now()))
            .unwrap_or(CANCEL_POLL)
            .min(CANCEL_POLL);
        match commands.recv_timeout(wait) {
            Ok(Command::Submit {
                deadline,
                cancelled,
                result,
            }) => {
                if timers.len() >= MAX_TIMERS {
                    let _ = result.send(Err(format!("pending timers exceed {MAX_TIMERS}")));
                } else if cancelled.load(Ordering::Acquire) {
                    let _ = result.send(Err("timer cancelled before start".into()));
                } else {
                    timers.push(TimerEntry {
                        deadline,
                        sequence,
                        cancelled,
                        result,
                    });
                    sequence = sequence.wrapping_add(1);
                }
            }
            Ok(Command::Wake) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                fail_all(&mut timers, "timer reactor stopped");
                return;
            }
        }
    }
}

fn complete_due(timers: &mut BinaryHeap<TimerEntry>) {
    let now = Instant::now();
    let mut retained = Vec::new();
    while let Some(timer) = timers.pop() {
        if timer.cancelled.load(Ordering::Acquire) {
            let _ = timer.result.send(Err("timer cancelled".into()));
        } else if timer.deadline <= now {
            let _ = timer.result.send(Ok(NativeAsyncOutput::Nil));
        } else {
            retained.push(timer);
        }
    }
    timers.extend(retained);
}

fn fail_all(timers: &mut BinaryHeap<TimerEntry>, message: &str) {
    while let Some(timer) = timers.pop() {
        let _ = timer.result.send(Err(message.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn several_timers_share_one_reactor_thread_and_preserve_deadlines() {
        let reactor = TimerReactor::new();
        let slow = reactor
            .submit(Duration::from_millis(20), Arc::new(AtomicBool::new(false)))
            .unwrap();
        let fast = reactor
            .submit(Duration::from_millis(1), Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert!(fast.recv().unwrap().is_ok());
        assert!(slow.recv().unwrap().is_ok());
    }
}
