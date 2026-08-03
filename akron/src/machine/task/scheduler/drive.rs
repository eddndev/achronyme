use std::time::Duration;

use crate::error::RuntimeError;

use super::super::state::{PendingKind, TaskState};
use super::super::VM;

const COOPERATIVE_WAIT_SLICE: Duration = Duration::from_millis(1);

impl VM {
    pub(in crate::machine::task) fn drive_task(&mut self, target: u32) -> Result<(), RuntimeError> {
        loop {
            if let Some(result) = self.terminal_task_result(target) {
                return result;
            }

            let tasks = self.task_scheduler.tasks.handles().collect::<Vec<_>>();
            for id in tasks {
                if self.poll_pending_task(id, false)? {
                    if let Some(Err(error)) = self.terminal_task_result(id) {
                        return Err(error);
                    }
                }
            }
            if let Some(result) = self.terminal_task_result(target) {
                return result;
            }

            if let Some(id) = self.next_ready_task() {
                self.execute_ready_task(id)?;
                if let Some(Err(error)) = self.terminal_task_result(id) {
                    return Err(error);
                }
                continue;
            }

            let pending_tasks = self
                .task_scheduler
                .tasks
                .iter()
                .filter_map(|(id, task)| {
                    matches!(task.state, TaskState::Pending(_, PendingKind::External)).then_some(id)
                })
                .collect::<Vec<_>>();
            let pending = pending_tasks
                .contains(&target)
                .then_some(target)
                .or_else(|| pending_tasks.first().copied());
            let Some(pending) = pending else {
                let state = self
                    .task_scheduler
                    .tasks
                    .get(target)
                    .map(|task| format!("{:?}", task.state))
                    .unwrap_or_else(|| "<reaped>".into());
                return Err(RuntimeError::task_failed(format!(
                    "scheduler stalled while waiting for task {target}: {state}"
                )));
            };
            if pending_tasks.len() == 1 {
                self.poll_pending_task(pending, true)?;
            } else {
                self.poll_pending_task_for(pending, COOPERATIVE_WAIT_SLICE)?;
            }
        }
    }

    pub(in crate::machine::task) fn drive_task_race(
        &mut self,
        targets: &[u32],
    ) -> Result<u32, RuntimeError> {
        loop {
            if let Some(winner) = self.race_winner(targets) {
                return Ok(winner);
            }

            let tasks = self.task_scheduler.tasks.handles().collect::<Vec<_>>();
            for id in tasks {
                if self.poll_pending_task(id, false)? {
                    if let Some(winner) = self.race_winner(targets) {
                        return Ok(winner);
                    }
                    if !targets.contains(&id) {
                        if let Some(Err(error)) = self.terminal_task_result(id) {
                            return Err(error);
                        }
                    }
                }
            }

            if let Some(id) = self.next_ready_task() {
                self.execute_ready_task(id)?;
                if let Some(winner) = self.race_winner(targets) {
                    return Ok(winner);
                }
                if !targets.contains(&id) {
                    if let Some(Err(error)) = self.terminal_task_result(id) {
                        return Err(error);
                    }
                }
                continue;
            }

            let pending = self.task_scheduler.tasks.iter().find_map(|(id, task)| {
                matches!(task.state, TaskState::Pending(_, PendingKind::External)).then_some(id)
            });
            let Some(pending) = pending else {
                return Err(RuntimeError::task_failed(
                    "scheduler stalled while waiting for task race",
                ));
            };
            self.poll_pending_task_for(pending, COOPERATIVE_WAIT_SLICE)?;
        }
    }

    fn next_ready_task(&mut self) -> Option<u32> {
        while let Some(id) = self.task_scheduler.ready.pop_front() {
            if self.task_scheduler.tasks.get(id).is_some_and(|task| {
                matches!(
                    task.state,
                    TaskState::Ready { .. } | TaskState::ReadyOutput(_) | TaskState::Runnable(_)
                )
            }) {
                return Some(id);
            }
        }
        None
    }

    fn execute_ready_task(&mut self, id: u32) -> Result<(), RuntimeError> {
        let (callee, args, initial_budget, resume, ready_output) = {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(id)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            let state = std::mem::replace(&mut task.state, TaskState::Running);
            match state {
                TaskState::Ready { instruction_budget } => (
                    task.callee,
                    task.args.clone(),
                    Some(instruction_budget),
                    None,
                    None,
                ),
                TaskState::ReadyOutput(receiver) => {
                    (task.callee, task.args.clone(), None, None, Some(receiver))
                }
                TaskState::Runnable(context) => {
                    (task.callee, task.args.clone(), None, Some(context), None)
                }
                state => {
                    task.state = state;
                    return Ok(());
                }
            }
        };

        if let Some(receiver) = ready_output {
            let outcome = receiver
                .recv()
                .map_err(|_| RuntimeError::task_failed("yield completion disconnected"))?
                .map_err(RuntimeError::task_failed)
                .and_then(|output| self.materialize_async_output(output));
            return self.store_task_outcome(id, outcome);
        }

        let parent = self.task_scheduler.current_task();
        self.suspend_active_context(parent)?;
        self.task_scheduler.current_tasks.push(id);
        let outcome = if let Some(context) = resume {
            self.restore_active_context(context)
                .and_then(|()| self.run_until_frame_depth(0))
                .map(|()| self.last_result)
        } else if let Some(instruction_budget) = initial_budget {
            self.instruction_budget = instruction_budget;
            self.call_value(callee, &args)
        } else {
            Err(RuntimeError::task_failed(
                "new task is missing its instruction budget",
            ))
        };
        let location = if outcome.is_err() {
            self.last_error_location
                .clone()
                .or_else(|| self.current_source_location())
        } else {
            self.current_source_location()
        };
        self.update_task_location(id, location);

        if matches!(&outcome, Err(RuntimeError::TaskSuspended)) {
            let context = self.capture_active_context()?;
            let wait = self.task_scheduler.pending_wait.take().ok_or_else(|| {
                RuntimeError::task_failed("task suspended without an await target")
            })?;
            self.task_scheduler.current_tasks.pop();
            self.task_scheduler
                .tasks
                .get_mut(id)
                .ok_or(RuntimeError::InvalidTaskHandle)?
                .state = TaskState::Waiting(context, wait);
            return self.resume_active_context(parent);
        }

        let outcome = if self.task_scope_count(Some(id)) == 0 {
            outcome
        } else {
            self.abort_task_scopes_for_owner_from(Some(id), 0);
            match outcome {
                Err(error) => Err(error),
                Ok(_) => Err(RuntimeError::task_failed(
                    "task returned with an open concurrent scope",
                )),
            }
        };

        let capture_result = self.capture_active_context();
        self.task_scheduler.current_tasks.pop();
        let outcome = match (outcome, capture_result) {
            (Ok(value), Ok(_)) => Ok(value),
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        };
        self.store_task_outcome(id, outcome)?;
        self.resume_active_context(parent)
    }
}
