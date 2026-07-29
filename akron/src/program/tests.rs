use std::collections::HashMap;

use memory::field::PrimeId;
use memory::{Function, Value};

use crate::opcode::instruction::encode_abc;
use crate::opcode::instruction::encode_abx;
use crate::opcode::OpCode;
use crate::VM;

use super::CompiledProgram;

fn minimal_program() -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        vec!["hello".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 1,
            chunk: vec![encode_abc(OpCode::Return.as_u8(), 0, 0, 0)],
            constants: vec![Value::string(0)],
            upvalue_info: Vec::new(),
            line_info: vec![7],
        },
        HashMap::from([(14, "print".to_string())]),
    )
}

#[test]
fn executable_roundtrip_preserves_program() {
    let program = minimal_program();
    let mut bytes = Vec::new();
    program.write_executable(&mut bytes).unwrap();
    assert_eq!(&bytes[..4], b"ACH\x0C");

    let decoded = CompiledProgram::read_executable(&mut bytes.as_slice()).unwrap();
    assert_eq!(decoded.main.chunk, program.main.chunk);
    assert_eq!(decoded.main.line_info, vec![7]);
    assert_eq!(decoded.strings, vec!["hello"]);
    assert_eq!(decoded.debug_symbols.get(&14), Some(&"print".to_string()));
}

#[test]
fn validation_rejects_unknown_opcode() {
    let mut program = minimal_program();
    program.main.chunk[0] = 0xFE00_0000;
    let error = program.validate().unwrap_err().to_string();
    assert!(error.contains("unknown opcode"), "{error}");
}

#[test]
fn validation_rejects_reachable_function_fallthrough() {
    let fallthrough = Function {
        name: "fallthrough".to_string(),
        arity: 0,
        max_slots: 1,
        chunk: vec![encode_abx(OpCode::LoadConst.as_u8(), 0, 0)],
        constants: vec![Value::int(99)],
        upvalue_info: Vec::new(),
        line_info: vec![2],
    };
    let mut program = minimal_program();
    program.functions.push(fallthrough);
    program.main.max_slots = 3;
    program.main.chunk = vec![
        encode_abx(OpCode::Closure.as_u8(), 0, 0),
        encode_abx(OpCode::LoadConst.as_u8(), 2, 0),
        encode_abc(OpCode::Call.as_u8(), 2, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
    ];
    program.main.constants = vec![Value::int(7)];
    program.main.line_info = vec![1; 4];
    program.capabilities = program.derived_capabilities();

    let error = program.validate().unwrap_err().to_string();

    assert!(error.contains("prototype 0 (fallthrough)"), "{error}");
    assert!(
        error.contains("reachable path exits without Return"),
        "{error}"
    );
}

#[test]
fn validation_rejects_empty_main() {
    let mut program = minimal_program();
    program.main.chunk.clear();
    program.main.line_info.clear();

    let error = program.validate().unwrap_err().to_string();

    assert!(
        error.contains("main reachable path exits without Return"),
        "{error}"
    );
}

#[test]
fn validation_rejects_conditional_fallthrough_edge() {
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abx(OpCode::Jump.as_u8(), 0, 2),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        encode_abx(OpCode::JumpIfFalse.as_u8(), 0, 1),
        encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
    ];
    program.main.line_info = vec![1; 4];

    let error = program.validate().unwrap_err().to_string();

    assert!(
        error.contains("reachable path exits without Return"),
        "{error}"
    );
}

#[test]
fn validation_allows_unreachable_trailing_fallthrough() {
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abx(OpCode::Jump.as_u8(), 0, 1),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
    ];
    program.main.line_info = vec![1; 3];

    program.validate().unwrap();
}

#[test]
fn validation_allows_reachable_cycle_without_exit() {
    let mut program = minimal_program();
    program.main.chunk = vec![encode_abx(OpCode::Jump.as_u8(), 0, 0)];
    program.main.line_info = vec![1];

    program.validate().unwrap();
}

#[test]
fn vm_load_program_materializes_main() {
    let mut vm = VM::new();
    vm.load_program(minimal_program()).unwrap();
    assert_eq!(vm.frames.len(), 1);
    vm.interpret().unwrap();
}

#[test]
fn vm_load_program_preallocates_program_global_slots() {
    let mut program = minimal_program();
    program.main.max_slots = 1;
    program.main.chunk = vec![
        encode_abx(OpCode::DefGlobalVar.as_u8(), 0, 20),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
    ];
    program.main.line_info = vec![7; 2];

    let mut vm = VM::new();
    vm.load_program(program).unwrap();

    assert_eq!(vm.globals.len(), 21);
}

#[test]
fn preallocated_global_is_undefined_until_definition_executes() {
    let mut program = minimal_program();
    program.main.max_slots = 1;
    program.main.chunk = vec![
        encode_abx(OpCode::GetGlobal.as_u8(), 0, 20),
        encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
    ];
    program.main.line_info = vec![7; 2];
    program.debug_symbols.insert(20, "future".to_string());

    let mut vm = VM::new();
    vm.load_program(program).unwrap();
    let error = vm.interpret().unwrap_err().to_string();

    assert!(error.contains("undefined global"), "{error}");
    assert!(error.contains("'future'"), "{error}");
}
