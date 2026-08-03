use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::{lower_program, lower_program_with_options};
use crate::LlvmTierOptions;

fn scalar_program(chunk: Vec<u32>, constants: Vec<Value>) -> CompiledProgram {
    CompiledProgram::new(
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
            max_slots: 4,
            chunk,
            constants,
            upvalue_info: Vec::new(),
            line_info: Vec::new(),
        },
        HashMap::new(),
    )
}

fn compiled_call_program() -> CompiledProgram {
    let main = Function {
        name: "main".to_string(),
        arity: 0,
        max_slots: 3,
        chunk: vec![
            encode_abx(OpCode::Closure.as_u8(), 0, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
            encode_abc(OpCode::Call.as_u8(), 2, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
        ],
        constants: vec![Value::int(41)],
        upvalue_info: Vec::new(),
        line_info: Vec::new(),
    };
    let increment = Function {
        name: "increment".to_string(),
        arity: 1,
        max_slots: 2,
        chunk: vec![
            encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
            encode_abc(OpCode::Add.as_u8(), 0, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        ],
        constants: vec![Value::int(1)],
        upvalue_info: Vec::new(),
        line_info: Vec::new(),
    };
    CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![increment],
        main,
        HashMap::new(),
    )
}

fn specialized_list_program() -> CompiledProgram {
    let mut program = scalar_program(
        vec![
            encode_abc(OpCode::BuildList.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Move.as_u8(), 2, 0, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 3, 1),
            encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
            encode_abc(OpCode::MethodCall.as_u8(), 1, 2, 1),
            encode_abx(OpCode::LoadConst.as_u8(), 3, 2),
            encode_abc(OpCode::GetIndex.as_u8(), 1, 0, 3),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
        vec![Value::string(0), Value::int(7), Value::int(0)],
    );
    program.strings.push("push".to_string());
    program
}

#[test]
fn scalar_arithmetic_lowers_to_a_native_entry() {
    let program = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 1, 1),
            encode_abc(OpCode::Add.as_u8(), 2, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
        ],
        vec![Value::int(40), Value::int(2)],
    );

    let module = lower_program(&program).unwrap();
    assert_eq!(module.entry_symbol, "akron_compiled_main");
    assert!(module.ir.contains("define i32 @akron_compiled_main"));
    assert_eq!(
        module.ir.matches("call i32 %execution_window_fn").count(),
        1
    );
    assert_eq!(module.ir.matches("call i32 %poll_fast_block_fn").count(), 1);
    assert_eq!(
        module.ir.matches("call i32 %poll_tier1_block_fn").count(),
        1
    );
    assert!(!module.ir.contains("call i32 %load_fn"));
    assert!(!module.ir.contains("call i32 %store_fn"));
    assert!(module.ir.contains("%reg.0 = alloca i64, align 8"));
    assert!(module.ir.contains("llvm.sadd.with.overflow.i64"));
}

#[test]
fn heap_constant_lowers_to_a_runtime_instruction() {
    let mut program = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        vec![Value::string(0)],
    );
    program.strings.push("not scalar".to_string());

    let module = lower_program(&program).unwrap();
    assert_eq!(module.native_instruction_count, 2);
    assert_eq!(module.runtime_instruction_count, 1);
    assert!(module.ir.contains("call i32 %execute_instruction_fn"));
}

#[test]
fn heap_equality_lowers_to_a_runtime_instruction() {
    let mut program = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 1, 1),
            encode_abc(OpCode::Eq.as_u8(), 2, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
        ],
        vec![Value::string(0), Value::string(1)],
    );
    program.strings = vec!["same".to_string(), "same".to_string()];

    let module = lower_program(&program).unwrap();

    assert_eq!(module.direct_instruction_count, 1);
    assert_eq!(module.runtime_instruction_count, 3);
}

#[test]
fn global_operations_lower_through_the_checked_global_window() {
    let program = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abx(OpCode::DefGlobalVar.as_u8(), 0, 15),
            encode_abx(OpCode::GetGlobal.as_u8(), 1, 15),
            encode_abx(OpCode::SetGlobal.as_u8(), 1, 15),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
        vec![Value::int(42)],
    );

    let module = lower_program(&program).unwrap();

    assert_eq!(module.direct_instruction_count, 5);
    assert_eq!(module.runtime_instruction_count, 0);
    assert!(module.ir.contains("call i32 %execution_window_fn"));
    assert!(module.ir.contains("getelementptr i8, ptr %globals"));
}

#[test]
fn whole_program_functions_and_calls_are_lowered() {
    let program = compiled_call_program();

    let module = lower_program(&program).unwrap();

    assert_eq!(module.instruction_count, 7);
    assert_eq!(module.native_instruction_count, 7);
    assert_eq!(module.direct_instruction_count, 5);
    assert_eq!(module.runtime_instruction_count, 1);
    assert_eq!(module.compiled_call_count, 1);
    assert!(module.ir.contains("define i32 @akron_compiled_fn_0"));
    assert!(module.ir.contains("call i32 %prepare_known_call_fn"));
    assert!(module.ir.contains("call i32 %prepare_call_fn"));
    assert!(module.ir.contains("call i32 @akron_compiled_fn_0"));
}

#[test]
fn uninitialized_arithmetic_operand_lowers_to_bailout() {
    let program = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abc(OpCode::Add.as_u8(), 2, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
        ],
        vec![Value::int(1)],
    );

    let module = lower_program(&program).unwrap();
    assert_eq!(module.native_instruction_count, 1);
    assert!(module.ir.contains("i32 1, ptr %result_out"));
}

#[test]
fn prove_and_circom_opcodes_lower_to_visible_bailouts() {
    let mut prove_program = scalar_program(
        vec![
            encode_abx(OpCode::Prove.as_u8(), 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        vec![Value::string(0)],
    );
    prove_program.strings.push("prove source".to_string());
    let prove_module = lower_program(&prove_program).unwrap();
    assert_eq!(prove_module.native_instruction_count, 0);
    assert_eq!(prove_module.ir.matches("call i32 %bailout_fn").count(), 2);

    let circom_program = scalar_program(
        vec![
            encode_abc(OpCode::CallCircomTemplate.as_u8(), 0, 1, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        ],
        Vec::new(),
    );
    let circom_module = lower_program(&circom_program).unwrap();
    assert_eq!(circom_module.native_instruction_count, 0);
    assert_eq!(circom_module.ir.matches("call i32 %bailout_fn").count(), 2);
}

#[test]
fn structured_task_operations_bail_out_before_any_compiled_interleaving() {
    let child = Function {
        name: "answer".into(),
        arity: 0,
        max_slots: 1,
        chunk: vec![encode_abc(OpCode::Return.as_u8(), 0, 0, 0)],
        constants: Vec::new(),
        upvalue_info: Vec::new(),
        line_info: vec![1],
    };
    let mut program = scalar_program(
        vec![
            encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
            encode_abx(OpCode::Closure.as_u8(), 0, 0),
            encode_abc(OpCode::Spawn.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Await.as_u8(), 0, 0, 0),
            encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        Vec::new(),
    );
    program.functions.push(child);
    program.capabilities = program.derived_capabilities();

    let module = lower_program(&program).unwrap();
    assert_eq!(module.native_instruction_count, 1);
    assert_eq!(module.ir.matches("call i32 %bailout_fn").count(), 7);
}

#[test]
fn unknown_opcode_is_rejected_before_llvm_emission() {
    let program = scalar_program(vec![encode_abc(4, 0, 0, 0)], Vec::new());
    let error = lower_program(&program).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("main contains unknown opcode 0x04 at ip 0"),
        "{error}"
    );
}

#[test]
fn immediate_list_push_and_list_index_lower_to_guarded_specializations() {
    let program = specialized_list_program();

    let module = lower_program(&program).unwrap();

    assert_eq!(module.runtime_instruction_count, 1);
    assert!(module.ir.contains("call i32 %list_push_fn"));
    assert!(module.ir.contains("call i32 %list_index_fn"));
    assert!(module.ir.contains("specialization.push.miss"));
    assert!(module.ir.contains("specialization.index.miss"));
}

#[test]
fn tier2_capabilities_can_be_disabled_independently_in_lowering() {
    let arithmetic = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        vec![Value::int(42)],
    );
    let no_fast_poll = lower_program_with_options(
        &arithmetic,
        LlvmTierOptions {
            fast_poll: false,
            ..LlvmTierOptions::default()
        },
    )
    .unwrap();
    assert!(!no_fast_poll.ir.contains("call i32 %poll_fast_block_fn"));
    assert!(no_fast_poll.ir.contains("call i32 %poll_tier1_block_fn"));

    let no_known_calls = lower_program_with_options(
        &compiled_call_program(),
        LlvmTierOptions {
            known_calls: false,
            ..LlvmTierOptions::default()
        },
    )
    .unwrap();
    assert!(!no_known_calls
        .ir
        .contains("call i32 %prepare_known_call_fn"));
    assert!(no_known_calls.ir.contains("call i32 %prepare_call_fn"));

    let no_lists = lower_program_with_options(
        &specialized_list_program(),
        LlvmTierOptions {
            list_specializations: false,
            ..LlvmTierOptions::default()
        },
    )
    .unwrap();
    assert!(!no_lists.ir.contains("call i32 %list_push_fn"));
    assert!(!no_lists.ir.contains("call i32 %list_index_fn"));
    assert_eq!(no_lists.runtime_instruction_count, 4);
}
