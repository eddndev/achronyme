use memory::Value;

use crate::error::RuntimeError;

use super::state::TaskState;
use super::VM;

impl VM {
    pub(super) fn make_race_outcome(
        &mut self,
        index: usize,
        ok: bool,
        payload: Value,
    ) -> Result<Value, RuntimeError> {
        let mut outcome = std::collections::HashMap::with_capacity(3);
        outcome.insert("index".into(), Value::int(index as i64));
        outcome.insert("ok".into(), Value::bool(ok));
        outcome.insert(if ok { "value" } else { "error" }.into(), payload);
        Ok(Value::map(self.heap.alloc_map(outcome)?))
    }

    pub(super) fn make_task_outcome(
        &mut self,
        ok: bool,
        payload: Value,
    ) -> Result<Value, RuntimeError> {
        let mut outcome = std::collections::HashMap::with_capacity(2);
        outcome.insert("ok".into(), Value::bool(ok));
        outcome.insert(if ok { "value" } else { "error" }.into(), payload);
        Ok(Value::map(self.heap.alloc_map(outcome)?))
    }

    pub(super) fn make_task_error_outcome(
        &mut self,
        message: String,
    ) -> Result<Value, RuntimeError> {
        let error = Value::string(self.heap.alloc_string(message)?);
        self.make_task_outcome(false, error)
    }

    pub(super) fn consume_failed_task_as_outcome(
        &mut self,
        id: u32,
        message: String,
    ) -> Result<Value, RuntimeError> {
        let record = self
            .task_scheduler
            .tasks
            .get_mut(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        if !matches!(record.state, TaskState::Failed(_) | TaskState::Cancelled) {
            return Err(RuntimeError::task_failed(
                "recoverable await consumed a non-failed task",
            ));
        }
        record.state = TaskState::Consumed;
        record.callee = Value::nil();
        record.args.clear();
        let outcome = self.make_task_error_outcome(message)?;
        self.reap_consumed_task(id);
        Ok(outcome)
    }

    pub(super) fn forget_task_handle(&mut self, task: Value) -> Result<(), RuntimeError> {
        let id = task
            .as_task_handle()
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        let owner = self.task_scheduler.current_task();
        let record = self
            .task_scheduler
            .tasks
            .get(id)
            .ok_or(RuntimeError::TaskOutOfScope)?;
        let owned_here = self
            .task_scheduler
            .scopes
            .iter()
            .any(|scope| scope.id == record.owner_scope && scope.owner_task == owner);
        if !owned_here {
            return Err(RuntimeError::TaskOutOfScope);
        }
        let completed = matches!(record.state, TaskState::Completed(_));
        self.task_scheduler
            .tasks
            .get_mut(id)
            .expect("task ownership checked above")
            .observed = false;
        if completed {
            self.reap_unobserved_success(id);
        }
        Ok(())
    }

    pub(super) fn reap_unobserved_success(&mut self, id: u32) {
        let Some(task) = self.task_scheduler.tasks.get(id) else {
            return;
        };
        if task.observed || !matches!(task.state, TaskState::Completed(_)) {
            return;
        }
        let owner_scope = task.owner_scope;
        self.close_resources_owned_by(Some(id));
        if let Some(scope) = self
            .task_scheduler
            .scopes
            .iter_mut()
            .find(|scope| scope.id == owner_scope)
        {
            scope.children.retain(|child| *child != id);
        }
        let _ = self.task_scheduler.tasks.release(id);
    }

    pub(super) fn reap_consumed_task(&mut self, id: u32) {
        let Some(task) = self.task_scheduler.tasks.get(id) else {
            return;
        };
        let owner_scope = task.owner_scope;
        self.close_resources_owned_by(Some(id));
        if let Some(scope) = self
            .task_scheduler
            .scopes
            .iter_mut()
            .find(|scope| scope.id == owner_scope)
        {
            scope.children.retain(|child| *child != id);
        }
        let _ = self.task_scheduler.tasks.release_consumed(id);
    }

    pub(super) fn consume_completed_task(
        &mut self,
        id: u32,
        owner: Option<u32>,
    ) -> Result<Value, RuntimeError> {
        let value = match self
            .task_scheduler
            .tasks
            .get(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?
            .state
        {
            TaskState::Completed(value) => value,
            _ => {
                return Err(RuntimeError::task_failed(
                    "task did not reach a terminal state",
                ));
            }
        };
        self.transfer_task_result(id, value, owner)?;
        let record = self
            .task_scheduler
            .tasks
            .get_mut(id)
            .ok_or(RuntimeError::InvalidTaskHandle)?;
        if !matches!(record.state, TaskState::Completed(_)) {
            return Err(RuntimeError::task_failed(
                "task did not reach a terminal state",
            ));
        }
        record.state = TaskState::Consumed;
        record.callee = Value::nil();
        record.args.clear();
        self.reap_consumed_task(id);
        Ok(value)
    }
}
