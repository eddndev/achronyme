use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::{ExecutionOutcome, JitEngine};

fn arithmetic_program(operator: OpCode, left: i64, right: i64) -> CompiledProgram {
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
                encode_abx(OpCode::LoadConst.as_u8(), 1, 1),
                encode_abc(operator.as_u8(), 2, 0, 1),
                encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
            ],
            constants: vec![Value::int(left), Value::int(right)],
            upvalue_info: Vec::new(),
            line_info: vec![1, 1, 1, 1],
        },
        HashMap::new(),
    )
}

#[test]
fn orc_jit_executes_lowered_integer_arithmetic() {
    let program = arithmetic_program(OpCode::Add, 40, 2);
    let engine = JitEngine::compile(&program).unwrap();
    assert_eq!(engine.llvm_version().major, 21);
    assert_eq!(engine.optimization_pipeline(), "default<O2>");

    let mut vm = VM::new();
    vm.load_program(program).unwrap();
    let result = engine.execute(&mut vm).unwrap();
    assert_eq!(result.value.as_int(), Some(42));
    assert_eq!(result.outcome, ExecutionOutcome::Native);
    assert!(vm.frames.is_empty());
    assert_eq!(vm.instruction_budget, u64::MAX.wrapping_sub(4));
}

#[test]
fn block_poll_matches_interpreter_at_each_fuel_boundary() {
    for budget in 0..=5 {
        let mut oracle = VM::new();
        oracle
            .load_program(arithmetic_program(OpCode::Add, 40, 2))
            .unwrap();
        oracle.instruction_budget = budget;
        let oracle_result = oracle.interpret().map(|_| oracle.last_result);

        let program = arithmetic_program(OpCode::Add, 40, 2);
        let engine = JitEngine::compile(&program).unwrap();
        let mut compiled = VM::new();
        compiled.load_program(program).unwrap();
        compiled.instruction_budget = budget;
        let compiled_result = engine.execute(&mut compiled).map(|result| result.value);

        assert_eq!(
            compiled_result.as_ref().map_err(ToString::to_string),
            oracle_result.as_ref().map_err(ToString::to_string),
            "result differs at budget {budget}"
        );
        assert_eq!(
            compiled.instruction_budget, oracle.instruction_budget,
            "remaining fuel differs at budget {budget}"
        );
        assert_eq!(
            &compiled.stack[..3],
            &oracle.stack[..3],
            "registers differ at budget {budget}"
        );
        assert_eq!(
            compiled.last_error_location, oracle.last_error_location,
            "error location differs at budget {budget}"
        );
    }
}

#[test]
fn stress_gc_uses_the_exact_interpreter_slow_path() {
    let program = arithmetic_program(OpCode::Add, 40, 2);
    let engine = JitEngine::compile(&program).unwrap();
    let mut vm = VM::new();
    vm.load_program(program).unwrap();
    vm.stress_mode = true;
    vm.instruction_budget = 4;

    let result = engine.execute(&mut vm).unwrap();

    assert_eq!(result.value.as_int(), Some(42));
    assert_eq!(
        result.outcome,
        ExecutionOutcome::InterpreterBailout { instruction: 0 }
    );
    assert_eq!(vm.instruction_budget, 0);
}

#[test]
fn orc_jit_preserves_division_by_zero_error() {
    let program = arithmetic_program(OpCode::Div, 10, 0);
    let engine = JitEngine::compile(&program).unwrap();
    let mut vm = VM::new();
    vm.load_program(program).unwrap();
    vm.instruction_budget = 4;

    let error = engine.execute(&mut vm).unwrap_err().to_string();
    assert!(error.contains("division by zero"), "{error}");
    assert_eq!(vm.last_error_location, Some(("main".to_string(), 1)));
    assert_eq!(vm.instruction_budget, 1);
}

#[test]
fn orc_jit_bails_out_to_interpreter_at_unsupported_opcode() {
    let program = arithmetic_program(OpCode::Pow, 2, 3);
    let engine = JitEngine::compile(&program).unwrap();
    let mut vm = VM::new();
    vm.load_program(program).unwrap();

    let result = engine.execute(&mut vm).unwrap();

    assert_eq!(
        result.outcome,
        ExecutionOutcome::InterpreterBailout { instruction: 2 }
    );
    assert_eq!(result.value.as_int(), Some(8));
    assert_eq!(vm.stack[2].as_int(), Some(8));
    assert!(vm.frames.is_empty());
    assert_eq!(vm.instruction_budget, u64::MAX.wrapping_sub(4));
}
