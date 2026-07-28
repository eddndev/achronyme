#![cfg(feature = "llvm")]

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn runtime_archive() -> PathBuf {
    workspace().join("target/debug/libakron_aot_runtime.a")
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
