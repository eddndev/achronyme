use memory::Value;

use crate::error::RuntimeError;

use super::super::state::{TaskState, TaskWaitMode};
use super::super::VM;

impl VM {
    pub(in crate::machine::task) fn wake_waiters(
        &mut self,
        target: u32,
    ) -> Result<(), RuntimeError> {
        let waiters = self
            .task_scheduler
            .tasks
            .iter()
            .filter_map(|(id, task)| match &task.state {
                TaskState::Waiting(_, wait) if wait.targets.contains(&target) => {
                    Some((id, wait.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        for (waiter, wait) in waiters {
            if wait.mode == TaskWaitMode::Race {
                self.complete_race_waiter(waiter, target)?;
                continue;
            }
            let capture_failure = wait.mode == TaskWaitMode::Outcome;
            let target_outcome = match &self
                .task_scheduler
                .tasks
                .get(target)
                .ok_or(RuntimeError::InvalidTaskHandle)?
                .state
            {
                TaskState::Completed(value) => Ok(*value),
                TaskState::Failed(message) => Err(message.clone()),
                TaskState::Cancelled => Err("task was cancelled".into()),
                _ => continue,
            };
            match target_outcome {
                Ok(value) => {
                    if let Err(error) = self.transfer_task_result(target, value, Some(waiter)) {
                        self.fail_waiting_task(waiter, error.to_string())?;
                        continue;
                    }
                    let delivered = if capture_failure {
                        self.make_task_outcome(true, value)?
                    } else {
                        value
                    };
                    self.resume_waiter_with_value(waiter, delivered)?;
                    self.mark_task_consumed(target)?;
                }
                Err(message) if capture_failure => {
                    let delivered = self.make_task_error_outcome(message)?;
                    self.resume_waiter_with_value(waiter, delivered)?;
                    self.mark_task_consumed(target)?;
                }
                Err(message) => self.fail_waiting_task(waiter, message)?,
            }
        }
        Ok(())
    }

    fn complete_race_waiter(&mut self, waiter: u32, winner: u32) -> Result<(), RuntimeError> {
        let state = {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(waiter)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            std::mem::replace(&mut task.state, TaskState::Running)
        };
        let TaskState::Waiting(mut context, wait) = state else {
            return Err(RuntimeError::task_failed(
                "race wake targeted a non-waiting task",
            ));
        };
        if wait.mode != TaskWaitMode::Race {
            return Err(RuntimeError::task_failed(
                "race waiter had the wrong wait mode",
            ));
        }
        let delivered = self.finish_task_race(&wait.targets, winner, Some(waiter))?;
        let destination = context
            .stack
            .get_mut(wait.destination)
            .ok_or_else(|| RuntimeError::task_failed("race destination left the task stack"))?;
        *destination = delivered;
        self.task_scheduler
            .tasks
            .get_mut(waiter)
            .ok_or(RuntimeError::InvalidTaskHandle)?
            .state = TaskState::Runnable(context);
        self.task_scheduler.ready.push_back(waiter);
        Ok(())
    }

    fn resume_waiter_with_value(&mut self, waiter: u32, value: Value) -> Result<(), RuntimeError> {
        let state = {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(waiter)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            std::mem::replace(&mut task.state, TaskState::Running)
        };
        let TaskState::Waiting(mut context, wait) = state else {
            return Err(RuntimeError::task_failed("woken task was not waiting"));
        };
        let destination = context
            .stack
            .get_mut(wait.destination)
            .ok_or_else(|| RuntimeError::task_failed("await destination left the task stack"))?;
        *destination = value;
        self.task_scheduler
            .tasks
            .get_mut(waiter)
            .ok_or(RuntimeError::InvalidTaskHandle)?
            .state = TaskState::Runnable(context);
        self.task_scheduler.ready.push_back(waiter);
        Ok(())
    }

    fn mark_task_consumed(&mut self, target: u32) -> Result<(), RuntimeError> {
        {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(target)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            task.state = TaskState::Consumed;
            task.callee = Value::nil();
            task.args.clear();
        }
        self.reap_consumed_task(target);
        Ok(())
    }

    fn fail_waiting_task(&mut self, task: u32, message: String) -> Result<(), RuntimeError> {
        let state = {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(task)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            std::mem::replace(&mut task.state, TaskState::Running)
        };
        if !matches!(state, TaskState::Waiting(_, _)) {
            self.task_scheduler
                .tasks
                .get_mut(task)
                .ok_or(RuntimeError::InvalidTaskHandle)?
                .state = state;
            return Err(RuntimeError::task_failed(
                "failed wakeup targeted a non-waiting task",
            ));
        }
        self.abort_task_scopes_for_owner_from(Some(task), 0);
        self.finish_task_resources(task, true);
        self.task_scheduler
            .tasks
            .get_mut(task)
            .ok_or(RuntimeError::InvalidTaskHandle)?
            .state = TaskState::Failed(message.clone());
        self.record_task_failure(task, message);
        self.cancel_task_siblings(task);
        self.wake_waiters(task)
    }
}
