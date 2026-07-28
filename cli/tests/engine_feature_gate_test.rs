#![cfg(not(feature = "llvm"))]

use std::process::Command;

#[test]
fn explicit_jit_explains_how_to_enable_llvm() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("scalar.ach");
    std::fs::write(&source, "return 40 + 2\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("run")
        .arg(&source)
        .arg("--engine")
        .arg("jit")
        .output()
        .expect("run ach");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not include LLVM"), "{stderr}");
    assert!(stderr.contains("--features llvm"), "{stderr}");
}

#[test]
fn aot_explains_how_to_enable_llvm() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("scalar.ach");
    std::fs::write(&source, "return 40 + 2\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("aot")
        .arg(&source)
        .output()
        .expect("run ach");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not include LLVM AOT"), "{stderr}");
    assert!(stderr.contains("--features llvm"), "{stderr}");
}
