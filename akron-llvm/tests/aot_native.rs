#![cfg(feature = "aot")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode};
use akron_llvm::{AotCompiler, AotOptions, LlvmTierOptions};
use memory::field::PrimeId;
use memory::{Function, Value};

fn arithmetic_program() -> CompiledProgram {
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
                encode_abc(OpCode::Mul.as_u8(), 2, 0, 1),
                encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
            ],
            constants: vec![Value::int(6), Value::int(7)],
            upvalue_info: Vec::new(),
            line_info: vec![1, 1, 1, 1],
        },
        HashMap::new(),
    )
}

fn runtime_archive() -> PathBuf {
    if let Some(path) = std::env::var_os("AKRON_AOT_RUNTIME_ARCHIVE") {
        return PathBuf::from(path);
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/debug/libakron_aot_runtime.a")
}

#[test]
fn aot_builds_and_executes_native_artifact() {
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("answer");
    let runtime = runtime_archive();
    assert!(
        runtime.is_file(),
        "build the AOT runtime first: cargo build -p akron-aot-runtime"
    );
    let options = AotOptions::new("clang", runtime);

    let artifact = AotCompiler::new(options)
        .build_executable(&arithmetic_program(), &executable)
        .unwrap();

    assert_eq!(artifact.executable, executable);
    #[cfg(target_os = "linux")]
    {
        assert_eq!(&std::fs::read(&executable).unwrap()[..4], b"\x7fELF");
        let dependencies = Command::new("ldd").arg(&executable).output().unwrap();
        assert!(dependencies.status.success());
        assert!(!String::from_utf8_lossy(&dependencies.stdout).contains("libLLVM"));
    }
    let output = Command::new(&executable).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Exit Status: 42\n");

    let repeated = Command::new(&executable)
        .env("AKRON_AOT_REPEAT", "3")
        .env("AKRON_ENGINE_TRACE", "1")
        .output()
        .unwrap();
    assert!(repeated.status.success());
    assert_eq!(
        String::from_utf8_lossy(&repeated.stderr)
            .matches("LLVM AOT sample:")
            .count(),
        3
    );
    assert_eq!(
        String::from_utf8_lossy(&repeated.stdout),
        "Exit Status: 42\n"
    );

    let invalid_repeat = Command::new(&executable)
        .env("AKRON_AOT_REPEAT", "0")
        .output()
        .unwrap();
    assert!(!invalid_repeat.status.success());
    assert!(String::from_utf8_lossy(&invalid_repeat.stderr).contains("between 1 and 10000"));

    let tier1_executable = directory.path().join("answer-tier1");
    let mut tier1_options = AotOptions::new("clang", runtime_archive());
    tier1_options.tier2 = LlvmTierOptions::tier1();
    AotCompiler::new(tier1_options)
        .build_executable(&arithmetic_program(), &tier1_executable)
        .unwrap();
    let tier1_output = Command::new(&tier1_executable).output().unwrap();
    assert!(tier1_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&tier1_output.stdout),
        "Exit Status: 42\n"
    );
}
