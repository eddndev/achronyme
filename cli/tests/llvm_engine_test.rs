#![cfg(feature = "llvm")]

use std::path::Path;
use std::process::{Command, Output};

fn write_source(directory: &tempfile::TempDir, name: &str, source: &str) -> std::path::PathBuf {
    let path = directory.path().join(name);
    std::fs::write(&path, source).unwrap();
    path
}

fn run(path: &Path, engine: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ach"))
        .env("AKRON_ENGINE_TRACE", "1")
        .arg("--no-config")
        .arg("run")
        .arg(path)
        .arg("--engine")
        .arg(engine)
        .output()
        .expect("run ach")
}

#[test]
fn explicit_jit_executes_supported_source() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "scalar.ach", "return 40 + 2\n");

    let output = run(&source, "jit");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn explicit_jit_uses_visible_interpreter_bailout_for_unsupported_opcode() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "print.ach", "print(42)\n");

    let output = run(&source, "jit");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("LLVM JIT bailout:"), "{stderr}");
    assert!(output.stdout.starts_with(b"42\n"));
}

#[test]
fn auto_engine_reports_fallback_and_runs_interpreter() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "print.ach", "print(42)\n");

    let output = run(&source, "auto");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("42\n"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("LLVM JIT bailout:"), "{stderr}");
}

#[test]
fn jit_preserves_runtime_error_location() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "division.ach", "\n\nreturn 10 / 0\n");

    let output = run(&source, "jit");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[line 3] in main"), "{stderr}");
    assert!(stderr.contains("division by zero"), "{stderr}");
}

#[test]
fn explicit_jit_executes_compiled_program() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "scalar.ach", "return 6 * 7\n");
    let binary = directory.path().join("scalar.achb");
    let compile = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("compile")
        .arg(&source)
        .arg("--output")
        .arg(&binary)
        .output()
        .expect("compile ach");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let output = run(&binary, "jit");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn jit_honors_cli_instruction_budget() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "budget.ach", "return 1 + 2\n");
    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("run")
        .arg(&source)
        .arg("--engine")
        .arg("jit")
        .arg("--max-instructions")
        .arg("2")
        .output()
        .expect("run ach");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("instruction budget exhausted"), "{stderr}");
    assert!(stderr.contains("[line 1] in main"), "{stderr}");
}

#[test]
fn jit_bailout_preserves_heap_limit() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "heap.ach", "print([1, 2, 3])\n");
    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("run")
        .arg(&source)
        .arg("--engine")
        .arg("jit")
        .arg("--max-heap")
        .arg("1")
        .output()
        .expect("run ach");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("heap"), "{stderr}");
}

#[test]
fn jit_bailout_runs_under_stress_gc() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(&directory, "gc.ach", "print([1, 2, 3])\n");
    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("run")
        .arg(&source)
        .arg("--engine")
        .arg("jit")
        .arg("--stress-gc")
        .arg("--gc-stats")
        .output()
        .expect("run ach");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Collections:"), "{stderr}");
}
