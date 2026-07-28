use std::collections::HashMap;

use memory::field::PrimeId;
use memory::{Function, Value};

use crate::opcode::instruction::encode_abc;
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
fn vm_load_program_materializes_main() {
    let mut vm = VM::new();
    vm.load_program(minimal_program()).unwrap();
    assert_eq!(vm.frames.len(), 1);
    vm.interpret().unwrap();
}
