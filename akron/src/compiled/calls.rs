use std::ffi::c_void;

use memory::Value;

use crate::machine::CompiledCallTarget;
use crate::opcode::instruction::{decode_a, decode_b, decode_c, decode_opcode};
use crate::OpCode;

use super::abi::{
    RuntimeStatus, STATUS_CALL_INTERPRETER_REQUIRED, STATUS_INVALID_ARGUMENT,
    STATUS_KNOWN_CALL_MISS, STATUS_NATIVE_CALL_COMPLETE, STATUS_OK,
};
use super::context::{run_helper, run_status_helper};

pub(super) unsafe extern "C" fn execute_instruction(
    context: *mut c_void,
    frame_index: u32,
    instruction_index: u32,
) -> RuntimeStatus {
    run_helper(context, |state| {
        state.stats_mut().record_runtime_call();
        unsafe { state.vm_mut() }
            .execute_compiled_runtime_instruction(frame_index as usize, instruction_index as usize)
    })
}

pub(super) unsafe extern "C" fn prepare_call(
    context: *mut c_void,
    frame_index: u32,
    instruction_index: u32,
    callee_frame_output: *mut u32,
    callee_base_output: *mut u32,
    prototype_output: *mut u32,
) -> RuntimeStatus {
    if callee_frame_output.is_null() || callee_base_output.is_null() || prototype_output.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }

    run_status_helper(context, |state| {
        let target = unsafe { state.vm_mut() }
            .prepare_compiled_call(frame_index as usize, instruction_index as usize)?;
        match target {
            CompiledCallTarget::NativeComplete => {
                state.stats_mut().record_native_function_call();
                Ok(STATUS_NATIVE_CALL_COMPLETE)
            }
            CompiledCallTarget::Prototype {
                frame_index,
                base,
                prototype_index,
            } => {
                state.stats_mut().record_compiled_function_call();
                unsafe {
                    callee_frame_output.write(frame_index);
                    callee_base_output.write(base);
                    prototype_output.write(prototype_index);
                }
                Ok(STATUS_OK)
            }
            CompiledCallTarget::InterpreterRequired { frame_index, base } => {
                unsafe {
                    callee_frame_output.write(frame_index);
                    callee_base_output.write(base);
                }
                Ok(STATUS_CALL_INTERPRETER_REQUIRED)
            }
        }
    })
}

pub(super) unsafe extern "C" fn prepare_known_call(
    context: *mut c_void,
    frame_index: u32,
    instruction: u32,
    expected_prototype: u32,
    callee_frame_output: *mut u32,
    callee_base_output: *mut u32,
) -> RuntimeStatus {
    if callee_frame_output.is_null() || callee_base_output.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    if decode_opcode(instruction) != OpCode::Call.as_u8() {
        return STATUS_INVALID_ARGUMENT;
    }

    run_status_helper(context, |state| {
        let target = unsafe { state.vm_mut() }.prepare_known_compiled_call(
            frame_index as usize,
            decode_a(instruction) as usize,
            decode_b(instruction) as usize,
            decode_c(instruction) as usize,
            expected_prototype as usize,
        )?;
        let Some((frame_index, base)) = target else {
            state.stats_mut().record_known_call_fast_miss();
            return Ok(STATUS_KNOWN_CALL_MISS);
        };

        state.stats_mut().record_known_call_fast_hit();
        state.stats_mut().record_compiled_function_call();
        unsafe {
            callee_frame_output.write(frame_index);
            callee_base_output.write(base);
        }
        Ok(STATUS_OK)
    })
}

pub(super) unsafe extern "C" fn finish_call(
    context: *mut c_void,
    frame_index: u32,
    result_bits: u64,
) -> RuntimeStatus {
    run_helper(context, |state| {
        let result =
            Value::from_abi_bits(result_bits).ok_or(crate::RuntimeError::InvalidOperand)?;
        unsafe { state.vm_mut() }.finish_compiled_call(frame_index as usize, result)
    })
}
