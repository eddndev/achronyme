use std::ffi::c_void;

use crate::opcode::instruction::decode_opcode;
use crate::OpCode;

use super::abi::{RuntimeStatus, STATUS_INVALID_ARGUMENT, STATUS_OK, STATUS_SPECIALIZATION_MISS};
use super::context::run_status_helper;

pub(super) unsafe extern "C" fn list_push(
    context: *mut c_void,
    base: u32,
    instruction: u32,
) -> RuntimeStatus {
    if decode_opcode(instruction) != OpCode::MethodCall.as_u8() {
        return STATUS_INVALID_ARGUMENT;
    }
    run_status_helper(context, |state| {
        let hit = unsafe { state.vm_mut() }
            .execute_list_push_specialization(base as usize, instruction)?;
        if hit {
            state.stats_mut().record_specialization_hits(2);
            Ok(STATUS_OK)
        } else {
            state.stats_mut().record_specialization_miss();
            Ok(STATUS_SPECIALIZATION_MISS)
        }
    })
}

pub(super) unsafe extern "C" fn list_index(
    context: *mut c_void,
    base: u32,
    instruction: u32,
) -> RuntimeStatus {
    if decode_opcode(instruction) != OpCode::GetIndex.as_u8() {
        return STATUS_INVALID_ARGUMENT;
    }
    run_status_helper(context, |state| {
        let hit = unsafe { state.vm_mut() }
            .execute_list_index_specialization(base as usize, instruction)?;
        if hit {
            state.stats_mut().record_specialization_hits(1);
            Ok(STATUS_OK)
        } else {
            state.stats_mut().record_specialization_miss();
            Ok(STATUS_SPECIALIZATION_MISS)
        }
    })
}
