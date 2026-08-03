use std::process::Command;

#[test]
fn circom_r1cs_proves_and_verifies_on_bls12_381() {
    let directory = tempfile::tempdir().unwrap();
    let source = format!(
        "{}/test/circom/multiplier.circom",
        env!("CARGO_MANIFEST_DIR").trim_end_matches("/cli")
    );
    let r1cs = directory.path().join("circuit.r1cs");
    let wtns = directory.path().join("witness.wtns");
    let prove = Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "--no-config",
            "--prime",
            "bls12-381",
            "--insecure-dev-setup",
            "circom",
            &source,
            "--inputs",
            "a=6,b=7",
            "--r1cs",
            r1cs.to_str().unwrap(),
            "--wtns",
            wtns.to_str().unwrap(),
            "--prove",
        ])
        .output()
        .expect("run BLS12-381 circom prove");
    assert!(
        prove.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&prove.stderr)
    );

    let verify = Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "verify",
            "--proof",
            directory.path().join("proof.json").to_str().unwrap(),
            "--public",
            directory.path().join("public.json").to_str().unwrap(),
            "--vkey",
            directory.path().join("vkey.json").to_str().unwrap(),
            "--curve",
            "bls12-381",
            "--format",
            "json",
        ])
        .output()
        .expect("run detached BLS12-381 verification");
    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&verify.stdout).unwrap();
    assert_eq!(
        result,
        serde_json::json!({"curve": "bls12-381", "valid": true})
    );
}
