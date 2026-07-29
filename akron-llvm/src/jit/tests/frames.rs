use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use memory::field::PrimeId;
use memory::{Function, UpvalueLocation, Value};

use super::{ExecutionOutcome, JitEngine};

fn captured_main_local_program() -> CompiledProgram {
    let add_offset = Function {
        name: "add_offset".to_string(),
        arity: 1,
        max_slots: 2,
        chunk: vec![
            encode_abx(OpCode::GetUpvalue.as_u8(), 1, 0),
            encode_abc(OpCode::Add.as_u8(), 0, 0, 1),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        constants: Vec::new(),
        upvalue_info: vec![1, 0],
        line_info: vec![2; 3],
    };
    let main = Function {
        name: "main".to_string(),
        arity: 0,
        max_slots: 2,
        chunk: vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abx(OpCode::Closure.as_u8(), 1, 0),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
        constants: vec![Value::int(5)],
        upvalue_info: Vec::new(),
        line_info: vec![1; 3],
    };
    CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![add_offset],
        main,
        HashMap::new(),
    )
}

fn captured_location(vm: &VM, closure: Value) -> UpvalueLocation {
    let closure = vm
        .heap
        .get_closure(closure.as_handle().expect("closure handle"))
        .expect("closure object");
    vm.heap
        .get_upvalue(closure.upvalues[0])
        .expect("captured upvalue")
        .location
}

#[test]
fn compiled_main_closes_captured_locals_before_returning() {
    let program = captured_main_local_program();

    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.interpret().unwrap();
    let oracle_closure = oracle.last_result;
    assert!(matches!(
        captured_location(&oracle, oracle_closure),
        UpvalueLocation::Closed(value) if value.as_int() == Some(5)
    ));
    assert_eq!(
        oracle.call_value(oracle_closure, &[Value::int(7)]).unwrap(),
        Value::int(12)
    );

    let engine = JitEngine::compile(&program).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program).unwrap();
    let result = engine.execute(&mut compiled).unwrap();

    assert_eq!(result.outcome, ExecutionOutcome::Native);
    assert!(compiled.frames.is_empty());
    assert!(compiled.open_upvalues.is_none());
    assert!(matches!(
        captured_location(&compiled, result.value),
        UpvalueLocation::Closed(value) if value.as_int() == Some(5)
    ));
    assert_eq!(
        compiled.call_value(result.value, &[Value::int(7)]).unwrap(),
        Value::int(12)
    );
}
