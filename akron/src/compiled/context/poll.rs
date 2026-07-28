use std::ffi::c_void;

use crate::RuntimeError;

use super::{run_status_helper, GC_CHECK_INTERVAL};
use crate::compiled::abi::{
    RuntimeStatus, STATUS_BAILOUT_REQUIRED, STATUS_INVALID_ARGUMENT, STATUS_OK,
};

fn advance_gc_countdown(countdown: u32, instruction_count: u32) -> u32 {
    if instruction_count < countdown {
        return countdown - instruction_count;
    }

    let after_first_check = instruction_count - countdown;
    let remainder = after_first_check % GC_CHECK_INTERVAL;
    if remainder == 0 {
        GC_CHECK_INTERVAL
    } else {
        GC_CHECK_INTERVAL - remainder
    }
}

pub(crate) unsafe extern "C" fn poll_block(
    context: *mut c_void,
    frame_index: u32,
    first_instruction: u32,
    instruction_count: u32,
) -> RuntimeStatus {
    poll_block_entry(
        context,
        frame_index,
        first_instruction,
        instruction_count,
        0,
        false,
    )
}

pub(crate) unsafe extern "C" fn poll_tier1_block(
    context: *mut c_void,
    frame_index: u32,
    first_instruction: u32,
    instruction_count: u32,
    direct_instruction_count: u32,
) -> RuntimeStatus {
    if direct_instruction_count > instruction_count {
        return STATUS_INVALID_ARGUMENT;
    }
    poll_block_entry(
        context,
        frame_index,
        first_instruction,
        instruction_count,
        direct_instruction_count,
        true,
    )
}

fn poll_block_entry(
    context: *mut c_void,
    frame_index: u32,
    first_instruction: u32,
    instruction_count: u32,
    direct_instruction_count: u32,
    record_stats: bool,
) -> RuntimeStatus {
    if instruction_count == 0 {
        return STATUS_INVALID_ARGUMENT;
    }
    let Some(next_instruction) = first_instruction.checked_add(instruction_count) else {
        return STATUS_INVALID_ARGUMENT;
    };

    run_status_helper(context, |state| {
        if record_stats {
            state.stats.record_block_poll();
            state.stats.record_slow_poll_entry();
        }
        let (stress_mode, instruction_budget) = {
            let vm = unsafe { state.vm_mut() };
            if vm.frames.get(frame_index as usize).is_none() {
                return Err(RuntimeError::StackUnderflow);
            }
            (vm.stress_mode, vm.instruction_budget)
        };
        if stress_mode || instruction_budget < u64::from(instruction_count) {
            if record_stats {
                state.stats.record_slow_path();
            }
            return Ok(STATUS_BAILOUT_REQUIRED);
        }

        let (request_gc, heap_limit_exceeded) = {
            let vm = unsafe { state.vm_mut() };
            (vm.heap.request_gc, vm.heap.heap_limit_exceeded)
        };
        let countdown = state.gc_countdown;
        if heap_limit_exceeded {
            state.gc_countdown = GC_CHECK_INTERVAL;
            unsafe { state.vm_mut() }.poll_gc()?;
            state.gc_countdown =
                advance_gc_countdown(state.gc_countdown, instruction_count.saturating_sub(1));
        } else {
            if request_gc && countdown <= instruction_count {
                unsafe { state.vm_mut() }.poll_gc()?;
            }
            state.gc_countdown = advance_gc_countdown(countdown, instruction_count);
        }

        let vm = unsafe { state.vm_mut() };
        vm.instruction_budget = vm
            .instruction_budget
            .wrapping_sub(u64::from(instruction_count));
        vm.frames[frame_index as usize].ip = next_instruction as usize;
        if record_stats {
            state
                .stats
                .record_direct_instructions(direct_instruction_count);
        }
        Ok(STATUS_OK)
    })
}
