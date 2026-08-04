//! Bounded executor for legacy blocking host operations.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{mpsc, Arc};

use crate::native::{NativeAsyncOutput, NativeJob};
use crate::RuntimeError;

pub(crate) type JobReceiver = mpsc::Receiver<Result<NativeAsyncOutput, String>>;

pub(crate) const DEFAULT_BLOCKING_WORKERS: usize = 4;
pub(crate) const DEFAULT_BLOCKING_QUEUE_CAPACITY: usize = 64;
pub(crate) const MAX_BLOCKING_WORKERS: usize = 64;
pub(crate) const MAX_BLOCKING_QUEUE_CAPACITY: usize = u16::MAX as usize;

const JOB_QUEUED: u8 = 0;
const JOB_RUNNING: u8 = 1;
const JOB_CANCELLED: u8 = 2;
const JOB_COMPLETED: u8 = 3;

#[derive(Debug)]
pub(crate) struct BlockingJobControl {
    state: AtomicU8,
}

impl BlockingJobControl {
    fn new() -> Self {
        Self {
            state: AtomicU8::new(JOB_QUEUED),
        }
    }

    fn begin(&self) -> bool {
        self.state
            .compare_exchange(JOB_QUEUED, JOB_RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    fn complete(&self) {
        self.state.store(JOB_COMPLETED, Ordering::Release);
    }

    /// Cancel only while the worker has not begun the host operation.
    /// A false result means cleanup must wait for the running or completed
    /// operation and discard its result.
    pub(crate) fn cancel_before_start(&self) -> bool {
        self.state
            .compare_exchange(
                JOB_QUEUED,
                JOB_CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}

pub(crate) struct BlockingSubmission {
    pub(crate) receiver: JobReceiver,
    pub(crate) control: Arc<BlockingJobControl>,
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::sync::Mutex;
    use std::thread::{self, JoinHandle};

    use super::*;

    struct WorkItem {
        job: NativeJob,
        control: Arc<BlockingJobControl>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    }

    pub(crate) struct BlockingPool {
        sender: Option<mpsc::SyncSender<WorkItem>>,
        workers: Vec<JoinHandle<()>>,
        queue_capacity: usize,
    }

    impl BlockingPool {
        pub(crate) fn new() -> Self {
            Self::with_capacity(DEFAULT_BLOCKING_WORKERS, DEFAULT_BLOCKING_QUEUE_CAPACITY)
                .expect("default blocking-pool worker creation failed")
        }

        pub(crate) fn with_capacity(
            worker_count: usize,
            queue_capacity: usize,
        ) -> Result<Self, RuntimeError> {
            if worker_count == 0 || worker_count > MAX_BLOCKING_WORKERS {
                return Err(RuntimeError::resource_limit_exceeded(format!(
                    "blocking workers must be 1..={MAX_BLOCKING_WORKERS}"
                )));
            }
            if queue_capacity == 0 || queue_capacity > MAX_BLOCKING_QUEUE_CAPACITY {
                return Err(RuntimeError::resource_limit_exceeded(format!(
                    "blocking queue capacity must be 1..={MAX_BLOCKING_QUEUE_CAPACITY}"
                )));
            }

            let (sender, receiver) = mpsc::sync_channel::<WorkItem>(queue_capacity);
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(worker_count);
            for index in 0..worker_count {
                let receiver = Arc::clone(&receiver);
                match thread::Builder::new()
                    .name(format!("achronyme-io-{index}"))
                    .spawn(move || worker_loop(&receiver))
                {
                    Ok(worker) => workers.push(worker),
                    Err(error) => {
                        drop(sender);
                        for worker in workers {
                            let _ = worker.join();
                        }
                        return Err(RuntimeError::resource_limit_exceeded(format!(
                            "could not create blocking worker {index}: {error}"
                        )));
                    }
                }
            }
            Ok(Self {
                sender: Some(sender),
                workers,
                queue_capacity,
            })
        }

        pub(crate) fn submit(&self, job: NativeJob) -> Result<BlockingSubmission, RuntimeError> {
            let (result, receiver) = mpsc::channel();
            let control = Arc::new(BlockingJobControl::new());
            let item = WorkItem {
                job,
                control: Arc::clone(&control),
                result,
            };
            match self
                .sender
                .as_ref()
                .expect("pool sender exists")
                .try_send(item)
            {
                Ok(()) => Ok(BlockingSubmission { receiver, control }),
                Err(mpsc::TrySendError::Full(_)) => {
                    Err(RuntimeError::resource_limit_exceeded(format!(
                        "blocking I/O queue is full (capacity {})",
                        self.queue_capacity
                    )))
                }
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    Err(RuntimeError::task_failed("blocking I/O pool stopped"))
                }
            }
        }
    }

    fn worker_loop(receiver: &Mutex<mpsc::Receiver<WorkItem>>) {
        loop {
            let item = match receiver.lock() {
                Ok(receiver) => receiver.recv(),
                Err(_) => return,
            };
            let Ok(item) = item else {
                return;
            };
            let output = if !item.control.begin() {
                Err("task cancelled before host work began".to_string())
            } else {
                let output = (item.job)();
                item.control.complete();
                output
            };
            let _ = item.result.send(output);
        }
    }

    impl Drop for BlockingPool {
        fn drop(&mut self) {
            self.sender.take();
            for worker in self.workers.drain(..) {
                let _ = worker.join();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::sync::atomic::{AtomicBool, AtomicUsize};
        use std::time::{Duration, Instant};

        use super::*;

        #[test]
        fn configured_worker_and_queue_bounds_are_enforced() {
            let pool = BlockingPool::with_capacity(1, 1).unwrap();
            let release = Arc::new(AtomicBool::new(false));
            let active = Arc::new(AtomicUsize::new(0));
            let release_worker = Arc::clone(&release);
            let active_worker = Arc::clone(&active);
            let running = pool
                .submit(Box::new(move || {
                    active_worker.fetch_add(1, Ordering::AcqRel);
                    while !release_worker.load(Ordering::Acquire) {
                        thread::yield_now();
                    }
                    Ok(NativeAsyncOutput::Nil)
                }))
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while active.load(Ordering::Acquire) != 1 {
                assert!(Instant::now() < deadline, "worker did not become active");
                thread::yield_now();
            }
            let queued = pool
                .submit(Box::new(|| Ok(NativeAsyncOutput::Nil)))
                .unwrap();
            let overflow = pool.submit(Box::new(|| Ok(NativeAsyncOutput::Nil)));

            release.store(true, Ordering::Release);
            running.receiver.recv().unwrap().unwrap();
            queued.receiver.recv().unwrap().unwrap();

            let error = match overflow {
                Err(error) => error,
                Ok(_) => panic!("third submission exceeded the configured queue"),
            };
            assert!(error.to_string().contains("capacity 1"), "{error}");
        }

        #[test]
        fn cancellation_while_queued_skips_host_work() {
            let pool = BlockingPool::new();
            let release = Arc::new(AtomicBool::new(false));
            let active = Arc::new(AtomicUsize::new(0));
            let mut blockers = Vec::new();
            for _ in 0..DEFAULT_BLOCKING_WORKERS {
                let release = Arc::clone(&release);
                let active = Arc::clone(&active);
                blockers.push(
                    pool.submit(Box::new(move || {
                        active.fetch_add(1, Ordering::AcqRel);
                        while !release.load(Ordering::Acquire) {
                            thread::sleep(Duration::from_millis(1));
                        }
                        Ok(NativeAsyncOutput::Nil)
                    }))
                    .unwrap(),
                );
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while active.load(Ordering::Acquire) != DEFAULT_BLOCKING_WORKERS {
                assert!(Instant::now() < deadline, "workers did not become active");
                thread::sleep(Duration::from_millis(1));
            }

            let ran = Arc::new(AtomicBool::new(false));
            let ran_in_job = Arc::clone(&ran);
            let queued = pool
                .submit(Box::new(move || {
                    ran_in_job.store(true, Ordering::Release);
                    Ok(NativeAsyncOutput::Nil)
                }))
                .unwrap();
            assert!(queued.control.cancel_before_start());

            release.store(true, Ordering::Release);
            for blocker in blockers {
                blocker
                    .receiver
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap()
                    .unwrap();
            }
            let cancellation = queued
                .receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .unwrap_err();
            assert!(cancellation.contains("cancelled before host work began"));
            assert!(!ran.load(Ordering::Acquire));
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::*;

    pub(crate) struct BlockingPool;

    impl BlockingPool {
        pub(crate) fn new() -> Self {
            Self
        }

        pub(crate) fn with_capacity(
            _worker_count: usize,
            _queue_capacity: usize,
        ) -> Result<Self, RuntimeError> {
            Ok(Self)
        }

        pub(crate) fn submit(&self, job: NativeJob) -> Result<BlockingSubmission, RuntimeError> {
            let (result, receiver) = mpsc::channel();
            let control = Arc::new(BlockingJobControl::new());
            let output = if control.begin() {
                job()
            } else {
                unreachable!()
            };
            control.complete();
            let _ = result.send(output);
            Ok(BlockingSubmission { receiver, control })
        }
    }
}

pub(crate) use imp::BlockingPool;
