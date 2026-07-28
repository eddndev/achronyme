use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::{ExecutionOutcome, JitEngine};

mod globals;

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

fn recursive_fibonacci_program(input: i64) -> CompiledProgram {
    let main = Function {
        name: "main".to_string(),
        arity: 0,
        max_slots: 4,
        chunk: vec![
            encode_abx(OpCode::Closure.as_u8(), 0, 0),
            encode_abc(OpCode::Move.as_u8(), 1, 0, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 2, 0),
            encode_abc(OpCode::Call.as_u8(), 3, 0, 2),
            encode_abc(OpCode::Return.as_u8(), 3, 1, 0),
        ],
        constants: vec![Value::int(input)],
        upvalue_info: Vec::new(),
        line_info: vec![1; 5],
    };
    let fibonacci = Function {
        name: "fibonacci".to_string(),
        arity: 2,
        max_slots: 12,
        chunk: vec![
            encode_abx(OpCode::LoadConst.as_u8(), 2, 0),
            encode_abc(OpCode::Lt.as_u8(), 3, 1, 2),
            encode_abx(OpCode::JumpIfFalse.as_u8(), 3, 4),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 4, 1),
            encode_abc(OpCode::Sub.as_u8(), 5, 1, 4),
            encode_abc(OpCode::Move.as_u8(), 6, 0, 0),
            encode_abc(OpCode::Move.as_u8(), 7, 0, 0),
            encode_abc(OpCode::Move.as_u8(), 8, 5, 0),
            encode_abc(OpCode::Call.as_u8(), 9, 6, 2),
            encode_abc(OpCode::Move.as_u8(), 3, 9, 0),
            encode_abc(OpCode::Sub.as_u8(), 5, 5, 4),
            encode_abc(OpCode::Move.as_u8(), 8, 5, 0),
            encode_abc(OpCode::Call.as_u8(), 10, 6, 2),
            encode_abc(OpCode::Add.as_u8(), 11, 3, 10),
            encode_abc(OpCode::Return.as_u8(), 11, 1, 0),
        ],
        constants: vec![Value::int(2), Value::int(1)],
        upvalue_info: Vec::new(),
        line_info: vec![2; 16],
    };
    CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fibonacci],
        main,
        HashMap::new(),
    )
}

fn nested_bailout_error_program() -> CompiledProgram {
    let main = Function {
        name: "main".to_string(),
        arity: 0,
        max_slots: 3,
        chunk: vec![
            encode_abx(OpCode::Closure.as_u8(), 0, 0),
            encode_abc(OpCode::Call.as_u8(), 1, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
        constants: Vec::new(),
        upvalue_info: Vec::new(),
        line_info: vec![1; 3],
    };
    let failing = Function {
        name: "failing".to_string(),
        arity: 1,
        max_slots: 5,
        chunk: vec![
            encode_abx(OpCode::LoadConst.as_u8(), 1, 0),
            encode_abx(OpCode::LoadConst.as_u8(), 2, 1),
            encode_abc(OpCode::Pow.as_u8(), 3, 1, 1),
            encode_abc(OpCode::Div.as_u8(), 4, 3, 2),
            encode_abc(OpCode::Return.as_u8(), 4, 1, 0),
        ],
        constants: vec![Value::int(2), Value::int(0)],
        upvalue_info: Vec::new(),
        line_info: vec![40, 41, 42, 77, 78],
    };
    CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![failing],
        main,
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
    assert_eq!(result.stats.direct_instructions, 4);
    assert_eq!(result.stats.runtime_calls, 0);
    assert_eq!(result.stats.interpreter_fallbacks, 0);
    assert!(result.stats.block_polls > 0);
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
fn orc_jit_executes_recursive_compiled_functions() {
    let program = recursive_fibonacci_program(10);
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.interpret().unwrap();
    assert_eq!(oracle.last_result.as_int(), Some(55));
    let engine = JitEngine::compile(&program).unwrap();
    assert_eq!(engine.instruction_count(), 21);
    assert_eq!(engine.native_instruction_count(), 21);

    let mut vm = VM::new();
    vm.load_program(program).unwrap();
    let result = engine.execute(&mut vm).unwrap();

    assert_eq!(result.value.as_int(), Some(55));
    assert_eq!(result.outcome, ExecutionOutcome::Native);
    assert!(result.stats.compiled_function_calls > 0);
    assert_eq!(result.stats.interpreter_fallbacks, 0);
    assert!(vm.frames.is_empty());
}

#[test]
fn recursive_calls_match_interpreter_at_low_fuel_boundaries() {
    let program = recursive_fibonacci_program(10);
    let engine = JitEngine::compile(&program).unwrap();

    for budget in 0..=64 {
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
        assert_eq!(
            compiled.instruction_budget, oracle.instruction_budget,
            "remaining fuel differs at budget {budget}"
        );
        assert_eq!(
            compiled.last_error_location, oracle.last_error_location,
            "error location differs at budget {budget}"
        );
        assert_eq!(
            compiled
                .frames
                .iter()
                .map(|frame| (frame.closure, frame.ip, frame.base, frame.dest_reg))
                .collect::<Vec<_>>(),
            oracle
                .frames
                .iter()
                .map(|frame| (frame.closure, frame.ip, frame.base, frame.dest_reg))
                .collect::<Vec<_>>(),
            "frames differ at budget {budget}"
        );
        assert_eq!(
            compiled.stack, oracle.stack,
            "stack differs at budget {budget}"
        );
    }
}

#[test]
fn nested_interpreter_bailout_preserves_error_location_and_fuel() {
    let program = nested_bailout_error_program();
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.instruction_budget = 8;
    let oracle_error = oracle.interpret().unwrap_err().to_string();

    let engine = JitEngine::compile(&program).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program).unwrap();
    compiled.instruction_budget = 8;
    let compiled_error = engine.execute(&mut compiled).unwrap_err().to_string();

    assert!(oracle_error.contains("division by zero"), "{oracle_error}");
    assert!(
        compiled_error.contains("division by zero"),
        "{compiled_error}"
    );
    assert_eq!(compiled.instruction_budget, oracle.instruction_budget);
    assert_eq!(compiled.last_error_location, oracle.last_error_location);
    assert_eq!(
        compiled.last_error_location,
        Some(("failing".to_string(), 77))
    );
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
    assert_eq!(result.stats.interpreter_fallbacks, 1);
    assert_eq!(result.stats.first_fallback_instruction, Some(2));
    assert_eq!(vm.stack[2].as_int(), Some(8));
    assert!(vm.frames.is_empty());
    assert_eq!(vm.instruction_budget, u64::MAX.wrapping_sub(4));
}
