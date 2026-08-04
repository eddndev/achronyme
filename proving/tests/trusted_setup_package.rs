#![cfg(feature = "groth16-bn254")]

use std::path::{Path, PathBuf};

use base64::Engine;
use constraints::r1cs::{ConstraintSystem, LinearCombination};
use memory::PrimeId;
use proving::trusted_setup::{
    package_trusted_key, CeremonyContributor, PackageFinalBeacon, PackageTrustedKey,
    TRUSTED_KEY_FORMAT, TRUSTED_KEY_VERSION,
};
use sha2::{Digest, Sha256};

const CONTRIBUTION_HASH: &str = "c4c8d61b26566a4e3110cb82dc894bdd5621c80d8d4e3d60b12671e6bb215413beebcb0b38cf77a2c77bb34736e1827ef1a65b9bc3694d6a6738867dead458c3";
const BEACON_CONTRIBUTION_HASH: &str = "d4c8d61b26566a4e3110cb82dc894bdd5621c80d8d4e3d60b12671e6bb215413beebcb0b38cf77a2c77bb34736e1827ef1a65b9bc3694d6a6738867dead458c3";
const BEACON_RANDOMNESS: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const PHASE1_BLAKE2B512: &str = "9aef0573cef4ded9c4a75f148709056bf989f80dad96876aadeb6f1c6d062391f07a394a9e756d16f7eb233198d5b69407cca44594c763ab4a5b67ae73254678";

fn basic_arithmetic_system() -> ConstraintSystem {
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
    cs
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct Inputs {
    _root: tempfile::TempDir,
    r1cs: PathBuf,
    zkey: PathBuf,
    contributed_zkey: PathBuf,
    phase1: PathBuf,
    store: PathBuf,
    contributors: Vec<CeremonyContributor>,
}

impl Inputs {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let r1cs = root.path().join("circuit.r1cs");
        let zkey = root.path().join("final.zkey");
        let contributed_zkey = root.path().join("contributed.zkey");
        let phase1 = root.path().join("powers.ptau");
        let store = root.path().join("trusted-keys");
        std::fs::write(
            &r1cs,
            constraints::write_r1cs(&basic_arithmetic_system(), PrimeId::Bn254),
        )
        .unwrap();
        let encoded = include_str!("fixtures/trusted_basic_arithmetic.zkey.b64");
        let zkey_bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded.split_whitespace().collect::<String>())
            .unwrap();
        std::fs::write(&zkey, zkey_bytes).unwrap();
        let mut contributed_zkey_bytes = std::fs::read(&zkey).unwrap();
        let last = contributed_zkey_bytes.last_mut().unwrap();
        *last ^= 1;
        std::fs::write(&contributed_zkey, contributed_zkey_bytes).unwrap();
        std::fs::write(&phase1, b"test-only phase-1 fixture\n").unwrap();
        Self {
            _root: root,
            r1cs,
            zkey,
            contributed_zkey,
            phase1,
            store,
            contributors: vec![CeremonyContributor {
                id: "Achronyme test phase2".to_string(),
                contribution_hash: CONTRIBUTION_HASH.to_string(),
            }],
        }
    }

    fn request(&self) -> PackageTrustedKey<'_> {
        PackageTrustedKey {
            r1cs: &self.r1cs,
            zkey: &self.zkey,
            phase1: &self.phase1,
            store: &self.store,
            tool: "snarkjs@0.7.6",
            phase1_source: "https://example.invalid/test-only.ptau",
            phase1_blake2b512: PHASE1_BLAKE2B512,
            contributors: &self.contributors,
            final_beacon: PackageFinalBeacon {
                contributed_zkey: &self.contributed_zkey,
                source: "https://example.invalid/public-randomness/42",
                round: 31006463,
                randomness: BEACON_RANDOMNESS,
                evidence_sha256: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                commitment_publication: "https://example.invalid/test-only/commitment-31006463",
                commitment_sha256:
                    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                iterations: 10,
                contribution_hash: BEACON_CONTRIBUTION_HASH,
            },
        }
    }
}

#[test]
fn packages_a_ceremony_key_with_reproducible_metadata() {
    let inputs = Inputs::new();
    let packaged = package_trusted_key(&inputs.request()).unwrap();
    let r1cs_bytes = std::fs::read(&inputs.r1cs).unwrap();
    let zkey_bytes = std::fs::read(&inputs.zkey).unwrap();
    let phase1_bytes = std::fs::read(&inputs.phase1).unwrap();

    assert_eq!(packaged.manifest.format, TRUSTED_KEY_FORMAT);
    assert_eq!(packaged.manifest.version, TRUSTED_KEY_VERSION);
    assert_eq!(TRUSTED_KEY_VERSION, 3);
    assert_eq!(packaged.manifest.r1cs_sha256, sha256_hex(&r1cs_bytes));
    assert_eq!(packaged.manifest.zkey_sha256, sha256_hex(&zkey_bytes));
    assert_eq!(packaged.manifest.constraints, 1);
    assert_eq!(packaged.manifest.public_inputs, 1);
    assert_eq!(packaged.manifest.variables, 5);
    assert_eq!(
        packaged.manifest.ceremony.phase1_sha256,
        sha256_hex(&phase1_bytes)
    );
    assert_eq!(
        packaged.artifact_dir,
        inputs.store.join(sha256_hex(&r1cs_bytes))
    );

    let transcript: serde_json::Value = serde_json::from_slice(
        &std::fs::read(packaged.artifact_dir.join("transcript.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(transcript["format"], "achronyme-ceremony-transcript");
    assert_eq!(transcript["version"], 3);
    assert_eq!(
        transcript["circuit"]["r1cs_sha256"],
        sha256_hex(&r1cs_bytes)
    );
    assert_eq!(transcript["phase1"]["blake2b512"], PHASE1_BLAKE2B512);
    assert_eq!(
        transcript["final_key"]["zkey_sha256"],
        sha256_hex(&zkey_bytes)
    );
    assert_eq!(
        transcript["contributors"][0]["contribution_hash"],
        CONTRIBUTION_HASH
    );
    assert_eq!(transcript["final_beacon"]["randomness"], BEACON_RANDOMNESS);
    assert_eq!(transcript["final_beacon"]["round"], 31006463);
    assert_eq!(
        transcript["final_beacon"]["evidence_sha256"],
        "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
    );
    assert_eq!(
        transcript["final_beacon"]["commitment_publication"],
        "https://example.invalid/test-only/commitment-31006463"
    );
    assert_eq!(
        transcript["final_beacon"]["commitment_sha256"],
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
    );
    assert_eq!(
        transcript["final_beacon"]["contributed_zkey_sha256"],
        sha256_hex(&std::fs::read(&inputs.contributed_zkey).unwrap())
    );

    proving::trusted_setup::load_trusted_key(&basic_arithmetic_system(), &inputs.store)
        .expect("packaged key must load against the exact circuit");
}

#[test]
fn packaging_refuses_to_replace_an_existing_artifact() {
    let inputs = Inputs::new();
    package_trusted_key(&inputs.request()).unwrap();

    let error = package_trusted_key(&inputs.request()).unwrap_err();
    assert!(error.contains("already exists"), "{error}");
}

#[test]
fn packaging_rejects_invalid_contributor_metadata() {
    let inputs = Inputs::new();
    let invalid = [CeremonyContributor {
        id: "release operator".to_string(),
        contribution_hash: "not-a-hash".to_string(),
    }];
    let mut request = inputs.request();
    request.contributors = &invalid;

    let error = package_trusted_key(&request).unwrap_err();
    assert!(error.contains("phase-2 contribution hash"), "{error}");
    assert!(!inputs.store.exists());
}

#[test]
fn packaging_rejects_invalid_final_beacon_metadata() {
    let inputs = Inputs::new();
    let mut request = inputs.request();
    request.final_beacon.randomness = "predictable";

    let error = package_trusted_key(&request).unwrap_err();
    assert!(error.contains("beacon randomness"), "{error}");
    assert!(!inputs.store.exists());
}

#[test]
fn packaging_rejects_a_zkey_for_different_r1cs_dimensions() {
    let inputs = Inputs::new();
    let mut r1cs = basic_arithmetic_system();
    let extra = r1cs.alloc_witness();
    assert_eq!(extra.index(), 5);
    std::fs::write(&inputs.r1cs, constraints::write_r1cs(&r1cs, PrimeId::Bn254)).unwrap();

    let error = package_trusted_key(&inputs.request()).unwrap_err();
    assert!(
        error.contains("zkey dimensions do not match R1CS"),
        "{error}"
    );
    assert!(!inputs.store.exists());
}

#[cfg(unix)]
#[test]
fn packaging_rejects_symlinked_inputs() {
    use std::os::unix::fs::symlink;

    let inputs = Inputs::new();
    let alias = inputs.r1cs.with_file_name("circuit-alias.r1cs");
    symlink(&inputs.r1cs, &alias).unwrap();
    let mut request = inputs.request();
    request.r1cs = Path::new(&alias);

    let error = package_trusted_key(&request).unwrap_err();
    assert!(error.contains("regular file"), "{error}");
}
