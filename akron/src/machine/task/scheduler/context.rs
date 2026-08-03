use memory::Value;

use crate::error::RuntimeError;
use crate::machine::upvalue::UpvalueOps;

use super::super::state::{TaskContext, TaskState};
use super::super::VM;

impl VM {
    pub(super) fn active_stack_top(&self) -> usize {
        self.frames
            .iter()
            .filter_map(|frame| {
                let closure = self.heap.get_closure(frame.closure)?;
                let function = self.heap.get_function(closure.function)?;
                Some(frame.base + function.max_slots as usize)
            })
            .max()
            .unwrap_or(0)
    }

    pub(super) fn capture_active_context(&mut self) -> Result<TaskContext, RuntimeError> {
        self.close_upvalues(0)?;
        let stack_top = self.active_stack_top();
        let stack = self.stack[..stack_top].to_vec();
        self.stack[..stack_top].fill(Value::nil());
        Ok(TaskContext {
            stack,
            frames: std::mem::take(&mut self.frames),
            last_result: std::mem::replace(&mut self.last_result, Value::nil()),
            instruction_budget: std::mem::replace(&mut self.instruction_budget, u64::MAX),
        })
    }

    pub(super) fn restore_active_context(
        &mut self,
        context: TaskContext,
    ) -> Result<(), RuntimeError> {
        if context.stack.len() > self.stack.len() {
            return Err(RuntimeError::StackOverflow);
        }
        self.stack[..context.stack.len()].copy_from_slice(&context.stack);
        self.frames = context.frames;
        self.last_result = context.last_result;
        self.instruction_budget = context.instruction_budget;
        self.open_upvalues = None;
        Ok(())
    }

    pub(super) fn suspend_active_context(
        &mut self,
        owner: Option<u32>,
    ) -> Result<(), RuntimeError> {
        let context = self.capture_active_context()?;
        if let Some(id) = owner {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(id)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            if !matches!(task.state, TaskState::Running) {
                return Err(RuntimeError::task_failed(
                    "active task was not in the running state",
                ));
            }
            task.state = TaskState::Suspended(context);
        } else {
            let root = &mut self.task_scheduler.root_task;
            if !matches!(root.state, TaskState::Running) {
                return Err(RuntimeError::task_failed(
                    "root task was not in the running state",
                ));
            }
            root.state = TaskState::Suspended(context);
        }
        Ok(())
    }

    pub(super) fn resume_active_context(&mut self, owner: Option<u32>) -> Result<(), RuntimeError> {
        let context = if let Some(id) = owner {
            let task = self
                .task_scheduler
                .tasks
                .get_mut(id)
                .ok_or(RuntimeError::InvalidTaskHandle)?;
            match std::mem::replace(&mut task.state, TaskState::Running) {
                TaskState::Suspended(context) => context,
                state => {
                    task.state = state;
                    return Err(RuntimeError::task_failed(
                        "task resumed without a suspended context",
                    ));
                }
            }
        } else {
            let root = &mut self.task_scheduler.root_task;
            match std::mem::replace(&mut root.state, TaskState::Running) {
                TaskState::Suspended(context) => context,
                state => {
                    root.state = state;
                    return Err(RuntimeError::task_failed(
                        "root task context was not suspended",
                    ));
                }
            }
        };
        self.restore_active_context(context)
    }
}
