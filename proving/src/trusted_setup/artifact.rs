use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use constraints::r1cs::ConstraintSystem;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const TRUSTED_KEY_FORMAT: &str = "achronyme-trusted-key";
pub const TRUSTED_KEY_VERSION: u32 = 1;
pub const MANIFEST_FILE: &str = "manifest.json";
pub const ZKEY_FILE: &str = "proving_key.zkey";
pub const TRANSCRIPT_FILE: &str = "transcript.json";
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedKeyManifest {
    pub format: String,
    pub version: u32,
    pub protocol: String,
    pub curve: String,
    pub r1cs_sha256: String,
    pub zkey_sha256: String,
    pub constraints: u64,
    pub public_inputs: u64,
    pub variables: u64,
    pub ceremony: CeremonyProvenance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CeremonyProvenance {
    pub tool: String,
    pub phase1_sha256: String,
    pub transcript_sha256: String,
    pub contributors: Vec<CeremonyContributor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CeremonyContributor {
    pub id: String,
    pub contribution_hash: String,
}

pub(super) struct TrustedArtifact {
    pub manifest: TrustedKeyManifest,
    pub zkey_file: File,
}

pub(super) fn load(
    store: &Path,
    cs: &ConstraintSystem,
    expected_digest: &str,
) -> Result<TrustedArtifact, String> {
    let artifact_dir = store.join(expected_digest);
    let artifact_metadata = std::fs::symlink_metadata(&artifact_dir).map_err(|error| {
        format!(
            "cannot inspect trusted-key artifact directory `{}`: {error}",
            artifact_dir.display()
        )
    })?;
    if artifact_metadata.file_type().is_symlink() || !artifact_metadata.is_dir() {
        return Err(format!(
            "trusted-key artifact directory `{}` must be a directory, not a symlink",
            artifact_dir.display()
        ));
    }
    let manifest_path = artifact_dir.join(MANIFEST_FILE);
    let manifest_file = open_regular_file(&manifest_path, "trusted-key manifest")?;
    let manifest_len = manifest_file
        .metadata()
        .map_err(|error| format!("cannot inspect trusted-key manifest: {error}"))?
        .len();
    if manifest_len > MAX_MANIFEST_BYTES {
        return Err(format!(
            "trusted-key manifest exceeds {MAX_MANIFEST_BYTES} bytes"
        ));
    }
    let manifest: TrustedKeyManifest = serde_json::from_reader(manifest_file)
        .map_err(|error| format!("invalid trusted-key manifest: {error}"))?;
    validate_manifest(&manifest, cs, expected_digest)?;

    let transcript_path = artifact_dir.join(TRANSCRIPT_FILE);
    let mut transcript_file = open_regular_file(&transcript_path, "ceremony transcript")?;
    let transcript_digest = sha256_reader(&mut transcript_file)?;
    if transcript_digest != manifest.ceremony.transcript_sha256 {
        return Err("ceremony transcript SHA-256 does not match manifest".to_string());
    }

    let zkey_path = artifact_dir.join(ZKEY_FILE);
    let mut zkey_file = open_regular_file(&zkey_path, "trusted proving key")?;
    let zkey_digest = sha256_reader(&mut zkey_file)?;
    if zkey_digest != manifest.zkey_sha256 {
        return Err("trusted proving-key SHA-256 does not match manifest".to_string());
    }

    Ok(TrustedArtifact {
        manifest,
        zkey_file,
    })
}

fn validate_manifest(
    manifest: &TrustedKeyManifest,
    cs: &ConstraintSystem,
    expected_digest: &str,
) -> Result<(), String> {
    if manifest.format != TRUSTED_KEY_FORMAT || manifest.version != TRUSTED_KEY_VERSION {
        return Err("unsupported trusted-key manifest format or version".to_string());
    }
    if manifest.protocol != "groth16" || manifest.curve != "bn254" {
        return Err("trusted-key manifest must select groth16 on bn254".to_string());
    }
    validate_hex(&manifest.r1cs_sha256, 64, "R1CS SHA-256")?;
    validate_hex(&manifest.zkey_sha256, 64, "proving-key SHA-256")?;
    if manifest.r1cs_sha256 != expected_digest {
        return Err("trusted-key manifest does not match exact R1CS SHA-256".to_string());
    }
    if manifest.constraints != cs.num_constraints() as u64
        || manifest.public_inputs != cs.num_pub_inputs() as u64
        || manifest.variables != cs.num_variables() as u64
    {
        return Err("trusted-key manifest circuit dimensions do not match R1CS".to_string());
    }
    if manifest.ceremony.tool.trim().is_empty() {
        return Err("trusted-key manifest ceremony tool cannot be empty".to_string());
    }
    validate_hex(&manifest.ceremony.phase1_sha256, 64, "phase-1 SHA-256")?;
    validate_hex(
        &manifest.ceremony.transcript_sha256,
        64,
        "transcript SHA-256",
    )?;
    if manifest.ceremony.contributors.is_empty() {
        return Err("trusted-key manifest must record a phase-2 contributor".to_string());
    }
    for contributor in &manifest.ceremony.contributors {
        if contributor.id.trim().is_empty() {
            return Err("ceremony contributor id cannot be empty".to_string());
        }
        validate_hex(
            &contributor.contribution_hash,
            128,
            "phase-2 contribution hash",
        )?;
    }
    Ok(())
}

fn open_regular_file(path: &Path, label: &str) -> Result<File, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} `{}` must be a regular file",
            path.display()
        ));
    }
    File::open(path).map_err(|error| format!("cannot open {label} `{}`: {error}", path.display()))
}

fn sha256_reader(reader: &mut File) -> Result<String, String> {
    reader
        .seek(SeekFrom::Start(0))
        .map_err(|error| format!("cannot seek artifact for hashing: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("cannot hash artifact: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex_digest(hasher.finalize()))
}

pub(super) fn sha256_bytes(bytes: &[u8]) -> String {
    hex_digest(Sha256::digest(bytes))
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_hex(value: &str, length: usize, label: &str) -> Result<(), String> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{label} must be {length} lowercase hex characters"));
    }
    Ok(())
}
