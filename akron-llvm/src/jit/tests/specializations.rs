use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::{ExecutionOutcome, JitEngine};

fn list_specialization_program() -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        vec!["push".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 6,
            chunk: vec![
                encode_abc(OpCode::BuildList.as_u8(), 0, 0, 0),
                encode_abc(OpCode::Move.as_u8(), 2, 0, 0),
                encode_abx(OpCode::LoadConst.as_u8(), 3, 1),
                encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
                encode_abc(OpCode::MethodCall.as_u8(), 1, 2, 1),
                encode_abx(OpCode::LoadConst.as_u8(), 4, 2),
                encode_abc(OpCode::GetIndex.as_u8(), 5, 0, 4),
                encode_abc(OpCode::Return.as_u8(), 5, 1, 0),
            ],
            constants: vec![Value::string(0), Value::int(7), Value::int(0)],
            upvalue_info: Vec::new(),
            line_info: vec![1; 8],
        },
        HashMap::new(),
    )
}

fn invalid_push_receiver_program() -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        vec!["push".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 4,
            chunk: vec![
                encode_abx(OpCode::LoadConst.as_u8(), 2, 1),
                encode_abx(OpCode::LoadConst.as_u8(), 3, 2),
                encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
                encode_abc(OpCode::MethodCall.as_u8(), 1, 2, 1),
                encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
            ],
            constants: vec![Value::string(0), Value::int(4), Value::int(7)],
            upvalue_info: Vec::new(),
            line_info: vec![20, 20, 77, 77, 78],
        },
        HashMap::new(),
    )
}

fn invalid_list_index_program(index: i64) -> CompiledProgram {
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
            max_slots: 3,
            chunk: vec![
                encode_abc(OpCode::BuildList.as_u8(), 0, 0, 0),
                encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
                encode_abc(OpCode::GetIndex.as_u8(), 2, 0, 1),
                encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
            ],
            constants: vec![Value::int(index)],
            upvalue_info: Vec::new(),
            line_info: vec![40, 41, 77, 78],
        },
        HashMap::new(),
    )
}

fn mixed_list_and_map_index_loop_program() -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        vec!["key".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 11,
            chunk: vec![
                encode_abx(OpCode::LoadConst.as_u8(), 1, 1),
                encode_abc(OpCode::BuildList.as_u8(), 0, 1, 1),
                encode_abx(OpCode::LoadConst.as_u8(), 2, 0),
                encode_abx(OpCode::LoadConst.as_u8(), 3, 2),
                encode_abc(OpCode::BuildMap.as_u8(), 4, 2, 1),
                encode_abx(OpCode::LoadConst.as_u8(), 5, 3),
                encode_abx(OpCode::LoadConst.as_u8(), 6, 4),
                encode_abx(OpCode::LoadConst.as_u8(), 7, 5),
                encode_abc(OpCode::GetIndex.as_u8(), 8, 0, 5),
                encode_abc(OpCode::GetIndex.as_u8(), 9, 4, 2),
                encode_abc(OpCode::Sub.as_u8(), 6, 6, 7),
                encode_abc(OpCode::Gt.as_u8(), 10, 6, 5),
                encode_abx(OpCode::JumpIfFalse.as_u8(), 10, 14),
                encode_abx(OpCode::Jump.as_u8(), 0, 8),
                encode_abc(OpCode::Return.as_u8(), 9, 1, 0),
            ],
            constants: vec![
                Value::string(0),
                Value::int(7),
                Value::int(9),
                Value::int(0),
                Value::int(4),
                Value::int(1),
            ],
            upvalue_info: Vec::new(),
            line_info: vec![1; 15],
        },
        HashMap::new(),
    )
}

#[test]
fn guarded_list_operations_replace_three_generic_runtime_boundaries() {
    let program = list_specialization_program();
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.interpret().unwrap();

    let engine = JitEngine::compile(&program).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program).unwrap();
    let result = engine.execute(&mut compiled).unwrap();

    assert_eq!(result.value, oracle.last_result);
    assert_eq!(result.value.as_int(), Some(7));
    assert_eq!(result.outcome, ExecutionOutcome::Native);
    assert_eq!(result.stats.specialization_hits, 3);
    assert_eq!(result.stats.specialization_misses, 0);
    assert_eq!(result.stats.runtime_calls, 1);
    assert_eq!(result.stats.interpreter_fallbacks, 0);
}

#[test]
fn list_push_miss_replays_the_canonical_preamble_and_method_once() {
    let program = invalid_push_receiver_program();
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.instruction_budget = 4;
    let oracle_error = oracle.interpret().unwrap_err().to_string();

    let engine = JitEngine::compile(&program).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program).unwrap();
    compiled.instruction_budget = 4;
    let compiled_error = engine.execute(&mut compiled).unwrap_err().to_string();

    assert_eq!(compiled_error, oracle_error);
    assert!(compiled_error.contains("Int has no method 'push'"));
    assert_eq!(compiled.instruction_budget, oracle.instruction_budget);
    assert_eq!(compiled.last_error_location, oracle.last_error_location);
    assert_eq!(compiled.last_error_location, Some(("main".to_string(), 77)));
    assert_eq!(compiled.stack, oracle.stack);
}

#[test]
fn list_index_errors_match_the_interpreter_without_fallback() {
    for index in [-1, 0] {
        let program = invalid_list_index_program(index);
        let mut oracle = VM::new();
        oracle.load_program(program.clone()).unwrap();
        oracle.instruction_budget = 3;
        let oracle_error = oracle.interpret().unwrap_err().to_string();

        let engine = JitEngine::compile(&program).unwrap();
        let mut compiled = VM::new();
        compiled.load_program(program).unwrap();
        compiled.instruction_budget = 3;
        let compiled_error = engine.execute(&mut compiled).unwrap_err().to_string();

        assert_eq!(compiled_error, oracle_error, "index {index}");
        assert_eq!(
            compiled.instruction_budget, oracle.instruction_budget,
            "index {index}"
        );
        assert_eq!(
            compiled.last_error_location, oracle.last_error_location,
            "index {index}"
        );
        assert_eq!(compiled.last_error_location, Some(("main".to_string(), 77)));
        assert_eq!(compiled.stack, oracle.stack, "index {index}");
    }
}

#[test]
fn mixed_index_loop_counts_list_hits_and_canonical_map_misses() {
    let program = mixed_list_and_map_index_loop_program();
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.interpret().unwrap();

    let engine = JitEngine::compile(&program).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program).unwrap();
    let result = engine.execute(&mut compiled).unwrap();

    assert_eq!(result.value, oracle.last_result);
    assert_eq!(result.value.as_int(), Some(9));
    assert_eq!(compiled.stack, oracle.stack);
    assert_eq!(result.outcome, ExecutionOutcome::Native);
    assert_eq!(result.stats.specialization_hits, 4);
    assert_eq!(result.stats.specialization_misses, 4);
    assert_eq!(result.stats.interpreter_fallbacks, 0);
}
