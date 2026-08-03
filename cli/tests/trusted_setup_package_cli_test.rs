use std::path::PathBuf;
use std::process::{Command, Output};

use base64::Engine;
use constraints::r1cs::{ConstraintSystem, LinearCombination};
use memory::PrimeId;
use sha2::{Digest, Sha256};

const CONTRIBUTION_HASH: &str = "c4c8d61b26566a4e3110cb82dc894bdd5621c80d8d4e3d60b12671e6bb215413beebcb0b38cf77a2c77bb34736e1827ef1a65b9bc3694d6a6738867dead458c3";
const PHASE1_BLAKE2B512: &str = "9aef0573cef4ded9c4a75f148709056bf989f80dad96876aadeb6f1c6d062391f07a394a9e756d16f7eb233198d5b69407cca44594c763ab4a5b67ae73254678";

struct Fixture {
    _root: tempfile::TempDir,
    r1cs: PathBuf,
    zkey: PathBuf,
    phase1: PathBuf,
    store: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let r1cs = root.path().join("circuit.r1cs");
        let zkey = root.path().join("final.zkey");
        let phase1 = root.path().join("phase1.ptau");
        let store = root.path().join("trusted-keys");

        let mut cs: ConstraintSystem = ConstraintSystem::new();
        let out = cs.alloc_input();
        let a = cs.alloc_witness();
        let b = cs.alloc_witness();
        let _unused = cs.alloc_witness();
        cs.enforce(
            LinearCombination::from_variable(a),
            LinearCombination::from_variable(b),
            LinearCombination::from_variable(out),
        );
        std::fs::write(&r1cs, constraints::write_r1cs(&cs, PrimeId::Bn254)).unwrap();
        let encoded =
            include_str!("../../proving/tests/fixtures/trusted_basic_arithmetic.zkey.b64");
        let zkey_bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded.split_whitespace().collect::<String>())
            .unwrap();
        std::fs::write(&zkey, zkey_bytes).unwrap();
        std::fs::write(&phase1, b"test-only phase-1 fixture\n").unwrap();
        std::fs::write(root.path().join("achronyme.toml"), "invalid = [").unwrap();

        Self {
            _root: root,
            r1cs,
            zkey,
            phase1,
            store,
        }
    }

    fn run(&self, contributor: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ach"))
            .current_dir(self.r1cs.parent().unwrap())
            .args([
                "trusted-setup",
                "package",
                "--r1cs",
                self.r1cs.to_str().unwrap(),
                "--zkey",
                self.zkey.to_str().unwrap(),
                "--phase1",
                self.phase1.to_str().unwrap(),
                "--store",
                self.store.to_str().unwrap(),
                "--tool",
                "snarkjs@0.7.6",
                "--phase1-source",
                "https://example.invalid/test-only.ptau",
                "--phase1-blake2b512",
                PHASE1_BLAKE2B512,
                "--contributor",
                contributor,
                "--format",
                "json",
            ])
            .output()
            .expect("run trusted-setup package")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn packages_without_loading_project_configuration() {
    let fixture = Fixture::new();
    let output = fixture.run(&format!("independent-contributor={CONTRIBUTION_HASH}"));
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["format"], "achronyme-trusted-key");
    assert_eq!(value["version"], 1);
    assert_eq!(
        value["r1cs_sha256"],
        sha256_hex(&std::fs::read(&fixture.r1cs).unwrap())
    );
    assert!(PathBuf::from(value["artifact_dir"].as_str().unwrap()).is_dir());
}

#[test]
fn malformed_contributor_fails_before_creating_the_store() {
    let fixture = Fixture::new();
    let output = fixture.run("operator=not-a-hash");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("contribution hash"));
    assert!(!fixture.store.exists());
}

#[test]
fn package_help_has_no_entropy_argument() {
    let output = Command::new(env!("CARGO_BIN_EXE_ach"))
        .args(["trusted-setup", "package", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    assert!(!help.contains("entropy"));
}
