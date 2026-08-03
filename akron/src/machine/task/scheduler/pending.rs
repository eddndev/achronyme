use std::sync::mpsc::{RecvTimeoutError, TryRecvError};
use std::time::Duration;

use memory::Value;

use crate::error::RuntimeError;

use super::super::state::TaskState;
use super::super::VM;

enum PendingPoll {
    Ready,
    Blocking,
    Timeout(Duration),
}

impl VM {
    pub(in crate::machine::task) fn poll_pending_task(
        &mut self,
        id: u32,
        blocking: bool,
    ) -> Result<bool, RuntimeError> {
        let poll = if blocking {
            PendingPoll::Blocking
        } else {
            PendingPoll::Ready
        };
        self.poll_pending_task_with_timeout(id, poll)
    }

    pub(in crate::machine::task) fn poll_pending_task_for(
        &mut self,
        id: u32,
        timeout: Duration,
    ) -> Result<bool, RuntimeError> {
        self.poll_pending_task_with_timeout(id, PendingPoll::Timeout(timeout))
    }

    fn poll_pending_task_with_timeout(
        &mut self,
        id: u32,
        poll: PendingPoll,
    ) -> Result<bool, RuntimeError> {
        let receiver = {
            let Some(task) = self.task_scheduler.tasks.get_mut(id) else {
                return Err(RuntimeError::InvalidTaskHandle);
            };
            if !matches!(task.state, TaskState::Pending(_, _)) {
                return Ok(false);
            }
            match std::mem::replace(&mut task.state, TaskState::Running) {
                TaskState::Pending(receiver, kind) => (receiver, kind),
                _ => unreachable!("pending state checked before replacement"),
            }
        };

        let (receiver, pending_kind) = receiver;

        let received = match poll {
            PendingPoll::Ready => receiver.try_recv(),
            PendingPoll::Blocking => receiver.recv().map_err(|_| TryRecvError::Disconnected),
            PendingPoll::Timeout(timeout) => match receiver.recv_timeout(timeout) {
                Ok(outcome) => Ok(outcome),
                Err(RecvTimeoutError::Timeout) => Err(TryRecvError::Empty),
                Err(RecvTimeoutError::Disconnected) => Err(TryRecvError::Disconnected),
            },
        };
        let cancelled = self.task_scheduler.tasks.get(id).is_some_and(|task| {
            task.cancel_requested
                .load(std::sync::atomic::Ordering::Acquire)
        });
        match received {
            Ok(outcome) if cancelled => {
                drop(outcome);
                self.store_task_cancellation(id)?;
                Ok(true)
            }
            Ok(outcome) => {
                let outcome = outcome
                    .map_err(RuntimeError::task_failed)
                    .and_then(|output| self.materialize_async_output(output));
                self.store_task_outcome(id, outcome)?;
                Ok(true)
            }
            Err(TryRecvError::Empty) => {
                self.task_scheduler
                    .tasks
                    .get_mut(id)
                    .ok_or(RuntimeError::InvalidTaskHandle)?
                    .state = TaskState::Pending(receiver, pending_kind);
                Ok(false)
            }
            Err(TryRecvError::Disconnected) => {
                if cancelled {
                    self.store_task_cancellation(id)?;
                } else {
                    self.store_task_outcome(
                        id,
                        Err(RuntimeError::task_failed(
                            "async host executor disconnected",
                        )),
                    )?;
                }
                Ok(true)
            }
        }
    }

    pub(super) fn terminal_task_result(&self, id: u32) -> Option<Result<(), RuntimeError>> {
        let Some(task) = self.task_scheduler.tasks.get(id) else {
            return Some(Ok(()));
        };
        match &task.state {
            TaskState::Completed(_) | TaskState::Consumed => Some(Ok(())),
            TaskState::Failed(message) => Some(Err(RuntimeError::task_failed(message.clone()))),
            TaskState::Cancelled => Some(Err(RuntimeError::TaskCancelled)),
            TaskState::Ready { .. }
            | TaskState::ReadyOutput(_)
            | TaskState::Runnable(_)
            | TaskState::Pending(_, _)
            | TaskState::Running
            | TaskState::Suspended(_)
            | TaskState::Waiting(_, _) => None,
        }
    }

    pub(in crate::machine::task) fn store_task_outcome(
        &mut self,
        id: u32,
        outcome: Result<Value, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let retain_success = outcome.is_ok()
            && self
                .task_scheduler
                .tasks
                .get(id)
                .is_some_and(|task| task.observed);
        let outcome = if retain_success
            && self.task_scheduler.retained_task_result_count()
                >= self.runtime_limits.max_retained_task_results
        {
            Err(RuntimeError::resource_limit_exceeded(format!(
                "retained task results exceed {}",
                self.runtime_limits.max_retained_task_results
            )))
        } else {
            outcome
        };
        let failed = outcome.is_err();
        let failure_message = outcome.as_ref().err().map(ToString::to_string);
        self.finish_task_resources(id, failed);
        let task = self
            .task_scheduler
            .tasks
            .get_mut(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        task.blocking_job = None;
        let propagate_failure = failed && !task.capture_failure;
        task.state = match outcome {
            Ok(value) => TaskState::Completed(value),
            Err(error) => TaskState::Failed(error.to_string()),
        };
        if propagate_failure {
            if let Some(message) = failure_message {
                self.record_task_failure(id, message);
            }
            self.cancel_task_siblings(id);
        }
        self.wake_waiters(id)?;
        if !failed {
            self.reap_unobserved_success(id);
        }
        Ok(())
    }

    fn store_task_cancellation(&mut self, id: u32) -> Result<(), RuntimeError> {
        self.finish_task_resources(id, true);
        let task = self
            .task_scheduler
            .tasks
            .get_mut(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        task.blocking_job = None;
        task.state = TaskState::Cancelled;
        self.wake_waiters(id)
    }
}
