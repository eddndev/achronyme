use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, ProgramCapabilities, VM};
use memory::field::PrimeId;
use memory::{Function, Value};

use super::{link_command, AotCompiler, AotError, AotOptions};

fn contains_arg(command: &std::process::Command, expected: &str) -> bool {
    command
        .get_args()
        .any(|argument| argument == OsStr::new(expected))
}

fn program(chunk: Vec<u32>, constants: Vec<Value>, strings: Vec<String>) -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        strings,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 3,
            line_info: vec![1; chunk.len()],
            chunk,
            constants,
            upvalue_info: Vec::new(),
        },
        HashMap::new(),
    )
}

fn missing_tools_error(program: &CompiledProgram) -> AotError {
    AotCompiler::new(AotOptions::new("missing-clang", "missing-runtime"))
        .build_executable(program, "unused-output")
        .unwrap_err()
}

#[cfg(target_os = "linux")]
#[test]
fn linker_gc_sections_are_enabled_by_default_and_can_be_disabled() {
    let enabled = AotOptions::new("clang-21", "runtime.a");
    let object = Path::new("program.o");
    let executable = Path::new("program");

    let enabled_link = link_command(&enabled, object, executable);
    assert!(contains_arg(&enabled_link, "-Wl,--gc-sections"));

    let mut disabled = enabled;
    disabled.linker_gc_sections = false;
    let disabled_link = link_command(&disabled, object, executable);
    assert!(!contains_arg(&disabled_link, "-Wl,--gc-sections"));
}

#[test]
fn invalid_program_is_rejected_before_aot_tools() {
    let program = program(
        vec![encode_abx(OpCode::LoadConst.as_u8(), 0, 0)],
        vec![Value::int(1)],
        Vec::new(),
    );

    let error = missing_tools_error(&program);

    assert!(matches!(error, AotError::Program(_)), "{error}");
    assert!(
        error
            .to_string()
            .contains("reachable path exits without Return"),
        "{error}"
    );
}

#[test]
fn unsupported_prove_capability_is_rejected_before_aot_tools() {
    let program = program(
        vec![
            encode_abx(OpCode::Prove.as_u8(), 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        ],
        vec![Value::string(0)],
        vec!["prove program".to_string()],
    );

    let error = missing_tools_error(&program);

    assert!(
        matches!(error, AotError::UnsupportedCapabilities(_)),
        "{error}"
    );
    assert!(error.to_string().contains("PROVE"), "{error}");
}

#[test]
fn unsupported_circom_capability_is_rejected_before_aot_tools() {
    let program = program(
        vec![
            encode_abc(OpCode::CallCircomTemplate.as_u8(), 0, 1, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        ],
        Vec::new(),
        Vec::new(),
    );

    let error = missing_tools_error(&program);

    assert!(
        matches!(error, AotError::UnsupportedCapabilities(_)),
        "{error}"
    );
    assert!(error.to_string().contains("CIRCOM"), "{error}");
}

#[test]
fn unsupported_verify_capability_is_rejected_before_aot_tools() {
    let verify_global = VM::new()
        .natives
        .iter()
        .position(|native| native.name == "verify_proof")
        .unwrap() as u16;
    let program = program(
        vec![
            encode_abx(OpCode::GetGlobal.as_u8(), 0, verify_global),
            encode_abc(OpCode::Call.as_u8(), 1, 0, 0),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
        Vec::new(),
        Vec::new(),
    );

    let error = missing_tools_error(&program);

    assert!(
        matches!(error, AotError::UnsupportedCapabilities(_)),
        "{error}"
    );
    assert!(error.to_string().contains("VERIFY"), "{error}");
}

#[test]
fn hosted_native_and_file_io_capabilities_reach_tool_discovery() {
    let mut program = program(
        vec![
            encode_abc(OpCode::Call.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        ],
        Vec::new(),
        Vec::new(),
    );
    program.capabilities |= ProgramCapabilities::FILE_IO;

    let error = missing_tools_error(&program);

    assert!(matches!(error, AotError::Configuration(_)), "{error}");
    assert!(error.to_string().contains("missing-runtime"), "{error}");
}
