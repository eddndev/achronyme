use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{self, NonNull};

use memory::Value;

use crate::{RuntimeError, VM};

use super::abi::{
    RuntimeStatus, ERROR_ASSERTION_FAILED, ERROR_DIVISION_BY_ZERO, ERROR_INTEGER_OVERFLOW,
    ERROR_INVALID_OPERAND, ERROR_STACK_OVERFLOW, STATUS_INTERNAL_ERROR, STATUS_INVALID_ARGUMENT,
    STATUS_OK, STATUS_RUNTIME_ERROR,
};

const GC_CHECK_INTERVAL: u32 = 1024;

struct RuntimeContextState {
    vm: NonNull<VM>,
    pending_error: Option<RuntimeError>,
    gc_countdown: u32,
    bailout_ip: Option<u32>,
}

impl RuntimeContextState {
    unsafe fn vm_mut(&mut self) -> &mut VM {
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
        if status == STATUS_OK && self.state.pending_error.is_none() {
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

fn run_helper<F>(context: *mut c_void, operation: F) -> RuntimeStatus
where
    F: FnOnce(&mut RuntimeContextState) -> Result<(), RuntimeError>,
{
    if context.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }

    let state = unsafe { &mut *context.cast::<RuntimeContextState>() };
    match catch_unwind(AssertUnwindSafe(|| operation(state))) {
        Ok(Ok(())) => STATUS_OK,
        Ok(Err(error)) => state.fail(error),
        Err(_) => {
            state.pending_error = Some(RuntimeError::resource_limit_exceeded(
                "compiled runtime helper panicked",
            ));
            STATUS_INTERNAL_ERROR
        }
    }
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
    run_helper(context, |state| {
        state.bailout_ip = Some(instruction_index);
        let vm = unsafe { state.vm_mut() };
        let frame = vm
            .frames
            .get_mut(frame_index as usize)
            .ok_or(RuntimeError::StackUnderflow)?;
        frame.ip = instruction_index as usize;
        vm.interpret()?;
        unsafe { output.write(vm.last_result.to_abi_bits()) };
        Ok(())
    })
}
