use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use constraints::r1cs::{ConstraintSystem, LinearCombination};
use memory::FieldElement;
use tempfile::TempDir;

struct ProofArtifacts {
    _directory: TempDir,
    proof: PathBuf,
    public: PathBuf,
    vkey: PathBuf,
}

fn basic_arithmetic_system() -> (ConstraintSystem, Vec<FieldElement>) {
    let mut cs = ConstraintSystem::new();
    let out = cs.alloc_input();
    let a = cs.alloc_witness();
    let b = cs.alloc_witness();
    cs.enforce(
        LinearCombination::from_variable(a),
        LinearCombination::from_variable(b),
        LinearCombination::from_variable(out),
    );
    let witness = vec![
        FieldElement::from_u64(1),
        FieldElement::from_u64(42),
        FieldElement::from_u64(6),
        FieldElement::from_u64(7),
    ];
    (cs, witness)
}

fn generate_bn254_artifacts() -> ProofArtifacts {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("cache");
    let (cs, witness) = basic_arithmetic_system();
    let result = proving::groth16_bn254::generate_proof(
        &cs,
        &witness,
        &cache,
        &proving::groth16::ProvingKeySource::InsecureLocal,
    )
    .unwrap();
    write_artifacts(directory, result)
}

fn generate_bls12_381_artifacts() -> ProofArtifacts {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("cache");
    let (cs, witness) = basic_arithmetic_system();
    let result = proving::groth16_bls12_381::generate_proof(
        &cs,
        &witness,
        &cache,
        &proving::groth16::ProvingKeySource::InsecureLocal,
    )
    .unwrap();
    write_artifacts(directory, result)
}

fn write_artifacts(directory: TempDir, result: akron::ProveResult) -> ProofArtifacts {
    let akron::ProveResult::Proof {
        proof_json,
        public_json,
        vkey_json,
    } = result
    else {
        panic!("expected Groth16 proof artifacts");
    };
    let proof = directory.path().join("proof.json");
    let public = directory.path().join("public.json");
    let vkey = directory.path().join("verification_key.json");
    std::fs::write(&proof, proof_json).unwrap();
    std::fs::write(&public, public_json).unwrap();
    std::fs::write(&vkey, vkey_json).unwrap();
    ProofArtifacts {
        _directory: directory,
        proof,
        public,
        vkey,
    }
}

fn verify(artifacts: &ProofArtifacts, curve: &str, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "verify",
            "--proof",
            artifacts.proof.to_str().unwrap(),
            "--public",
            artifacts.public.to_str().unwrap(),
            "--vkey",
            artifacts.vkey.to_str().unwrap(),
            "--curve",
            curve,
            "--format",
            format,
        ])
        .output()
        .expect("run ach verify")
}

fn edit_json(path: &Path, edit: impl FnOnce(&mut serde_json::Value)) {
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    edit(&mut value);
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

#[test]
fn verifies_bn254_artifacts_in_a_fresh_process_with_json_output() {
    let artifacts = generate_bn254_artifacts();
    let output = verify(&artifacts, "bn254", "json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value, serde_json::json!({"curve": "bn254", "valid": true}));
}

#[test]
fn verifies_bls12_381_artifacts_in_a_fresh_process() {
    let artifacts = generate_bls12_381_artifacts();
    let output = verify(&artifacts, "bls12-381", "text");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "proof valid (bls12-381)\n"
    );
}

#[test]
fn proof_public_and_vkey_tampering_each_fail() {
    for artifact in ["proof", "public", "vkey"] {
        let artifacts = generate_bn254_artifacts();
        match artifact {
            "proof" => edit_json(&artifacts.proof, |value| value["pi_a"][0] = "1".into()),
            "public" => edit_json(&artifacts.public, |value| value[0] = "43".into()),
            "vkey" => edit_json(&artifacts.vkey, |value| value["IC"][0][0] = "1".into()),
            _ => unreachable!(),
        }

        let output = verify(&artifacts, "bn254", "json");
        assert!(!output.status.success(), "tampered {artifact} was accepted");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["curve"], "bn254");
        assert_eq!(value["valid"], false);
    }
}

#[test]
fn malformed_g2_artifact_fails_without_panicking() {
    let artifacts = generate_bn254_artifacts();
    edit_json(&artifacts.proof, |value| {
        value["pi_b"] = serde_json::json!([["1"], ["2"], ["1", "0"]]);
    });

    let output = verify(&artifacts, "bn254", "json");
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["valid"], false);
    assert!(value["error"].as_str().unwrap().contains("G2 x"));
}

#[test]
fn proof_and_vkey_metadata_tampering_each_fail() {
    for artifact in ["proof-curve", "vkey-curve", "vkey-public-count"] {
        let artifacts = generate_bn254_artifacts();
        match artifact {
            "proof-curve" => edit_json(&artifacts.proof, |value| {
                value["curve"] = "bls12-381".into()
            }),
            "vkey-curve" => edit_json(&artifacts.vkey, |value| value["curve"] = "bls12-381".into()),
            "vkey-public-count" => edit_json(&artifacts.vkey, |value| value["nPublic"] = 2.into()),
            _ => unreachable!(),
        }

        let output = verify(&artifacts, "bn254", "json");
        assert!(!output.status.success(), "tampered {artifact} was accepted");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["valid"], false);
    }
}

#[test]
fn curve_is_required_and_explicit() {
    let artifacts = generate_bn254_artifacts();
    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "verify",
            "--proof",
            artifacts.proof.to_str().unwrap(),
            "--public",
            artifacts.public.to_str().unwrap(),
            "--vkey",
            artifacts.vkey.to_str().unwrap(),
        ])
        .output()
        .expect("run ach verify");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--curve"));
}
