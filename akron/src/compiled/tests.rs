use std::mem::size_of;
use std::ptr;

use memory::Value;

use crate::{CallFrame, RuntimeError, VM};

use super::{
    runtime_api, RuntimeCapabilities, RuntimeContext, ERROR_DIVISION_BY_ZERO, RUNTIME_ABI_VERSION,
    STATUS_INVALID_ARGUMENT, STATUS_OK, STATUS_RUNTIME_ERROR,
};
use crate::opcode::instruction::{encode_abc, encode_abx};
use crate::{CompiledProgram, OpCode};
use memory::field::PrimeId;
use memory::Function;
use std::collections::HashMap;

#[test]
fn runtime_api_header_is_self_describing() {
    let api = runtime_api();
    assert_eq!(api.magic, *b"AKRTABI\0");
    assert_eq!(api.abi_version, RUNTIME_ABI_VERSION);
    assert_eq!(api.struct_size as usize, size_of::<super::RuntimeApi>());
    assert!(api.capabilities.contains(RuntimeCapabilities::CORE));
    assert!(api
        .validate(
            RUNTIME_ABI_VERSION,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::CORE,
        )
        .is_ok());
    assert!(api
        .validate(
            RUNTIME_ABI_VERSION + 1,
            size_of::<super::RuntimeApi>() as u32,
            RuntimeCapabilities::CORE,
        )
        .is_err());
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
fn poll_updates_location_and_consumes_one_unit_of_fuel() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.instruction_budget = 1;

    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        assert_eq!(unsafe { (runtime_api().poll)(opaque, 0, 9) }, STATUS_OK);
        let status = unsafe { (runtime_api().poll)(opaque, 0, 10) };
        assert_eq!(status, STATUS_RUNTIME_ERROR);
        context.finish(status).unwrap_err()
    };

    assert!(matches!(error, RuntimeError::InstructionBudgetExhausted));
    assert_eq!(vm.frames[0].ip, 11);
    assert_eq!(vm.instruction_budget, 0);
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
        assert_eq!(status, STATUS_OK);
        bailout_ip = context.bailout_ip();
        context.finish(status).unwrap();
    }

    assert_eq!(bailout_ip, Some(1));
    assert_eq!(vm.instruction_budget, 0);
    assert!(vm.frames.is_empty());
    assert_eq!(vm.last_result.as_int(), Some(7));
    assert_eq!(Value::from_abi_bits(output), Some(Value::int(7)));
}
