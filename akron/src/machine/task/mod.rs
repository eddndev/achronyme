use super::stack::StackOps;
use super::vm::VM;
use crate::error::RuntimeError;
use crate::opcode::{instruction::*, OpCode};

mod awaiting;
mod cancellation;
mod diagnostics;
mod outcomes;
mod resources;
mod scheduler;
mod spawn;
mod state;
pub use diagnostics::{
    TaskAwaitMode, TaskDiagnostic, TaskDiagnosticState, TaskFailureDiagnostic, TaskWaitReason,
};
pub(crate) use state::TaskScheduler;
impl VM {
    pub(crate) fn handle_task_instruction(
        &mut self,
        op: OpCode,
        instruction: u32,
        base: usize,
    ) -> Result<(), RuntimeError> {
        match op {
            OpCode::ScopeEnter => self.enter_task_scope(),
            OpCode::ScopeExit => self.exit_task_scope(),
            OpCode::Spawn => {
                let destination = decode_a(instruction) as usize;
                let callee_register = decode_b(instruction) as usize;
                let argument_count = decode_c(instruction) as usize;
                let callee = self.get_reg(base, callee_register)?;
                let argument_start = base + callee_register + 1;
                let argument_end = argument_start
                    .checked_add(argument_count)
                    .filter(|end| *end <= self.stack.len())
                    .ok_or(RuntimeError::StackOverflow)?;
                let arguments = self.stack[argument_start..argument_end].to_vec();
                let task = self.spawn_task(callee, arguments)?;
                self.set_reg(base, destination, task)
            }
            OpCode::Await | OpCode::AwaitOutcome => {
                let destination = decode_a(instruction) as usize;
                let task_register = decode_b(instruction) as usize;
                let task = self.get_reg(base, task_register)?;
                let absolute_destination = base
                    .checked_add(destination)
                    .filter(|index| *index < self.stack.len())
                    .ok_or(RuntimeError::StackOverflow)?;
                let result =
                    self.await_task(task, absolute_destination, op == OpCode::AwaitOutcome)?;
                self.set_reg(base, destination, result)
            }
            OpCode::AwaitRace => {
                let destination = decode_a(instruction) as usize;
                let start = decode_b(instruction) as usize;
                let count = decode_c(instruction) as usize;
                let end = base
                    .checked_add(start)
                    .and_then(|start| start.checked_add(count))
                    .filter(|end| *end <= self.stack.len())
                    .ok_or(RuntimeError::StackOverflow)?;
                let tasks = self.stack[base + start..end].to_vec();
                let absolute_destination = base
                    .checked_add(destination)
                    .filter(|index| *index < self.stack.len())
                    .ok_or(RuntimeError::StackOverflow)?;
                let result = self.await_task_race(&tasks, absolute_destination)?;
                self.set_reg(base, destination, result)
            }
            OpCode::TaskForget => {
                let task_register = decode_a(instruction) as usize;
                let task = self.get_reg(base, task_register)?;
                self.forget_task_handle(task)
            }
            OpCode::CancelCheck => self.check_task_cancellation(),
            _ => Err(RuntimeError::InvalidOpcode(op.as_u8())),
        }
    }
}
