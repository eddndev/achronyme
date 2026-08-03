use memory::Value;

use crate::error::RuntimeError;

use super::state::{TaskState, TaskWait, TaskWaitMode};
use super::VM;

enum AwaitState {
    Pending,
    Completed,
    Failed(String),
    Cancelled,
    Consumed,
    Running,
}

impl VM {
    pub(super) fn await_task(
        &mut self,
        task: Value,
        destination: usize,
        capture_failure: bool,
    ) -> Result<Value, RuntimeError> {
        let id = task
            .as_task_handle()
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        let owner = self.task_scheduler.current_task();
        let Some(record) = self.task_scheduler.tasks.get(id) else {
            return if self.task_scheduler.tasks.was_consumed(id) {
                Err(RuntimeError::TaskAlreadyAwaited)
            } else {
                Err(RuntimeError::TaskOutOfScope)
            };
        };
        let owner_scope = record.owner_scope;
        let owned_here = self
            .task_scheduler
            .scopes
            .iter()
            .any(|scope| scope.id == owner_scope && scope.owner_task == owner);
        if !owned_here {
            return Err(RuntimeError::TaskOutOfScope);
        }
        if capture_failure {
            self.task_scheduler
                .tasks
                .get_mut(id)
                .expect("task ownership checked above")
                .capture_failure = true;
        }
        let state = match &self
            .task_scheduler
            .tasks
            .get(id)
            .expect("task ownership checked above")
            .state
        {
            TaskState::Consumed => AwaitState::Consumed,
            TaskState::Cancelled => AwaitState::Cancelled,
            TaskState::Running | TaskState::Suspended(_) => AwaitState::Running,
            TaskState::Failed(message) => AwaitState::Failed(message.clone()),
            TaskState::Completed(_) => AwaitState::Completed,
            TaskState::Ready { .. }
            | TaskState::ReadyOutput(_)
            | TaskState::Runnable(_)
            | TaskState::Pending(_, _)
            | TaskState::Waiting(_, _) => AwaitState::Pending,
        };
        match state {
            AwaitState::Consumed => return Err(RuntimeError::TaskAlreadyAwaited),
            AwaitState::Cancelled if capture_failure => {
                return self.consume_failed_task_as_outcome(id, "task was cancelled".into());
            }
            AwaitState::Cancelled => return Err(RuntimeError::TaskCancelled),
            AwaitState::Running => {
                return Err(RuntimeError::task_failed("cyclic task await"));
            }
            AwaitState::Failed(message) if capture_failure => {
                return self.consume_failed_task_as_outcome(id, message);
            }
            AwaitState::Failed(message) => {
                return Err(RuntimeError::task_failed(message));
            }
            AwaitState::Completed => {
                let value = self.consume_completed_task(id, owner)?;
                return if capture_failure {
                    self.make_task_outcome(true, value)
                } else {
                    Ok(value)
                };
            }
            AwaitState::Pending => {}
        }

        if owner.is_some() {
            if self.native_call_depth != 0 {
                return Err(RuntimeError::task_failed(
                    "await cannot suspend through an immediate native callback",
                ));
            }
            if self.task_scheduler.pending_wait.is_some() {
                return Err(RuntimeError::task_failed(
                    "task attempted to register more than one await",
                ));
            }
            self.task_scheduler.pending_wait = Some(TaskWait {
                targets: vec![id],
                destination,
                mode: if capture_failure {
                    TaskWaitMode::Outcome
                } else {
                    TaskWaitMode::Propagate
                },
            });
            return Err(RuntimeError::TaskSuspended);
        }
        let drive_result = self.drive_task(id);
        if capture_failure {
            match &self
                .task_scheduler
                .tasks
                .get(id)
                .ok_or(RuntimeError::TaskOutOfScope)?
                .state
            {
                TaskState::Completed(_) => {
                    let value = self.consume_completed_task(id, owner)?;
                    self.make_task_outcome(true, value)
                }
                TaskState::Failed(message) => {
                    let message = message.clone();
                    self.consume_failed_task_as_outcome(id, message)
                }
                TaskState::Cancelled => {
                    self.consume_failed_task_as_outcome(id, "task was cancelled".into())
                }
                _ => {
                    drive_result?;
                    Err(RuntimeError::task_failed(
                        "recoverable await did not reach a terminal task state",
                    ))
                }
            }
        } else {
            drive_result?;
            self.consume_completed_task(id, owner)
        }
    }

    pub(super) fn await_task_race(
        &mut self,
        tasks: &[Value],
        destination: usize,
    ) -> Result<Value, RuntimeError> {
        if tasks.len() < 2 {
            return Err(RuntimeError::task_failed(
                "task race requires at least two handles",
            ));
        }
        let owner = self.task_scheduler.current_task();
        let mut targets = Vec::with_capacity(tasks.len());
        for task in tasks {
            let id = task
                .as_task_handle()
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            if targets.contains(&id) {
                return Err(RuntimeError::task_failed(
                    "task race cannot contain the same handle twice",
                ));
            }
            let Some(record) = self.task_scheduler.tasks.get(id) else {
                return if self.task_scheduler.tasks.was_consumed(id) {
                    Err(RuntimeError::TaskAlreadyAwaited)
                } else {
                    Err(RuntimeError::TaskOutOfScope)
                };
            };
            let owned_here = self
                .task_scheduler
                .scopes
                .iter()
                .any(|scope| scope.id == record.owner_scope && scope.owner_task == owner);
            if !owned_here {
                return Err(RuntimeError::TaskOutOfScope);
            }
            if matches!(record.state, TaskState::Running | TaskState::Suspended(_)) {
                return Err(RuntimeError::task_failed("cyclic task race"));
            }
            targets.push(id);
        }
        for target in &targets {
            self.task_scheduler
                .tasks
                .get_mut(*target)
                .expect("race target validated above")
                .capture_failure = true;
        }

        if let Some(winner) = self.race_winner(&targets) {
            return self.finish_task_race(&targets, winner, owner);
        }
        if owner.is_some() {
            if self.native_call_depth != 0 {
                return Err(RuntimeError::task_failed(
                    "task race cannot suspend through an immediate native callback",
                ));
            }
            if self.task_scheduler.pending_wait.is_some() {
                return Err(RuntimeError::task_failed(
                    "task attempted to register more than one await",
                ));
            }
            self.task_scheduler.pending_wait = Some(TaskWait {
                targets,
                destination,
                mode: TaskWaitMode::Race,
            });
            return Err(RuntimeError::TaskSuspended);
        }
        let winner = self.drive_task_race(&targets)?;
        self.finish_task_race(&targets, winner, owner)
    }

    pub(super) fn race_winner(&self, targets: &[u32]) -> Option<u32> {
        targets.iter().copied().find(|target| {
            self.task_scheduler.tasks.get(*target).is_some_and(|task| {
                matches!(
                    task.state,
                    TaskState::Completed(_) | TaskState::Failed(_) | TaskState::Cancelled
                )
            })
        })
    }

    pub(super) fn finish_task_race(
        &mut self,
        targets: &[u32],
        winner: u32,
        owner: Option<u32>,
    ) -> Result<Value, RuntimeError> {
        let index = targets
            .iter()
            .position(|target| *target == winner)
            .ok_or_else(|| RuntimeError::task_failed("race winner was not a target"))?;
        let outcome = match &self
            .task_scheduler
            .tasks
            .get(winner)
            .ok_or(RuntimeError::InvalidTaskHandle)?
            .state
        {
            TaskState::Completed(value) => Ok(*value),
            TaskState::Failed(message) => Err(message.clone()),
            TaskState::Cancelled => Err("task was cancelled".into()),
            _ => {
                return Err(RuntimeError::task_failed(
                    "race winner was not in a terminal state",
                ));
            }
        };

        let delivered = match outcome {
            Ok(value) => {
                self.transfer_task_result(winner, value, owner)?;
                self.make_race_outcome(index, true, value)?
            }
            Err(message) => {
                let error = Value::string(self.heap.alloc_string(message)?);
                self.make_race_outcome(index, false, error)?
            }
        };
        for target in targets {
            if *target != winner {
                self.cancel_task(*target);
            }
        }
        for target in targets {
            if *target != winner {
                self.wait_for_cancelled_host_work(*target)?;
            }
        }
        for target in targets {
            self.reap_consumed_task(*target);
        }
        Ok(delivered)
    }
}
