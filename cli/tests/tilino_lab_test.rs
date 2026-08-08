#[cfg(unix)]
#[test]
fn modular_private_auction_passes_the_full_engine_contract() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let contract = workspace.join("test/projects/tilino-lab/test.sh");
    assert!(
        contract.is_file(),
        "tilino-lab contract is missing: {}",
        contract.display()
    );

    let output = std::process::Command::new("bash")
        .arg(&contract)
        .env("ACH_BIN", env!("CARGO_BIN_EXE_ach"))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "tilino-lab failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
