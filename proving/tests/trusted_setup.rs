#![cfg(feature = "groth16-bn254")]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine;
use constraints::r1cs::{ConstraintSystem, LinearCombination};
use memory::FieldElement;
use serde_json::Value;
use sha2::{Digest, Sha256};

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

fn basic_arithmetic_system() -> (ConstraintSystem, Vec<FieldElement>) {
    let mut cs = ConstraintSystem::new();
    let out = cs.alloc_input();
    let a = cs.alloc_witness();
    let b = cs.alloc_witness();
    let _unused = cs.alloc_witness();
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
        FieldElement::from_u64(42),
    ];
    (cs, witness)
}

fn same_dimensions_different_system() -> ConstraintSystem {
    let mut cs = ConstraintSystem::new();
    let out = cs.alloc_input();
    let a = cs.alloc_witness();
    let b = cs.alloc_witness();
    let unused = cs.alloc_witness();
    cs.enforce(
        LinearCombination::from_variable(unused),
        LinearCombination::from_variable(b),
        LinearCombination::from_variable(out),
    );
    assert_eq!(a.index(), 2);
    assert_eq!(out.index(), 1);
    cs
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn unique_temp_root(label: &str) -> PathBuf {
    let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("ach-trusted-{label}-{}-{id}", std::process::id()))
}

struct FixtureStore {
    root: PathBuf,
    digest: String,
}

impl FixtureStore {
    fn new(cs: &ConstraintSystem) -> Self {
        let root = unique_temp_root("zkey");
        let digest = proving::trusted_setup::r1cs_sha256(cs);
        let artifact_dir = root.join(&digest);
        std::fs::create_dir_all(&artifact_dir).unwrap();

        let encoded = include_str!("fixtures/trusted_basic_arithmetic.zkey.b64");
        let zkey = base64::engine::general_purpose::STANDARD
            .decode(encoded.split_whitespace().collect::<String>())
            .unwrap();
        let zkey_sha256 = sha256_hex(&zkey);
        std::fs::write(artifact_dir.join("proving_key.zkey"), zkey).unwrap();

        let transcript = b"{\"fixture\":\"local-only\",\"snarkjs\":\"0.7.6\"}\n";
        std::fs::write(artifact_dir.join("transcript.json"), transcript).unwrap();
        let manifest = serde_json::json!({
            "format": "achronyme-trusted-key",
            "version": 1,
            "protocol": "groth16",
            "curve": "bn254",
            "r1cs_sha256": digest,
            "zkey_sha256": zkey_sha256,
            "constraints": 1,
            "public_inputs": 1,
            "variables": 5,
            "ceremony": {
                "tool": "snarkjs@0.7.6",
                "phase1_sha256": "19a7c2843196e632054628fc0ce4226a6607472a89c057dfc0a61ffaca4a1395",
                "transcript_sha256": sha256_hex(transcript),
                "contributors": [{
                    "id": "Achronyme test phase2",
                    "contribution_hash": "c4c8d61b26566a4e3110cb82dc894bdd5621c80d8d4e3d60b12671e6bb215413beebcb0b38cf77a2c77bb34736e1827ef1a65b9bc3694d6a6738867dead458c3"
                }]
            }
        });
        std::fs::write(
            artifact_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        Self { root, digest }
    }

    fn artifact_dir(&self) -> PathBuf {
        self.root.join(&self.digest)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.artifact_dir().join(name)
    }

    fn edit_manifest(&self, edit: impl FnOnce(&mut Value)) {
        let path = self.path("manifest.json");
        let mut manifest: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        edit(&mut manifest);
        std::fs::write(path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    }

    fn update_zkey_hash(&self) {
        let digest = sha256_hex(&std::fs::read(self.path("proving_key.zkey")).unwrap());
        self.edit_manifest(|manifest| manifest["zkey_sha256"] = digest.into());
    }

    fn clone_artifact_for(&self, cs: &ConstraintSystem) {
        let digest = proving::trusted_setup::r1cs_sha256(cs);
        let target = self.root.join(&digest);
        std::fs::create_dir_all(&target).unwrap();
        for name in ["manifest.json", "proving_key.zkey", "transcript.json"] {
            std::fs::copy(self.path(name), target.join(name)).unwrap();
        }
        let manifest_path = target.join("manifest.json");
        let mut manifest: Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        manifest["r1cs_sha256"] = digest.into();
        std::fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    }
}

impl Drop for FixtureStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn load_error(cs: &ConstraintSystem, store: &Path) -> String {
    proving::trusted_setup::load_trusted_key(cs, store)
        .err()
        .expect("trusted artifact should be rejected")
}

#[test]
fn trusted_zkey_proves_exact_exported_r1cs() {
    let (cs, witness) = basic_arithmetic_system();
    let store = FixtureStore::new(&cs);
    let result = proving::groth16_bn254::generate_proof(
        &cs,
        &witness,
        &store.root,
        &proving::groth16::ProvingKeySource::TrustedStore(store.root.clone()),
    )
    .expect("ceremony-derived zkey should prove the exact circuit");

    let akron::ProveResult::Proof {
        proof_json,
        public_json,
        vkey_json,
    } = result
    else {
        panic!("expected proof artifacts");
    };
    assert!(
        proving::groth16_bn254::verify_proof_from_json(&proof_json, &public_json, &vkey_json)
            .expect("verify trusted proof")
    );
}

#[test]
fn rejects_zkey_hash_tampering() {
    let (cs, _) = basic_arithmetic_system();
    let store = FixtureStore::new(&cs);
    let path = store.path("proving_key.zkey");
    let mut bytes = std::fs::read(&path).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    std::fs::write(path, bytes).unwrap();

    assert!(load_error(&cs, &store.root).contains("SHA-256 does not match manifest"));
}

#[test]
fn rejects_transcript_hash_tampering() {
    let (cs, _) = basic_arithmetic_system();
    let store = FixtureStore::new(&cs);
    std::fs::write(store.path("transcript.json"), b"changed\n").unwrap();

    assert!(load_error(&cs, &store.root).contains("transcript SHA-256"));
}

#[test]
fn rejects_manifest_curve_tampering() {
    let (cs, _) = basic_arithmetic_system();
    let store = FixtureStore::new(&cs);
    store.edit_manifest(|manifest| manifest["curve"] = "bls12-381".into());

    assert!(load_error(&cs, &store.root).contains("groth16 on bn254"));
}

#[test]
fn rejects_malformed_zkey_even_when_manifest_hash_matches() {
    let (cs, _) = basic_arithmetic_system();
    let store = FixtureStore::new(&cs);
    std::fs::write(store.path("proving_key.zkey"), b"zkey\x01\0\0\0").unwrap();
    store.update_zkey_hash();

    assert!(load_error(&cs, &store.root).contains("truncated zkey integer"));
}

#[test]
fn rejects_same_size_key_for_different_constraints() {
    let (fixture_cs, _) = basic_arithmetic_system();
    let other_cs = same_dimensions_different_system();
    let store = FixtureStore::new(&fixture_cs);
    store.clone_artifact_for(&other_cs);

    assert!(load_error(&other_cs, &store.root).contains("constraint matrices do not match"));
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_artifact_directory() {
    use std::os::unix::fs::symlink;

    let (cs, _) = basic_arithmetic_system();
    let store = FixtureStore::new(&cs);
    let alias_root = unique_temp_root("alias");
    std::fs::create_dir_all(&alias_root).unwrap();
    symlink(store.artifact_dir(), alias_root.join(&store.digest)).unwrap();

    let error = load_error(&cs, &alias_root);
    let _ = std::fs::remove_dir_all(alias_root);
    assert!(error.contains("artifact directory") && error.contains("must be a directory"));
}
