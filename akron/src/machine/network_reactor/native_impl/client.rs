use std::thread::{self, JoinHandle};

use super::*;

pub(crate) struct NetworkReactor {
    sender: Option<mpsc::SyncSender<Command>>,
    waker: Arc<Waker>,
    thread: Option<JoinHandle<()>>,
}

impl NetworkReactor {
    pub(crate) fn new() -> Self {
        let poll = Poll::new().expect("network reactor poll creation failed");
        let waker = Arc::new(
            Waker::new(poll.registry(), WAKE_TOKEN).expect("network reactor waker creation failed"),
        );
        let (sender, commands) = mpsc::sync_channel(COMMAND_CAPACITY);
        let thread = thread::Builder::new()
            .name("achronyme-reactor".into())
            .spawn(move || {
                ReactorLoop {
                    poll,
                    events: Events::with_capacity(256),
                    commands,
                    listeners: HashMap::new(),
                    connections: HashMap::new(),
                    pending: HashMap::new(),
                }
                .run();
            })
            .expect("network reactor thread creation failed");
        Self {
            sender: Some(sender),
            waker,
            thread: Some(thread),
        }
    }

    pub(crate) fn submit(
        &self,
        request: NativeNetworkRequest,
        cancelled: Arc<AtomicBool>,
    ) -> Result<JobReceiver, RuntimeError> {
        let (result, receiver) = mpsc::channel();
        self.try_send(Command::Submit {
            request,
            cancelled,
            result,
        })?;
        Ok(receiver)
    }

    pub(crate) fn close_silently(&self, handle: u32) {
        let Some(sender) = &self.sender else {
            return;
        };
        if sender.send(Command::Close(handle)).is_ok() {
            self.wake();
        }
    }

    pub(crate) fn wake(&self) {
        let _ = self.waker.wake();
    }

    fn try_send(&self, command: Command) -> Result<(), RuntimeError> {
        match self
            .sender
            .as_ref()
            .expect("reactor sender exists")
            .try_send(command)
        {
            Ok(()) => {
                self.wake();
                Ok(())
            }
            Err(mpsc::TrySendError::Full(_)) => Err(RuntimeError::resource_limit_exceeded(
                format!("network command queue is full (capacity {COMMAND_CAPACITY})"),
            )),
            Err(mpsc::TrySendError::Disconnected(_)) => {
                Err(RuntimeError::task_failed("network reactor stopped"))
            }
        }
    }
}

impl Drop for NetworkReactor {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(Command::Shutdown);
            let _ = self.waker.wake();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
