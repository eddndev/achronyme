use std::collections::HashMap;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::lower_program;

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
    assert_eq!(module.ir.matches("call i32 %register_window_fn").count(), 1);
    assert_eq!(module.ir.matches("call i32 %poll_block_fn").count(), 1);
    assert!(!module.ir.contains("call i32 %load_fn"));
    assert!(!module.ir.contains("call i32 %store_fn"));
    assert!(module.ir.contains("%reg.0 = alloca i64, align 8"));
    assert!(module.ir.contains("llvm.sadd.with.overflow.i64"));
}

#[test]
fn heap_constant_lowers_to_interpreter_bailout() {
    let mut program = scalar_program(
        vec![
            encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        vec![Value::string(0)],
    );
    program.strings.push("not scalar".to_string());

    let module = lower_program(&program).unwrap();
    assert_eq!(module.native_instruction_count, 0);
    assert!(module.ir.contains("call i32 %bailout_fn"));
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
        vec![encode_abx(OpCode::Prove.as_u8(), 0, 0)],
        vec![Value::string(0)],
    );
    prove_program.strings.push("prove source".to_string());
    let prove_module = lower_program(&prove_program).unwrap();
    assert_eq!(prove_module.native_instruction_count, 0);
    assert_eq!(prove_module.ir.matches("call i32 %bailout_fn").count(), 1);

    let circom_program = scalar_program(
        vec![encode_abc(OpCode::CallCircomTemplate.as_u8(), 0, 1, 0)],
        Vec::new(),
    );
    let circom_module = lower_program(&circom_program).unwrap();
    assert_eq!(circom_module.native_instruction_count, 0);
    assert_eq!(circom_module.ir.matches("call i32 %bailout_fn").count(), 1);
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
