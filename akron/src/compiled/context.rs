use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{self, NonNull};

use memory::Value;

use crate::{RuntimeError, VM};

use super::abi::{
    RuntimeStatus, ERROR_ASSERTION_FAILED, ERROR_DIVISION_BY_ZERO, ERROR_INTEGER_OVERFLOW,
    ERROR_INVALID_OPERAND, ERROR_STACK_OVERFLOW, STATUS_BAILOUT_REQUIRED, STATUS_INTERNAL_ERROR,
    STATUS_INTERPRETER_COMPLETED, STATUS_INVALID_ARGUMENT, STATUS_OK, STATUS_RUNTIME_ERROR,
};

const GC_CHECK_INTERVAL: u32 = 1024;
const _: () = assert!(std::mem::size_of::<Value>() == std::mem::size_of::<u64>());
const _: () = assert!(std::mem::align_of::<Value>() == std::mem::align_of::<u64>());

pub(super) struct RuntimeContextState {
    vm: NonNull<VM>,
    pending_error: Option<RuntimeError>,
    gc_countdown: u32,
    bailout_ip: Option<u32>,
}

impl RuntimeContextState {
    pub(super) unsafe fn vm_mut(&mut self) -> &mut VM {
        unsafe { self.vm.as_mut() }
    }

    fn fail(&mut self, error: RuntimeError) -> RuntimeStatus {
        self.pending_error = Some(error);
        STATUS_RUNTIME_ERROR
    }
}

/// Owns the exclusive VM borrow used by one compiled invocation.
pub struct RuntimeContext<'vm> {
    state: RuntimeContextState,
    _vm: PhantomData<&'vm mut VM>,
}

impl<'vm> RuntimeContext<'vm> {
    pub fn new(vm: &'vm mut VM) -> Self {
        let gc_countdown = if vm.stress_mode { 1 } else { GC_CHECK_INTERVAL };
        Self {
            state: RuntimeContextState {
                vm: NonNull::from(vm),
                pending_error: None,
                gc_countdown,
                bailout_ip: None,
            },
            _vm: PhantomData,
        }
    }

    pub fn as_opaque(&mut self) -> *mut c_void {
        ptr::from_mut(&mut self.state).cast()
    }

    pub fn bailout_ip(&self) -> Option<u32> {
        self.state.bailout_ip
    }

    pub fn finish(mut self, status: RuntimeStatus) -> Result<(), RuntimeError> {
        if matches!(status, STATUS_OK | STATUS_INTERPRETER_COMPLETED)
            && self.state.pending_error.is_none()
        {
            return Ok(());
        }

        let error = self
            .state
            .pending_error
            .take()
            .unwrap_or_else(|| match status {
                STATUS_INVALID_ARGUMENT => RuntimeError::InvalidOperand,
                _ => RuntimeError::resource_limit_exceeded(
                    "compiled runtime returned an internal error without details",
                ),
            });
        unsafe { self.state.vm_mut() }.capture_error_location();
        Err(error)
    }
}

pub(super) fn run_helper<F>(context: *mut c_void, operation: F) -> RuntimeStatus
where
    F: FnOnce(&mut RuntimeContextState) -> Result<(), RuntimeError>,
{
    run_status_helper(context, |state| {
        operation(state)?;
        Ok(STATUS_OK)
    })
}

pub(super) fn run_status_helper<F>(context: *mut c_void, operation: F) -> RuntimeStatus
where
    F: FnOnce(&mut RuntimeContextState) -> Result<RuntimeStatus, RuntimeError>,
{
    if context.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }

    let state = unsafe { &mut *context.cast::<RuntimeContextState>() };
    match catch_unwind(AssertUnwindSafe(|| operation(state))) {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => state.fail(error),
        Err(_) => {
            state.pending_error = Some(RuntimeError::resource_limit_exceeded(
                "compiled runtime helper panicked",
            ));
            STATUS_INTERNAL_ERROR
        }
    }
}

pub(super) unsafe extern "C" fn register_window(
    context: *mut c_void,
    base: u32,
    register_count: u32,
    output: *mut *mut u64,
) -> RuntimeStatus {
    if output.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    run_helper(context, |state| {
        let base = base as usize;
        let end = base
            .checked_add(register_count as usize)
            .ok_or_else(|| RuntimeError::out_of_bounds("register window overflow"))?;
        let stack = &mut unsafe { state.vm_mut() }.stack;
        if end > stack.len() {
            return Err(RuntimeError::out_of_bounds(format!(
                "register window {base}..{end}"
            )));
        }
        let window = unsafe { stack.as_mut_ptr().add(base).cast::<u64>() };
        unsafe { output.write(window) };
        Ok(())
    })
}

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

pub(super) unsafe extern "C" fn poll_block(
    context: *mut c_void,
    frame_index: u32,
    first_instruction: u32,
    instruction_count: u32,
) -> RuntimeStatus {
    if instruction_count == 0 {
        return STATUS_INVALID_ARGUMENT;
    }
    let Some(next_instruction) = first_instruction.checked_add(instruction_count) else {
        return STATUS_INVALID_ARGUMENT;
    };

    run_status_helper(context, |state| {
        let (stress_mode, instruction_budget) = {
            let vm = unsafe { state.vm_mut() };
            if vm.frames.get(frame_index as usize).is_none() {
                return Err(RuntimeError::StackUnderflow);
            }
            (vm.stress_mode, vm.instruction_budget)
        };
        if stress_mode || instruction_budget < u64::from(instruction_count) {
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
        Ok(STATUS_OK)
    })
}

pub(super) unsafe extern "C" fn refund_block(
    context: *mut c_void,
    frame_index: u32,
    next_instruction: u32,
    refund_count: u32,
) -> RuntimeStatus {
    run_helper(context, |state| {
        let vm = unsafe { state.vm_mut() };
        let frame = vm
            .frames
            .get_mut(frame_index as usize)
            .ok_or(RuntimeError::StackUnderflow)?;
        frame.ip = next_instruction as usize;
        vm.instruction_budget = vm.instruction_budget.wrapping_add(u64::from(refund_count));
        Ok(())
    })
}

pub(super) unsafe extern "C" fn load_register(
    context: *mut c_void,
    base: u32,
    register: u32,
    output: *mut u64,
) -> RuntimeStatus {
    if output.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    run_helper(context, |state| {
        let index = (base as usize)
            .checked_add(register as usize)
            .ok_or_else(|| RuntimeError::out_of_bounds("register index overflow"))?;
        let value = unsafe { state.vm_mut() }
            .stack
            .get(index)
            .copied()
            .ok_or_else(|| RuntimeError::out_of_bounds(format!("register {index}")))?;
        unsafe { output.write(value.to_abi_bits()) };
        Ok(())
    })
}

pub(super) unsafe extern "C" fn store_register(
    context: *mut c_void,
    base: u32,
    register: u32,
    bits: u64,
) -> RuntimeStatus {
    run_helper(context, |state| {
        let value = Value::from_abi_bits(bits).ok_or(RuntimeError::InvalidOperand)?;
        let index = (base as usize)
            .checked_add(register as usize)
            .ok_or_else(|| RuntimeError::out_of_bounds("register index overflow"))?;
        let slot = unsafe { state.vm_mut() }
            .stack
            .get_mut(index)
            .ok_or_else(|| RuntimeError::out_of_bounds(format!("register {index}")))?;
        *slot = value;
        Ok(())
    })
}

pub(super) unsafe extern "C" fn poll(
    context: *mut c_void,
    frame_index: u32,
    instruction_index: u32,
) -> RuntimeStatus {
    run_helper(context, |state| {
        state.gc_countdown -= 1;
        let (stress_mode, heap_limit_exceeded) = {
            let vm = unsafe { state.vm_mut() };
            (vm.stress_mode, vm.heap.heap_limit_exceeded)
        };
        if state.gc_countdown == 0 || heap_limit_exceeded {
            state.gc_countdown = if stress_mode { 1 } else { GC_CHECK_INTERVAL };
            unsafe { state.vm_mut() }.poll_gc()?;
        }

        let vm = unsafe { state.vm_mut() };
        let frame = vm
            .frames
            .get_mut(frame_index as usize)
            .ok_or(RuntimeError::StackUnderflow)?;
        frame.ip = instruction_index
            .checked_add(1)
            .ok_or(RuntimeError::InvalidOperand)? as usize;

        if vm.instruction_budget == 0 {
            return Err(RuntimeError::InstructionBudgetExhausted);
        }
        vm.instruction_budget = vm.instruction_budget.wrapping_sub(1);
        Ok(())
    })
}

pub(super) unsafe extern "C" fn raise_error(
    context: *mut c_void,
    error_code: u32,
) -> RuntimeStatus {
    run_helper(context, |_| {
        Err(match error_code {
            ERROR_INVALID_OPERAND => RuntimeError::InvalidOperand,
            ERROR_DIVISION_BY_ZERO => RuntimeError::DivisionByZero,
            ERROR_INTEGER_OVERFLOW => RuntimeError::IntegerOverflow,
            ERROR_ASSERTION_FAILED => RuntimeError::AssertionFailed,
            ERROR_STACK_OVERFLOW => RuntimeError::StackOverflow,
            _ => RuntimeError::InvalidOperand,
        })
    })
}

pub(super) unsafe extern "C" fn interpreter_bailout(
    context: *mut c_void,
    frame_index: u32,
    instruction_index: u32,
    output: *mut u64,
) -> RuntimeStatus {
    if output.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    run_status_helper(context, |state| {
        state.bailout_ip = Some(instruction_index);
        let vm = unsafe { state.vm_mut() };
        let frame = vm
            .frames
            .get_mut(frame_index as usize)
            .ok_or(RuntimeError::StackUnderflow)?;
        frame.ip = instruction_index as usize;
        vm.interpret()?;
        unsafe { output.write(vm.last_result.to_abi_bits()) };
        Ok(STATUS_INTERPRETER_COMPLETED)
    })
}
