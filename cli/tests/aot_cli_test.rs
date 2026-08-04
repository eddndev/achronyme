#![cfg(feature = "llvm")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use akron::opcode::instruction::encode_abc;
use akron::{CompiledProgram, OpCode};
use memory::field::PrimeId;
use memory::Function;

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn runtime_archive() -> PathBuf {
    workspace().join("target/debug/libakron_aot_runtime.a")
}

fn circom_program() -> CompiledProgram {
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
            max_slots: 1,
            chunk: vec![
                encode_abc(OpCode::CallCircomTemplate.as_u8(), 0, 1, 0),
                encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
            ],
            constants: Vec::new(),
            upvalue_info: Vec::new(),
            line_info: vec![1; 2],
        },
        HashMap::new(),
    )
}

#[test]
fn aot_command_builds_executable_from_source() {
    let runtime = runtime_archive();
    assert!(
        runtime.is_file(),
        "build the runtime first with cargo build -p akron-aot-runtime"
    );
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("answer.ach");
    let executable = directory.path().join("answer");
    std::fs::write(&source, "return 6 * 7\n").unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("aot")
        .arg(&source)
        .arg("--output")
        .arg(&executable)
        .arg("--runtime")
        .arg(&runtime)
        .output()
        .expect("run ach aot");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let output = Command::new(&executable).output().expect("run AOT program");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Exit Status: 42\n");

    let limited = Command::new(&executable)
        .env("AKRON_INSTRUCTION_BUDGET", "2")
        .output()
        .expect("run limited AOT program");
    assert!(!limited.status.success());
    let stderr = String::from_utf8_lossy(&limited.stderr);
    assert!(stderr.contains("instruction budget exhausted"), "{stderr}");
    assert!(stderr.contains("[line 1] in main"), "{stderr}");

    let invalid_pool = Command::new(&executable)
        .env("AKRON_BLOCKING_WORKERS", "0")
        .output()
        .expect("run AOT program with invalid pool configuration");
    assert!(!invalid_pool.status.success());
    let stderr = String::from_utf8_lossy(&invalid_pool.stderr);
    assert!(stderr.contains("blocking_workers must be 1..="), "{stderr}");

    let invalid_pending = Command::new(&executable)
        .env("AKRON_MAX_PENDING_NATIVE_REQUESTS", "4097")
        .output()
        .expect("run AOT program with invalid pending-request limit");
    assert!(!invalid_pending.status.success());
    let stderr = String::from_utf8_lossy(&invalid_pending.stderr);
    assert!(
        stderr.contains("max_pending_native_requests must be 0..="),
        "{stderr}"
    );

    let invalid_retained = Command::new(&executable)
        .env("AKRON_MAX_RETAINED_TASK_RESULTS", "4097")
        .output()
        .expect("run AOT program with invalid retained-result limit");
    assert!(!invalid_retained.status.success());
    let stderr = String::from_utf8_lossy(&invalid_retained.stderr);
    assert!(
        stderr.contains("max_retained_task_results must be 0..="),
        "{stderr}"
    );
}

#[test]
fn aot_bailout_honors_heap_limit() {
    let runtime = runtime_archive();
    assert!(runtime.is_file());
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("heap.ach");
    let executable = directory.path().join("heap");
    std::fs::write(&source, "print([1, 2, 3])\n").unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("aot")
        .arg(&source)
        .arg("--output")
        .arg(&executable)
        .arg("--runtime")
        .arg(&runtime)
        .output()
        .expect("run ach aot");
    assert!(compile.status.success());

    let output = Command::new(&executable)
        .env("AKRON_MAX_HEAP", "1")
        .output()
        .expect("run heap-limited AOT program");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("heap limit exceeded"), "{stderr}");
}

#[test]
fn aot_bailout_runs_the_embedded_structured_scheduler() {
    let runtime = runtime_archive();
    assert!(runtime.is_file());
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("tasks.ach");
    let executable = directory.path().join("tasks");
    std::fs::write(
        &source,
        "fn answer() { 42 }\nreturn concurrent { let child = spawn answer(); await child }\n",
    )
    .unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("aot")
        .arg(&source)
        .arg("--output")
        .arg(&executable)
        .arg("--runtime")
        .arg(&runtime)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let output = Command::new(&executable)
        .env("AKRON_ENGINE_TRACE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Exit Status: 42\n");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LLVM AOT bailout:"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aot_rejects_unhosted_verify_from_source() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("verify.ach");
    let executable = directory.path().join("verify");
    std::fs::write(&source, "return verify_proof(1)\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("aot")
        .arg(&source)
        .arg("--output")
        .arg(&executable)
        .arg("--runtime")
        .arg(runtime_archive())
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!executable.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required capabilities: VERIFY"), "{stderr}");
}

#[test]
fn aot_rejects_unhosted_circom_from_achb() {
    let directory = tempfile::tempdir().unwrap();
    let bytecode = directory.path().join("circom.achb");
    let executable = directory.path().join("circom");
    let mut bytes = Vec::new();
    circom_program().write_executable(&mut bytes).unwrap();
    std::fs::write(&bytecode, bytes).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("aot")
        .arg(&bytecode)
        .arg("--output")
        .arg(&executable)
        .arg("--runtime")
        .arg(runtime_archive())
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!executable.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required capabilities: CIRCOM"), "{stderr}");
}
