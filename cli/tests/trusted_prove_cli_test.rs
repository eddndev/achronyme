use std::process::Command;

use base64::Engine;
use sha2::{Digest, Sha256};

const R1CS_SHA256: &str = "d641de2416205f323639887a159ce421142bea5d60972f41ea0777ba0a2a5082";
const BEACON_CONTRIBUTION_HASH: &str = "d4c8d61b26566a4e3110cb82dc894bdd5621c80d8d4e3d60b12671e6bb215413beebcb0b38cf77a2c77bb34736e1827ef1a65b9bc3694d6a6738867dead458c3";
const BEACON_RANDOMNESS: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const CONTRIBUTED_ZKEY_SHA256: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn install_fixture_store(root: &std::path::Path) {
    let artifact_dir = root.join(R1CS_SHA256);
    std::fs::create_dir_all(&artifact_dir).unwrap();
    let encoded = include_str!("../../proving/tests/fixtures/trusted_basic_arithmetic.zkey.b64");
    let zkey = base64::engine::general_purpose::STANDARD
        .decode(encoded.split_whitespace().collect::<String>())
        .unwrap();
    let zkey_sha256 = sha256_hex(&zkey);
    std::fs::write(artifact_dir.join("proving_key.zkey"), zkey).unwrap();

    let phase1_sha256 = "19a7c2843196e632054628fc0ce4226a6607472a89c057dfc0a61ffaca4a1395";
    let contribution_hash = "c4c8d61b26566a4e3110cb82dc894bdd5621c80d8d4e3d60b12671e6bb215413beebcb0b38cf77a2c77bb34736e1827ef1a65b9bc3694d6a6738867dead458c3";
    let transcript_value = serde_json::json!({
        "format": "achronyme-ceremony-transcript",
        "version": 3,
        "protocol": "groth16",
        "curve": "bn254",
        "circuit": {
            "file": "circuit.r1cs",
            "r1cs_sha256": R1CS_SHA256,
            "constraints": 1,
            "public_inputs": 1,
            "variables": 5
        },
        "phase1": {
            "file": "phase1.ptau",
            "source": "https://example.invalid/test-only.ptau",
            "sha256": phase1_sha256,
            "blake2b512": "9aef0573cef4ded9c4a75f148709056bf989f80dad96876aadeb6f1c6d062391f07a394a9e756d16f7eb233198d5b69407cca44594c763ab4a5b67ae73254678"
        },
        "final_key": {
            "file": "proving_key.zkey",
            "zkey_sha256": zkey_sha256
        },
        "tool": "snarkjs@0.7.6",
        "contributors": [{
            "id": "Achronyme test phase2",
            "contribution_hash": contribution_hash
        }],
        "final_beacon": {
            "source": "https://example.invalid/public-randomness/42",
            "round": 42,
            "randomness": BEACON_RANDOMNESS,
            "evidence_sha256": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "commitment_publication": "https://example.invalid/test-only/commitment-42",
            "commitment_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "iterations": 10,
            "contribution_hash": BEACON_CONTRIBUTION_HASH,
            "contributed_zkey_sha256": CONTRIBUTED_ZKEY_SHA256
        },
        "verification": {
            "phase1_hash": "b2sum phase1.ptau",
            "phase1_transcript": "snarkjs powersoftau verify phase1.ptau",
            "circuit_key_binding": "snarkjs zkey verify circuit.r1cs phase1.ptau proving_key.zkey"
        }
    });
    let mut transcript = serde_json::to_vec_pretty(&transcript_value).unwrap();
    transcript.push(b'\n');
    std::fs::write(artifact_dir.join("transcript.json"), &transcript).unwrap();
    let manifest = serde_json::json!({
        "format": "achronyme-trusted-key",
        "version": 3,
        "protocol": "groth16",
        "curve": "bn254",
        "r1cs_sha256": R1CS_SHA256,
        "zkey_sha256": zkey_sha256,
        "constraints": 1,
        "public_inputs": 1,
        "variables": 5,
        "ceremony": {
            "tool": "snarkjs@0.7.6",
            "phase1_sha256": phase1_sha256,
            "transcript_sha256": sha256_hex(&transcript),
            "contributors": [{
                "id": "Achronyme test phase2",
                "contribution_hash": contribution_hash
            }],
            "final_beacon": {
                "source": "https://example.invalid/public-randomness/42",
                "round": 42,
                "randomness": BEACON_RANDOMNESS,
                "evidence_sha256": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "commitment_publication": "https://example.invalid/test-only/commitment-42",
                "commitment_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "iterations": 10,
                "contribution_hash": BEACON_CONTRIBUTION_HASH,
                "contributed_zkey_sha256": CONTRIBUTED_ZKEY_SHA256
            }
        }
    });
    std::fs::write(
        artifact_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn trusted_circuit_prove_then_detached_verify_succeeds() {
    let directory = tempfile::tempdir().unwrap();
    let store = directory.path().join("trusted-keys");
    let output_dir = directory.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    install_fixture_store(&store);

    let source = format!(
        "{}/test/circuit/basic_arithmetic.ach",
        env!("CARGO_MANIFEST_DIR").trim_end_matches("/cli")
    );
    let r1cs = output_dir.join("circuit.r1cs");
    let wtns = output_dir.join("witness.wtns");
    let prove = Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "--no-config",
            "--trusted-key-dir",
            store.to_str().unwrap(),
            "circuit",
            &source,
            "--inputs",
            "out=42,a=6,b=7",
            "--r1cs",
            r1cs.to_str().unwrap(),
            "--wtns",
            wtns.to_str().unwrap(),
            "--prove",
        ])
        .output()
        .expect("run trusted ach circuit prove");
    assert!(
        prove.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&prove.stderr)
    );
    assert!(!String::from_utf8_lossy(&prove.stderr).contains("LOCAL single-party"));
    assert_eq!(sha256_hex(&std::fs::read(&r1cs).unwrap()), R1CS_SHA256);

    let proof = output_dir.join("proof.json");
    let public = output_dir.join("public.json");
    let vkey = output_dir.join("vkey.json");
    let verify = Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "verify",
            "--proof",
            proof.to_str().unwrap(),
            "--public",
            public.to_str().unwrap(),
            "--vkey",
            vkey.to_str().unwrap(),
            "--curve",
            "bn254",
            "--format",
            "json",
        ])
        .output()
        .expect("run detached ach verify");
    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&verify.stdout).unwrap();
    assert_eq!(result, serde_json::json!({"curve": "bn254", "valid": true}));
}
