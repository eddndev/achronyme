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

    let occupied_listener = (30_000..50_000)
        .find_map(|port| std::net::TcpListener::bind(("127.0.0.1", port)).ok())
        .expect("a loopback port is available for the collision regression");
    let occupied_port = occupied_listener.local_addr().unwrap().port();

    let output = std::process::Command::new("bash")
        .arg(&contract)
        .env("ACH_BIN", env!("CARGO_BIN_EXE_ach"))
        .env("TILINO_PORT_BASE", occupied_port.to_string())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "tilino-lab failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
