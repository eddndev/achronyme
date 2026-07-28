use std::mem::size_of;
use std::ptr;

use memory::Value;

use crate::{RuntimeError, VM};

use super::{
    runtime_api, RuntimeCapabilities, RuntimeContext, ERROR_DIVISION_BY_ZERO, RUNTIME_ABI_V1_SIZE,
    RUNTIME_ABI_V2_SIZE, RUNTIME_ABI_V3_SIZE, RUNTIME_ABI_V4_SIZE, RUNTIME_ABI_VERSION,
    STATUS_INVALID_ARGUMENT, STATUS_OK, STATUS_RUNTIME_ERROR,
};
use crate::opcode::instruction::{encode_abc, encode_abx};
use crate::{CompiledProgram, OpCode};
use memory::field::PrimeId;
use memory::Function;
use std::collections::HashMap;

mod panic_boundary;
mod poll;
mod runtime_calls;
mod specializations;

#[test]
fn runtime_api_header_is_self_describing() {
    let api = runtime_api();
    assert_eq!(api.magic, *b"AKRTABI\0");
    assert_eq!(api.abi_version, 5);
    assert_eq!(api.abi_version, RUNTIME_ABI_VERSION);
    assert_eq!(api.struct_size as usize, size_of::<super::RuntimeApi>());
    assert!(api.capabilities.contains(RuntimeCapabilities::CORE));
    assert!(api.capabilities.contains(RuntimeCapabilities::LLVM_TIER2));
    assert!(api
        .validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::CORE,
        )
        .is_ok());

    let mut baseline_only = *api;
    baseline_only.capabilities = RuntimeCapabilities::LLVM_BASELINE;
    assert!(matches!(
        baseline_only.validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER1,
        ),
        Err(super::RuntimeAbiError::Capabilities { .. })
    ));
    assert!(api
        .validate(
            RUNTIME_ABI_VERSION + 1,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::CORE,
        )
        .is_err());
    assert!(api
        .validate(1, RUNTIME_ABI_V1_SIZE, RuntimeCapabilities::LLVM_BASELINE,)
        .is_ok());
    let mut tier1_only = *api;
    tier1_only.abi_version = 2;
    tier1_only.struct_size = RUNTIME_ABI_V2_SIZE;
    tier1_only.capabilities = RuntimeCapabilities::LLVM_TIER1;
    assert!(tier1_only
        .validate(2, RUNTIME_ABI_V2_SIZE, RuntimeCapabilities::LLVM_TIER1)
        .is_ok());
    assert!(tier1_only
        .validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER2,
        )
        .is_err());
    let mut fast_poll_only = *api;
    fast_poll_only.abi_version = 3;
    fast_poll_only.struct_size = RUNTIME_ABI_V3_SIZE;
    fast_poll_only.capabilities = RuntimeCapabilities::LLVM_TIER1 | RuntimeCapabilities::FAST_POLL;
    assert!(fast_poll_only
        .validate(
            3,
            RUNTIME_ABI_V3_SIZE,
            RuntimeCapabilities::LLVM_TIER1 | RuntimeCapabilities::FAST_POLL,
        )
        .is_ok());
    assert!(fast_poll_only
        .validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER2,
        )
        .is_err());
    let mut known_calls_only = *api;
    known_calls_only.abi_version = 4;
    known_calls_only.struct_size = RUNTIME_ABI_V4_SIZE;
    known_calls_only.capabilities = RuntimeCapabilities::LLVM_TIER1
        | RuntimeCapabilities::FAST_POLL
        | RuntimeCapabilities::KNOWN_CALLS;
    assert!(known_calls_only
        .validate(
            4,
            RUNTIME_ABI_V4_SIZE,
            RuntimeCapabilities::LLVM_TIER1
                | RuntimeCapabilities::FAST_POLL
                | RuntimeCapabilities::KNOWN_CALLS,
        )
        .is_ok());
    assert!(known_calls_only
        .validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER2,
        )
        .is_err());
    assert!(api
        .validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER2,
        )
        .is_ok());
}

#[test]
fn register_window_returns_a_stable_checked_stack_pointer() {
    let mut vm = VM::new();
    let expected = unsafe { vm.stack.as_mut_ptr().add(4).cast::<u64>() };
    let value = Value::int(29);

    {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        let mut first = ptr::null_mut();
        let mut second = ptr::null_mut();

        assert_eq!(
            unsafe { (runtime_api().register_window)(opaque, 4, 3, &mut first) },
            STATUS_OK
        );
        assert_eq!(
            unsafe { (runtime_api().register_window)(opaque, 4, 3, &mut second) },
            STATUS_OK
        );
        assert_eq!(first, expected);
        assert_eq!(second, first);
        unsafe { first.add(1).write(value.to_abi_bits()) };
        context.finish(STATUS_OK).unwrap();
    }

    assert_eq!(vm.stack[5], value);
}

#[test]
fn execution_window_returns_stable_registers_and_global_entries() {
    let mut vm = VM::new();
    let expected_registers = unsafe { vm.stack.as_mut_ptr().add(4).cast::<u64>() };
    let expected_globals = vm.globals.as_mut_ptr().cast();
    let expected_global_count = vm.globals.len() as u32;

    let mut context = RuntimeContext::new(&mut vm);
    let mut registers = ptr::null_mut();
    let mut globals = ptr::null_mut();
    let mut global_count = 0;
    assert_eq!(
        unsafe {
            (runtime_api().execution_window)(
                context.as_opaque(),
                4,
                3,
                &mut registers,
                &mut globals,
                &mut global_count,
            )
        },
        STATUS_OK
    );

    assert_eq!(registers, expected_registers);
    assert_eq!(globals, expected_globals);
    assert_eq!(global_count, expected_global_count);
}

#[test]
fn register_window_rejects_null_and_out_of_bounds_requests() {
    let mut vm = VM::new();
    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let mut output = ptr::null_mut();
        let status = unsafe {
            (runtime_api().register_window)(context.as_opaque(), u32::MAX, 2, &mut output)
        };
        assert_eq!(status, STATUS_RUNTIME_ERROR);
        context.finish(status).unwrap_err()
    };
    assert!(matches!(error, RuntimeError::OutOfBounds(_)));

    let mut vm = VM::new();
    let mut context = RuntimeContext::new(&mut vm);
    assert_eq!(
        unsafe { (runtime_api().register_window)(context.as_opaque(), 0, 1, ptr::null_mut()) },
        STATUS_INVALID_ARGUMENT
    );
}

#[test]
fn register_helpers_round_trip_canonical_values() {
    let mut vm = VM::new();
    let value = Value::int(-17);
    let status;
    let loaded;
    {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        status = unsafe { (runtime_api().store_register)(opaque, 4, 3, value.to_abi_bits()) };
        let mut bits = 0;
        assert_eq!(
            unsafe { (runtime_api().load_register)(opaque, 4, 3, &mut bits) },
            STATUS_OK
        );
        loaded = Value::from_abi_bits(bits).unwrap();
        context.finish(status).unwrap();
    }

    assert_eq!(status, STATUS_OK);
    assert_eq!(loaded, value);
    assert_eq!(vm.stack[7], value);
}

#[test]
fn register_helpers_reject_invalid_input() {
    let mut vm = VM::new();
    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let status =
            unsafe { (runtime_api().store_register)(context.as_opaque(), 0, 0, (1u64 << 60) | 1) };
        assert_eq!(status, STATUS_RUNTIME_ERROR);
        context.finish(status).unwrap_err()
    };
    assert!(matches!(error, RuntimeError::InvalidOperand));

    let mut bits = 0;
    assert_eq!(
        unsafe { (runtime_api().load_register)(ptr::null_mut(), 0, 0, &mut bits) },
        STATUS_INVALID_ARGUMENT
    );
}

#[test]
fn raise_error_preserves_runtime_error_kind() {
    let mut vm = VM::new();
    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let status =
            unsafe { (runtime_api().raise_error)(context.as_opaque(), ERROR_DIVISION_BY_ZERO) };
        assert_eq!(status, STATUS_RUNTIME_ERROR);
        context.finish(status).unwrap_err()
    };
    assert!(matches!(error, RuntimeError::DivisionByZero));
}

#[test]
fn interpreter_bailout_resumes_at_exact_instruction_without_double_fuel() {
    let program = CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 1,
            chunk: vec![
                encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
                encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
            ],
            constants: vec![Value::int(7)],
            upvalue_info: Vec::new(),
            line_info: vec![1, 2],
        },
        HashMap::new(),
    );
    let mut vm = VM::new();
    vm.load_program(program).unwrap();
    vm.stack[0] = Value::int(7);
    vm.instruction_budget = 1;

    let bailout_ip;
    let mut output = 0;
    {
        let mut context = RuntimeContext::new(&mut vm);
        let status =
            unsafe { (runtime_api().interpreter_bailout)(context.as_opaque(), 0, 1, &mut output) };
        assert_eq!(status, super::STATUS_INTERPRETER_COMPLETED);
        bailout_ip = context.bailout_ip();
        context.finish(status).unwrap();
    }

    assert_eq!(bailout_ip, Some(1));
    assert_eq!(vm.instruction_budget, 0);
    assert!(vm.frames.is_empty());
    assert_eq!(vm.last_result.as_int(), Some(7));
    assert_eq!(Value::from_abi_bits(output), Some(Value::int(7)));
}
