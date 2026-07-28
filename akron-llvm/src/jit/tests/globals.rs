use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::JitEngine;

fn mutable_global_program() -> CompiledProgram {
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
                encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
                encode_abx(OpCode::DefGlobalVar.as_u8(), 0, 15),
                encode_abx(OpCode::GetGlobal.as_u8(), 1, 15),
                encode_abx(OpCode::LoadConst.as_u8(), 2, 1),
                encode_abc(OpCode::Add.as_u8(), 1, 1, 2),
                encode_abx(OpCode::SetGlobal.as_u8(), 1, 15),
                encode_abx(OpCode::GetGlobal.as_u8(), 0, 15),
                encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
            ],
            constants: vec![Value::int(41), Value::int(1)],
            upvalue_info: Vec::new(),
            line_info: vec![1; 8],
        },
        HashMap::from([(15, "answer".to_string())]),
    )
}

fn immutable_global_error_program() -> CompiledProgram {
    let mut program = mutable_global_program();
    program.main.chunk[1] = encode_abx(OpCode::DefGlobalLet.as_u8(), 0, 15);
    program.main.line_info = vec![10, 11, 12, 13, 14, 77, 78, 79];
    program
}

#[test]
fn orc_jit_executes_checked_global_operations_directly() {
    let program = mutable_global_program();
    let engine = JitEngine::compile(&program).unwrap();
    let mut vm = VM::new();
    vm.load_program(program).unwrap();

    let result = engine.execute(&mut vm).unwrap();

    assert_eq!(result.value.as_int(), Some(42));
    assert_eq!(result.stats.direct_instructions, 8);
    assert_eq!(result.stats.runtime_calls, 0);
    assert_eq!(result.stats.interpreter_fallbacks, 0);
}

#[test]
fn direct_globals_match_interpreter_at_each_fuel_boundary() {
    let program = mutable_global_program();
    let engine = JitEngine::compile(&program).unwrap();

    for budget in 0..=9 {
        let mut oracle = VM::new();
        oracle.load_program(program.clone()).unwrap();
        oracle.instruction_budget = budget;
        let oracle_result = oracle.interpret().map(|_| oracle.last_result);

        let mut compiled = VM::new();
        compiled.load_program(program.clone()).unwrap();
        compiled.instruction_budget = budget;
        let compiled_result = engine.execute(&mut compiled).map(|result| result.value);

        assert_eq!(
            compiled_result.as_ref().map_err(ToString::to_string),
            oracle_result.as_ref().map_err(ToString::to_string),
            "result differs at budget {budget}"
        );
        assert_eq!(compiled.instruction_budget, oracle.instruction_budget);
        assert_eq!(compiled.last_error_location, oracle.last_error_location);
        assert_eq!(compiled.stack, oracle.stack);
        assert_eq!(
            compiled
                .globals
                .iter()
                .map(|entry| (entry.value, entry.mutable, entry.defined))
                .collect::<Vec<_>>(),
            oracle
                .globals
                .iter()
                .map(|entry| (entry.value, entry.mutable, entry.defined))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn immutable_global_deoptimizes_to_the_exact_interpreter_error() {
    let program = immutable_global_error_program();
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.instruction_budget = 8;
    let oracle_error = oracle.interpret().unwrap_err().to_string();

    let engine = JitEngine::compile(&program).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program).unwrap();
    compiled.instruction_budget = 8;
    let compiled_error = engine.execute(&mut compiled).unwrap_err().to_string();

    assert!(oracle_error.contains("immutable global"), "{oracle_error}");
    assert!(
        compiled_error.contains("immutable global"),
        "{compiled_error}"
    );
    assert_eq!(compiled.instruction_budget, oracle.instruction_budget);
    assert_eq!(compiled.last_error_location, oracle.last_error_location);
    assert_eq!(compiled.last_error_location, Some(("main".to_string(), 77)));
}
