use crate::error::RuntimeError;
use crate::opcode::{instruction::*, OpCode};
use memory::Value;

use super::arithmetic::ArithmeticOps;
use super::closure::ClosureOps;
use super::comparison::ComparisonOps;
use super::control::ControlFlowOps;
use super::data::DataOps;
use super::globals::GlobalOps;
use super::iterator::IteratorOps;
use super::method_call::MethodCallOps;
use super::stack::StackOps;
use super::value_ops::ValueOps;
use super::vm::STACK_MAX;

/// Trait for the main interpretation loop
pub trait InterpreterOps {
    fn interpret_inner(&mut self) -> Result<(), RuntimeError>;
}

impl InterpreterOps for super::vm::VM {
    fn interpret_inner(&mut self) -> Result<(), RuntimeError> {
        // Validation: Check initial frame fits
        if let Some(frame) = self.frames.last() {
            let closure = self
                .heap
                .get_closure(frame.closure)
                .ok_or(RuntimeError::FunctionNotFound)?;
            let func = self
                .heap
                .get_function(closure.function)
                .ok_or(RuntimeError::FunctionNotFound)?;
            if frame.base + (func.max_slots as usize) >= STACK_MAX {
                return Err(RuntimeError::StackOverflow);
            }
        }

        self.run_until_frame_depth(0)
    }
}

impl super::vm::VM {
    /// Run the interpreter loop until the frame stack drops to `target_depth`.
    ///
    /// This is the core execution engine. `interpret_inner()` delegates here
    /// with `target_depth = 0` (run until all frames are consumed).
    /// `call_value()` uses a higher target to execute a single closure call
    /// and return to the native caller.
    pub(crate) fn run_until_frame_depth(
        &mut self,
        target_depth: usize,
    ) -> Result<(), RuntimeError> {
        // Dispatch cache: avoid re-fetching closure→function on every
        // instruction when the frame hasn't changed. In a tight loop
        // (no Call/Return), the closure stays the same across iterations,
        // so we skip the expensive Arena::get (HashSet `is_free` check)
        // on every instruction.
        //
        // Safety: the current frame's closure is GC-rooted (mark_roots
        // marks all frame closures), so both the closure and its function
        // are guaranteed live. GC is mark-sweep without compaction, so
        // arena indices remain stable.
        let mut cached_closure_idx: u32 = u32::MAX; // sentinel: cache empty
        let mut cached_func_idx: u32 = 0;

        // Batch GC checks: instead of checking `request_gc` and
        // `heap_limit_exceeded` on every instruction (two branch checks),
        // check every GC_CHECK_INTERVAL instructions. The flags are sticky
        // (set by alloc, stay true until cleared), so delaying the check by
        // up to 1024 instructions is safe — the worst case is slightly
        // delayed collection, which is negligible.
        // In stress_mode, check every instruction (stress_mode forces GC
        // on every instruction for testing correctness).
        const GC_CHECK_INTERVAL: u32 = 1024;
        let mut gc_countdown: u32 = if self.stress_mode {
            1
        } else {
            GC_CHECK_INTERVAL
        };

        while self.frames.len() > target_depth {
            // Batched GC check point
            gc_countdown -= 1;
            if gc_countdown == 0 || self.heap.heap_limit_exceeded {
                gc_countdown = if self.stress_mode {
                    1
                } else {
                    GC_CHECK_INTERVAL
                };

                self.poll_gc()?;
            }

            let frame_idx = self.frames.len() - 1;

            let (closure_idx, ip, base) = {
                let f = &self.frames[frame_idx];
                (f.closure, f.ip, f.base)
            };

            // Fast path: reuse cached function index when closure hasn't changed
            let func_idx = if closure_idx == cached_closure_idx {
                cached_func_idx
            } else {
                let closure = self
                    .heap
                    .get_closure(closure_idx)
                    .ok_or(RuntimeError::FunctionNotFound)?;
                let fi = closure.function;
                cached_closure_idx = closure_idx;
                cached_func_idx = fi;
                fi
            };

            // SAFETY: The current frame's closure is GC-rooted, so the
            // function it references is reachable and will not be freed.
            // The func_idx was obtained from a validated get_closure call.
            let func = unsafe { self.heap.get_function_unchecked(func_idx) };

            if ip >= func.chunk.len() {
                if let Some(frame) = self.frames.pop() {
                    if self.frames.is_empty() {
                        self.last_result = self.stack[frame.base];
                    }
                }
                continue;
            }

            let instruction = func.chunk[ip];
            let max_slots = func.max_slots as usize;
            let chunk_len = func.chunk.len();
            self.frames[frame_idx].ip += 1;

            // Instruction fuel check (u64::MAX = unlimited, wrapping
            // decrement means it never reaches 0 from MAX).
            if self.instruction_budget == 0 {
                return Err(RuntimeError::InstructionBudgetExhausted);
            }
            self.instruction_budget = self.instruction_budget.wrapping_sub(1);

            let op_byte = decode_opcode(instruction);
            let op = OpCode::from_u8(op_byte).ok_or(RuntimeError::InvalidOpcode(op_byte))?;

            use crate::opcode::OpCode::*;

            match op {
                // Arithmetic (delegated to arithmetic.rs)
                Add | Sub | Mul | Div | Mod | Pow | Neg => {
                    self.handle_arithmetic(op, instruction, base)?;
                }

                // Control Flow (delegated to control.rs)
                Call | Return => {
                    self.handle_control(op, instruction, base)?;
                }

                // Globals (delegated to globals.rs)
                DefGlobalVar | DefGlobalLet | GetGlobal | SetGlobal => {
                    self.handle_globals(op, instruction, base, closure_idx)?;
                }

                // Control Flow - Jumps
                Jump => {
                    let dest = decode_bx(instruction) as usize;
                    if dest > chunk_len {
                        return Err(RuntimeError::out_of_bounds(format!(
                            "Jump target {dest} exceeds chunk length {chunk_len}"
                        )));
                    }
                    self.frames[frame_idx].ip = dest;
                }

                JumpIfFalse => {
                    let a = decode_a(instruction) as usize;
                    let dest = decode_bx(instruction) as usize;
                    if dest > chunk_len {
                        return Err(RuntimeError::out_of_bounds(format!(
                            "JumpIfFalse target {dest} exceeds chunk length {chunk_len}"
                        )));
                    }
                    let val = self.get_reg(base, a)?;
                    if val.is_falsey() {
                        self.frames[frame_idx].ip = dest;
                    }
                }

                // Comparisons (delegated to comparison.rs)
                Eq | Lt | Gt | NotEq | Le | Ge | LogNot => {
                    self.handle_comparison(op, instruction, base)?;
                }

                // Constants & Moves
                LoadConst => {
                    let a = decode_a(instruction) as usize;
                    let bx = decode_bx(instruction) as usize;
                    let val = func.constants.get(bx).cloned().ok_or_else(|| {
                        RuntimeError::out_of_bounds(format!(
                            "constant index {bx} out of range (len {})",
                            func.constants.len()
                        ))
                    })?;
                    self.set_reg(base, a, val)?;
                }

                LoadTrue => {
                    let a = decode_a(instruction) as usize;
                    self.set_reg(base, a, Value::true_val())?;
                }

                LoadFalse => {
                    let a = decode_a(instruction) as usize;
                    self.set_reg(base, a, Value::false_val())?;
                }

                LoadNil => {
                    let a = decode_a(instruction) as usize;
                    self.set_reg(base, a, Value::nil())?;
                }

                Move => {
                    let a = decode_a(instruction) as usize;
                    let b = decode_b(instruction) as usize;
                    let val = self.get_reg(base, b)?;
                    self.set_reg(base, a, val)?;
                }

                Print => {
                    let a = decode_a(instruction) as usize;
                    let val = self.get_reg(base, a)?;
                    println!("{}", self.val_to_string(&val));
                }

                Prove => {
                    self.handle_prove(instruction, base, closure_idx)?;
                }

                MethodCall => self.handle_method_call(instruction, base)?,

                BuildList | BuildMap | GetIndex | SetIndex => {
                    self.handle_data(op, instruction, base)?;
                }

                // Closures & Upvalues (delegated to closure.rs)
                OpCode::GetUpvalue | SetUpvalue | CloseUpvalue | Closure => {
                    self.handle_closure(op, instruction, base, frame_idx)?;
                }

                // Iterators (delegated to iterator.rs)
                OpCode::GetIter | OpCode::ForIter => {
                    self.handle_iterator(op, instruction, base, frame_idx, max_slots, chunk_len)?;
                }

                CallCircomTemplate => {
                    self.handle_call_circom_template(instruction, base)?;
                }

                Nop => { /* no-op */ }
            }
        }
        Ok(())
    }

    pub(crate) fn execute_compiled_runtime_instruction(
        &mut self,
        frame_index: usize,
        instruction_index: usize,
    ) -> Result<(), RuntimeError> {
        let (closure_index, base, instruction, constant) = {
            let frame = self
                .frames
                .get(frame_index)
                .ok_or(RuntimeError::StackUnderflow)?;
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
                .ok_or_else(|| RuntimeError::out_of_bounds("compiled instruction index"))?;
            let constant = function
                .constants
                .get(decode_bx(instruction) as usize)
                .copied();
            (frame.closure, frame.base, instruction, constant)
        };
        let opcode_byte = decode_opcode(instruction);
        let opcode =
            OpCode::from_u8(opcode_byte).ok_or(RuntimeError::InvalidOpcode(opcode_byte))?;

        use crate::opcode::OpCode::*;
        match opcode {
            DefGlobalVar | DefGlobalLet | GetGlobal | SetGlobal => {
                self.handle_globals(opcode, instruction, base, closure_index)
            }
            Closure | GetUpvalue | SetUpvalue | CloseUpvalue => {
                self.handle_closure(opcode, instruction, base, frame_index)
            }
            BuildList | BuildMap | GetIndex | SetIndex => {
                self.handle_data(opcode, instruction, base)
            }
            Eq | NotEq => self.handle_comparison(opcode, instruction, base),
            MethodCall => self.handle_method_call(instruction, base),
            LoadConst => {
                let value = constant.ok_or_else(|| {
                    RuntimeError::out_of_bounds("compiled runtime constant index")
                })?;
                self.set_reg(base, decode_a(instruction) as usize, value)
            }
            Print => {
                let value = self.get_reg(base, decode_a(instruction) as usize)?;
                println!("{}", self.val_to_string(&value));
                Ok(())
            }
            _ => Err(RuntimeError::InvalidOpcode(opcode as u8)),
        }
    }

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
}
