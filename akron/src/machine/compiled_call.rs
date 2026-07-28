use crate::error::RuntimeError;
use crate::opcode::instruction::decode_opcode;
use crate::OpCode;

use super::control::{ClosureFrameSpec, ControlFlowOps};

impl super::vm::VM {
    pub(crate) fn prepare_compiled_call(
        &mut self,
        frame_index: usize,
        instruction_index: usize,
    ) -> Result<super::CompiledCallTarget, RuntimeError> {
        if frame_index + 1 != self.frames.len() {
            return Err(RuntimeError::StackUnderflow);
        }
        let (base, instruction) = {
            let frame = &self.frames[frame_index];
            let closure = self
                .heap
                .get_closure(frame.closure)
                .ok_or(RuntimeError::FunctionNotFound)?;
            let function = self
                .heap
                .get_function(closure.function)
                .ok_or(RuntimeError::FunctionNotFound)?;
            let instruction = *function
                .chunk
                .get(instruction_index)
                .ok_or_else(|| RuntimeError::out_of_bounds("compiled call instruction"))?;
            (frame.base, instruction)
        };
        if OpCode::from_u8(decode_opcode(instruction)) != Some(OpCode::Call) {
            return Err(RuntimeError::InvalidOperand);
        }

        let previous_depth = self.frames.len();
        self.handle_control(OpCode::Call, instruction, base)?;
        if self.frames.len() == previous_depth {
            return Ok(super::CompiledCallTarget::NativeComplete);
        }
        if self.frames.len() != previous_depth + 1 {
            return Err(RuntimeError::StackOverflow);
        }

        let callee_index = self.frames.len() - 1;
        let callee = &self.frames[callee_index];
        let closure = self
            .heap
            .get_closure(callee.closure)
            .ok_or(RuntimeError::FunctionNotFound)?;
        let prototype_index = self
            .prototypes
            .iter()
            .position(|&function| function == closure.function);
        let frame_index = u32::try_from(callee_index).map_err(|_| RuntimeError::StackOverflow)?;
        let base = u32::try_from(callee.base).map_err(|_| RuntimeError::StackOverflow)?;
        match prototype_index {
            Some(prototype_index) => Ok(super::CompiledCallTarget::Prototype {
                frame_index,
                base,
                prototype_index: u32::try_from(prototype_index)
                    .map_err(|_| RuntimeError::FunctionNotFound)?,
            }),
            None => Ok(super::CompiledCallTarget::InterpreterRequired { frame_index, base }),
        }
    }

    pub(crate) fn prepare_known_compiled_call(
        &mut self,
        frame_index: usize,
        result_register: usize,
        callee_register: usize,
        args_count: usize,
        expected_prototype: usize,
    ) -> Result<Option<(u32, u32)>, RuntimeError> {
        if frame_index + 1 != self.frames.len() {
            return Err(RuntimeError::StackUnderflow);
        }
        let base = self.frames[frame_index].base;
        let callee_index = base
            .checked_add(callee_register)
            .ok_or(RuntimeError::StackOverflow)?;
        let args_start = callee_index
            .checked_add(1)
            .ok_or(RuntimeError::StackOverflow)?;
        let args_end = args_start
            .checked_add(args_count)
            .ok_or(RuntimeError::StackOverflow)?;
        if args_end > self.stack.len() {
            return Err(RuntimeError::StackUnderflow);
        }

        let Some(expected_function) = self.prototypes.get(expected_prototype).copied() else {
            return Ok(None);
        };
        let Some(handle) = self.stack[callee_index].as_handle() else {
            return Ok(None);
        };
        if !self.stack[callee_index].is_closure() {
            return Ok(None);
        }
        let Some(closure) = self.heap.get_closure(handle) else {
            return Ok(None);
        };
        if closure.function != expected_function {
            return Ok(None);
        }
        let Some(function) = self.heap.get_function(closure.function) else {
            return Ok(None);
        };
        let (arity, max_slots) = (function.arity, function.max_slots);
        self.push_closure_frame(ClosureFrameSpec {
            closure: handle,
            arity,
            max_slots,
            args_start,
            args_count,
            caller_base: base,
            result_register,
        })?;

        let callee_frame = self.frames.len() - 1;
        let frame_index = u32::try_from(callee_frame).map_err(|_| RuntimeError::StackOverflow)?;
        let base = u32::try_from(args_start).map_err(|_| RuntimeError::StackOverflow)?;
        Ok(Some((frame_index, base)))
    }
}
