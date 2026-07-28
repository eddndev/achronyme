use crate::error::RuntimeError;
use crate::opcode::{instruction::*, OpCode};
use memory::Value;

use super::stack::StackOps;
use super::upvalue::UpvalueOps;

/// Trait for control flow instruction handlers
pub trait ControlFlowOps {
    fn handle_control(
        &mut self,
        op: OpCode,
        instruction: u32,
        base: usize,
    ) -> Result<(), RuntimeError>;

    fn call_native(
        &mut self,
        func_val: Value,
        args_start: usize,
        args_count: usize,
        base: usize,
        result_reg: usize,
    ) -> Result<(), RuntimeError>;
}

pub(crate) struct ClosureFrameSpec {
    pub closure: u32,
    pub arity: u8,
    pub max_slots: u16,
    pub args_start: usize,
    pub args_count: usize,
    pub caller_base: usize,
    pub result_register: usize,
}

impl ControlFlowOps for super::vm::VM {
    fn handle_control(
        &mut self,
        op: OpCode,
        instruction: u32,
        base: usize,
    ) -> Result<(), RuntimeError> {
        match op {
            OpCode::Call => {
                let a = decode_a(instruction) as usize;
                let b = decode_b(instruction) as usize;
                let c = decode_c(instruction) as usize;

                let func_val = self.get_reg(base, b)?;
                let args_start = base + b + 1;
                let args_count = c;

                if args_start + args_count > self.stack.len() {
                    return Err(RuntimeError::StackUnderflow);
                }

                if func_val.is_native() {
                    self.call_native(func_val, args_start, args_count, base, a)?;
                } else if func_val.is_closure() {
                    let handle = func_val
                        .as_handle()
                        .ok_or_else(|| RuntimeError::type_mismatch("Expected closure handle"))?;

                    let (arity, max_slots) = {
                        let closure = self
                            .heap
                            .get_closure(handle)
                            .ok_or(RuntimeError::FunctionNotFound)?;
                        let func = self
                            .heap
                            .get_function(closure.function)
                            .ok_or(RuntimeError::FunctionNotFound)?;
                        (func.arity, func.max_slots)
                    };
                    self.push_closure_frame(ClosureFrameSpec {
                        closure: handle,
                        arity,
                        max_slots,
                        args_start,
                        args_count,
                        caller_base: base,
                        result_register: a,
                    })?;
                } else {
                    return Err(RuntimeError::type_mismatch(
                        "Call target must be Closure or Native",
                    ));
                }
            }

            OpCode::Return => {
                let a = decode_a(instruction) as usize;
                let b = decode_b(instruction) as usize; // 1 if has value, 0 if nil

                // Get return value

                let ret_val = if b == 1 {
                    self.get_reg(base, a)?
                } else {
                    Value::nil()
                };

                // Close Upvalues for current frame
                // "base" is the start of the current frame (R0)
                // Any upvalue pointing to base or higher is now invalid/closed
                self.close_upvalues(base)?;

                // Pop current frame and get its dest_reg
                if let Some(frame) = self.frames.pop() {
                    // Write return value to dest_reg (absolute stack index)
                    // Only if there's still a caller (not the top-level script)
                    if !self.frames.is_empty() {
                        self.set_reg(0, frame.dest_reg, ret_val)?;
                    } else {
                        self.last_result = ret_val;
                    }
                }
            }

            _ => return Err(RuntimeError::InvalidOpcode(op as u8)),
        }

        Ok(())
    }

    fn call_native(
        &mut self,
        func_val: Value,
        args_start: usize,
        args_count: usize,
        base: usize,
        result_reg: usize,
    ) -> Result<(), RuntimeError> {
        let handle = func_val
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("Expected native handle"))?;
        let (func, arity) = {
            let n = self
                .natives
                .get(handle as usize)
                .ok_or(RuntimeError::FunctionNotFound)?;
            (n.func, n.arity)
        };

        if arity != -1 && arity as usize != args_count {
            return Err(RuntimeError::arity_mismatch(format!(
                "Expected {} args, got {}",
                arity, args_count
            )));
        }

        let args: Vec<Value> = self.stack[args_start..args_start + args_count].to_vec();
        let res = func(self, &args)?;
        self.set_reg(base, result_reg, res)?;

        Ok(())
    }
}

impl super::vm::VM {
    pub(crate) fn push_closure_frame(
        &mut self,
        spec: ClosureFrameSpec,
    ) -> Result<(), RuntimeError> {
        if spec.arity as usize != spec.args_count {
            return Err(RuntimeError::arity_mismatch(format!(
                "Expected {} args, got {}",
                spec.arity, spec.args_count
            )));
        }
        if spec.args_start + spec.max_slots as usize >= crate::machine::vm::STACK_MAX {
            return Err(RuntimeError::StackOverflow);
        }
        let dest_reg = spec
            .caller_base
            .checked_add(spec.result_register)
            .filter(|&destination| destination < crate::machine::vm::STACK_MAX)
            .ok_or(RuntimeError::StackOverflow)?;
        if self.frames.len() >= crate::machine::vm::MAX_FRAMES {
            return Err(RuntimeError::StackOverflow);
        }
        self.frames.push(crate::machine::frame::CallFrame {
            closure: spec.closure,
            ip: 0,
            base: spec.args_start,
            dest_reg,
        });
        Ok(())
    }
}
