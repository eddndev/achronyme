use std::sync::atomic::Ordering;

use crate::error::RuntimeError;

use super::state::TaskState;
use super::VM;

impl VM {
    pub(super) fn cancel_task_siblings(&mut self, failed: u32) {
        let Some(owner_scope) = self
            .task_scheduler
            .tasks
            .get(failed)
            .map(|task| task.owner_scope)
        else {
            return;
        };
        let siblings = self
            .task_scheduler
            .scopes
            .iter()
            .find(|scope| scope.id == owner_scope)
            .map(|scope| scope.children.clone())
            .unwrap_or_default();
        for sibling in siblings {
            if sibling != failed {
                self.cancel_task(sibling);
            }
        }
    }

    pub(super) fn exit_task_scope(&mut self) -> Result<(), RuntimeError> {
        let owner = self.task_scheduler.current_task();
        let scope_position = self
            .task_scheduler
            .scopes
            .iter()
            .rposition(|scope| scope.owner_task == owner)
            .ok_or(RuntimeError::TaskOutOfScope)?;
        let (scope_id, children) = {
            let scope = &self.task_scheduler.scopes[scope_position];
            (scope.id, scope.children.clone())
        };

        let mut failure = None;
        for (position, id) in children.iter().copied().enumerate() {
            let Some(task) = self.task_scheduler.tasks.get(id) else {
                continue;
            };
            let state = &task.state;
            let result = match state {
                TaskState::Ready { .. }
                | TaskState::ReadyOutput(_)
                | TaskState::Runnable(_)
                | TaskState::Pending(_, _)
                | TaskState::Waiting(_, _) => self.drive_task(id),
                TaskState::Failed(message) => Err(RuntimeError::task_failed(message.clone())),
                TaskState::Cancelled => Err(RuntimeError::TaskCancelled),
                TaskState::Running | TaskState::Suspended(_) => {
                    Err(RuntimeError::task_failed("task still running"))
                }
                TaskState::Completed(_) | TaskState::Consumed => Ok(()),
            };
            if let Err(error) = result {
                failure = Some(error);
                for pending in children.iter().skip(position + 1).copied() {
                    self.cancel_task(pending);
                }
                break;
            }
        }

        let scope = self.task_scheduler.scopes.remove(scope_position);
        debug_assert_eq!(scope.id, scope_id);
        for id in scope.children {
            self.close_task(id);
        }

        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub(crate) fn check_task_cancellation(&self) -> Result<(), RuntimeError> {
        let Some(id) = self.task_scheduler.current_task() else {
            return Ok(());
        };
        let task = self
            .task_scheduler
            .tasks
            .get(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        if task.cancel_requested.load(Ordering::Acquire) {
            Err(RuntimeError::TaskCancelled)
        } else {
            Ok(())
        }
    }

    pub(super) fn cancel_task(&mut self, id: u32) {
        let mut terminal = false;
        if let Some(task) = self.task_scheduler.tasks.get_mut(id) {
            task.cancel_requested.store(true, Ordering::Release);
            let pending_blocking =
                matches!(&task.state, TaskState::Pending(_, _)) && task.blocking_job.is_some();
            let cancelled_before_start = pending_blocking
                && task
                    .blocking_job
                    .as_ref()
                    .is_some_and(|job| job.cancel_before_start());
            if cancelled_before_start
                || (!pending_blocking
                    && matches!(
                        &task.state,
                        TaskState::Ready { .. }
                            | TaskState::ReadyOutput(_)
                            | TaskState::Runnable(_)
                            | TaskState::Pending(_, _)
                            | TaskState::Waiting(_, _)
                    ))
            {
                task.state = TaskState::Cancelled;
                task.blocking_job = None;
                terminal = true;
            }
        }
        self.network_reactor.wake();
        self.timer_reactor.wake();
        self.channel_hub.cancel_requested();
        if terminal {
            self.abort_task_scopes_for_owner_from(Some(id), 0);
            self.finish_task_resources(id, true);
            if let Err(error) = self.wake_waiters(id) {
                if let Some(task) = self.task_scheduler.tasks.get_mut(id) {
                    task.state = TaskState::Failed(error.to_string());
                }
                self.record_task_failure(id, error.to_string());
            }
        }
    }

    pub(crate) fn request_compiled_task_cancel(&mut self, id: u32) -> Result<(), RuntimeError> {
        if self.task_scheduler.tasks.get(id).is_none() {
            return Err(RuntimeError::InvalidTaskHandle);
        }
        self.cancel_task(id);
        Ok(())
    }

    pub(crate) fn wake_compiled_task(&mut self, id: u32) -> Result<(), RuntimeError> {
        let state = self
            .task_scheduler
            .tasks
            .get(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        let enqueue = matches!(
            state.state,
            TaskState::Ready { .. } | TaskState::ReadyOutput(_) | TaskState::Runnable(_)
        );
        let pending = matches!(state.state, TaskState::Pending(_, _));
        let terminal = matches!(
            state.state,
            TaskState::Completed(_) | TaskState::Failed(_) | TaskState::Cancelled
        );
        if matches!(state.state, TaskState::Consumed) {
            return Err(RuntimeError::TaskAlreadyAwaited);
        }

        if enqueue && !self.task_scheduler.ready.contains(&id) {
            self.task_scheduler.ready.push_back(id);
        }
        if pending {
            self.network_reactor.wake();
            self.timer_reactor.wake();
            let _ = self.poll_pending_task(id, false)?;
        }
        if terminal {
            self.wake_waiters(id)?;
        }
        Ok(())
    }

    fn close_task(&mut self, id: u32) {
        if let Err(error) = self.wait_for_cancelled_host_work(id) {
            if let Some(task) = self.task_scheduler.tasks.get_mut(id) {
                task.state = TaskState::Failed(error.to_string());
            }
            self.record_task_failure(id, error.to_string());
        }
        self.abort_task_scopes_for_owner_from(Some(id), 0);
        self.finish_task_resources(id, true);
        let _ = self.task_scheduler.tasks.release(id);
    }

    pub(in crate::machine::task) fn wait_for_cancelled_host_work(
        &mut self,
        id: u32,
    ) -> Result<(), RuntimeError> {
        let should_wait = self.task_scheduler.tasks.get(id).is_some_and(|task| {
            task.cancel_requested.load(Ordering::Acquire)
                && task.blocking_job.is_some()
                && matches!(&task.state, TaskState::Pending(_, _))
        });
        if should_wait {
            self.poll_pending_task(id, true)?;
        }
        Ok(())
    }

    pub(super) fn task_scope_count(&self, owner: Option<u32>) -> usize {
        self.task_scheduler
            .scopes
            .iter()
            .filter(|scope| scope.owner_task == owner)
            .count()
    }

    pub(super) fn abort_task_scopes_for_owner_from(&mut self, owner: Option<u32>, depth: usize) {
        while self.task_scope_count(owner) > depth {
            let Some(position) = self
                .task_scheduler
                .scopes
                .iter()
                .rposition(|scope| scope.owner_task == owner)
            else {
                break;
            };
            let scope = self.task_scheduler.scopes.remove(position);
            for id in scope.children {
                self.cancel_task(id);
                self.close_task(id);
            }
        }
    }

    pub(crate) fn abort_all_task_scopes(&mut self) {
        while let Some(scope) = self.task_scheduler.scopes.pop() {
            for id in scope.children {
                self.cancel_task(id);
                self.close_task(id);
            }
        }
    }

    /// Number of live lexical task scopes, exposed for invariant tests.
    pub fn active_task_scope_count(&self) -> usize {
        self.task_scheduler.scopes.len()
    }

    /// Number of live task records, exposed for resource-bound tests and diagnostics.
    pub fn live_task_count(&self) -> usize {
        self.task_scheduler.tasks.len()
    }
}

#[cfg(test)]
#[path = "cancellation/tests.rs"]
mod tests;
