use std::collections::HashMap;

use memory::field::PrimeId;
use memory::{Function, Value};

use crate::compiled::{runtime_api, RuntimeContext, STATUS_NATIVE_CALL_COMPLETE, STATUS_OK};
use crate::opcode::instruction::{encode_abc, encode_abx};
use crate::{CompiledProgram, OpCode, VM};

fn function_program(main: Function, functions: Vec<Function>) -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        functions,
        main,
        HashMap::new(),
    )
}

fn function(name: &str, arity: u8, max_slots: u16, chunk: Vec<u32>) -> Function {
    Function {
        name: name.to_string(),
        arity,
        max_slots,
        chunk,
        constants: Vec::new(),
        upvalue_info: Vec::new(),
        line_info: Vec::new(),
    }
}

#[test]
fn runtime_instruction_reuses_closure_and_global_handlers() {
    let mut vm = VM::new();
    let global_index = vm.globals.len() as u16;
    let main = function(
        "main",
        0,
        2,
        vec![
            encode_abx(OpCode::Closure.as_u8(), 0, 0),
            encode_abx(OpCode::DefGlobalLet.as_u8(), 0, global_index),
            encode_abx(OpCode::GetGlobal.as_u8(), 1, global_index),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
    );
    let prototype = function(
        "identity",
        1,
        1,
        vec![encode_abc(OpCode::Return.as_u8(), 0, 1, 0)],
    );
    vm.load_program(function_program(main, vec![prototype]))
        .unwrap();

    {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        for instruction in 0..3 {
            assert_eq!(
                unsafe { (runtime_api().execute_instruction)(opaque, 0, instruction) },
                STATUS_OK
            );
        }
        context.finish(STATUS_OK).unwrap();
    }

    assert!(vm.stack[0].is_closure());
    assert_eq!(vm.stack[1], vm.stack[0]);
    assert_eq!(vm.globals.len(), global_index as usize + 1);
    assert_eq!(vm.globals[global_index as usize].value, vm.stack[0]);
    assert!(!vm.globals[global_index as usize].mutable);
}

#[test]
fn call_helpers_materialize_and_finish_a_vm_frame() {
    let main = Function {
        constants: vec![Value::int(41)],
        ..function(
            "main",
            0,
            3,
            vec![
                encode_abx(OpCode::Closure.as_u8(), 0, 0),
                encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
                encode_abc(OpCode::Call.as_u8(), 2, 0, 1),
                encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
            ],
        )
    };
    let prototype = function(
        "increment",
        1,
        2,
        vec![encode_abc(OpCode::Return.as_u8(), 0, 1, 0)],
    );
    let mut vm = VM::new();
    vm.load_program(function_program(main, vec![prototype]))
        .unwrap();

    {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        assert_eq!(
            unsafe { (runtime_api().execute_instruction)(opaque, 0, 0) },
            STATUS_OK
        );
        assert_eq!(
            unsafe { (runtime_api().execute_instruction)(opaque, 0, 1) },
            STATUS_OK
        );

        let mut frame_index = u32::MAX;
        let mut base = u32::MAX;
        let mut prototype_index = u32::MAX;
        assert_eq!(
            unsafe {
                (runtime_api().prepare_call)(
                    opaque,
                    0,
                    2,
                    &mut frame_index,
                    &mut base,
                    &mut prototype_index,
                )
            },
            STATUS_OK
        );
        assert_eq!((frame_index, base, prototype_index), (1, 1, 0));
        assert_eq!(
            unsafe {
                (runtime_api().finish_call)(opaque, frame_index, Value::int(42).to_abi_bits())
            },
            STATUS_OK
        );
        context.finish(STATUS_OK).unwrap();
    }

    assert_eq!(vm.frames.len(), 1);
    assert_eq!(vm.stack[2].as_int(), Some(42));
}

#[test]
fn prepare_call_reports_native_completion_without_pushing_a_frame() {
    let main = function(
        "main",
        0,
        1,
        vec![encode_abc(OpCode::Call.as_u8(), 0, 0, 0)],
    );
    let mut vm = VM::new();
    vm.load_program(function_program(main, Vec::new())).unwrap();
    vm.stack[0] = Value::native(0);

    {
        let mut context = RuntimeContext::new(&mut vm);
        let mut frame_index = 0;
        let mut base = 0;
        let mut prototype_index = 0;
        let status = unsafe {
            (runtime_api().prepare_call)(
                context.as_opaque(),
                0,
                0,
                &mut frame_index,
                &mut base,
                &mut prototype_index,
            )
        };
        assert_eq!(status, STATUS_NATIVE_CALL_COMPLETE);
        context.finish(STATUS_OK).unwrap();
    }

    assert_eq!(vm.frames.len(), 1);
}
