use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use memory::{Bn254Fr, FieldElement};

use super::artifact::{
    validate_beacon, validate_contributor, CeremonyBeacon, CeremonyContributor, CeremonyProvenance,
    CeremonyTranscript, TranscriptCircuit, TranscriptFinalKey, TranscriptPhase1,
    TranscriptVerification, TrustedKeyManifest, TRUSTED_KEY_FORMAT, TRUSTED_KEY_VERSION, ZKEY_FILE,
};
use super::install::{install_artifact, json_bytes};
use super::support::{open_regular_file, sha256_bytes, sha256_reader, validate_hex};

#[derive(Clone, Copy)]
pub struct PackageFinalBeacon<'a> {
    pub contributed_zkey: &'a Path,
    pub source: &'a str,
    pub round: u64,
    pub randomness: &'a str,
    pub evidence_sha256: &'a str,
    pub commitment_publication: &'a str,
    pub commitment_sha256: &'a str,
    pub iterations: u32,
    pub contribution_hash: &'a str,
}

impl PackageFinalBeacon<'_> {
    fn metadata(self, contributed_zkey_sha256: String) -> CeremonyBeacon {
        CeremonyBeacon {
            source: self.source.to_string(),
            round: self.round,
            randomness: self.randomness.to_string(),
            evidence_sha256: self.evidence_sha256.to_string(),
            commitment_publication: self.commitment_publication.to_string(),
            commitment_sha256: self.commitment_sha256.to_string(),
            iterations: self.iterations,
            contribution_hash: self.contribution_hash.to_string(),
            contributed_zkey_sha256,
        }
    }
}

pub struct PackageTrustedKey<'a> {
    pub r1cs: &'a Path,
    pub zkey: &'a Path,
    pub phase1: &'a Path,
    pub store: &'a Path,
    pub tool: &'a str,
    pub phase1_source: &'a str,
    pub phase1_blake2b512: &'a str,
    pub contributors: &'a [CeremonyContributor],
    pub final_beacon: PackageFinalBeacon<'a>,
}

#[derive(Debug)]
pub struct PackagedTrustedKey {
    pub artifact_dir: PathBuf,
    pub manifest: TrustedKeyManifest,
}

#[derive(Clone, Copy)]
struct R1csMetadata {
    constraints: u64,
    public_inputs: u64,
    variables: u64,
}

#[derive(Clone, Copy)]
struct Section {
    offset: u64,
    size: u64,
}

pub fn package_trusted_key(request: &PackageTrustedKey<'_>) -> Result<PackagedTrustedKey, String> {
    validate_request(request)?;
    let mut r1cs_file = open_regular_file(request.r1cs, "R1CS artifact")?;
    let r1cs_sha256 = sha256_reader(&mut r1cs_file)?;
    let metadata = inspect_r1cs(&mut r1cs_file)?;

    let mut zkey_file = open_regular_file(request.zkey, "final proving key")?;
    let zkey_sha256 = sha256_reader(&mut zkey_file)?;
    inspect_zkey(&mut zkey_file, metadata)?;

    let mut contributed_zkey_file = open_regular_file(
        request.final_beacon.contributed_zkey,
        "contributed proving key",
    )?;
    let contributed_zkey_sha256 = sha256_reader(&mut contributed_zkey_file)?;
    inspect_zkey(&mut contributed_zkey_file, metadata)?;
    if contributed_zkey_sha256 == zkey_sha256 {
        return Err("final beacon did not change the contributed zkey".to_string());
    }
    let final_beacon = request
        .final_beacon
        .metadata(contributed_zkey_sha256.clone());

    let mut phase1_file = open_regular_file(request.phase1, "phase-1 artifact")?;
    let phase1_sha256 = sha256_reader(&mut phase1_file)?;

    let transcript = CeremonyTranscript {
        format: "achronyme-ceremony-transcript".to_string(),
        version: TRUSTED_KEY_VERSION,
        protocol: "groth16".to_string(),
        curve: "bn254".to_string(),
        circuit: TranscriptCircuit {
            file: "circuit.r1cs".to_string(),
            r1cs_sha256: r1cs_sha256.clone(),
            constraints: metadata.constraints,
            public_inputs: metadata.public_inputs,
            variables: metadata.variables,
        },
        phase1: TranscriptPhase1 {
            file: "phase1.ptau".to_string(),
            source: request.phase1_source.to_string(),
            sha256: phase1_sha256.clone(),
            blake2b512: request.phase1_blake2b512.to_string(),
        },
        final_key: TranscriptFinalKey {
            file: ZKEY_FILE.to_string(),
            zkey_sha256: zkey_sha256.clone(),
        },
        tool: request.tool.to_string(),
        contributors: request.contributors.to_vec(),
        final_beacon: final_beacon.clone(),
        verification: TranscriptVerification {
            phase1_hash: "b2sum phase1.ptau".to_string(),
            phase1_transcript: "snarkjs powersoftau verify phase1.ptau".to_string(),
            circuit_key_binding: "snarkjs zkey verify circuit.r1cs phase1.ptau proving_key.zkey"
                .to_string(),
        },
    };
    let transcript_bytes = json_bytes(&transcript)?;
    let manifest = TrustedKeyManifest {
        format: TRUSTED_KEY_FORMAT.to_string(),
        version: TRUSTED_KEY_VERSION,
        protocol: "groth16".to_string(),
        curve: "bn254".to_string(),
        r1cs_sha256: r1cs_sha256.clone(),
        zkey_sha256,
        constraints: metadata.constraints,
        public_inputs: metadata.public_inputs,
        variables: metadata.variables,
        ceremony: CeremonyProvenance {
            tool: request.tool.to_string(),
            phase1_sha256,
            transcript_sha256: sha256_bytes(&transcript_bytes),
            contributors: request.contributors.to_vec(),
            final_beacon,
        },
    };
    let manifest_bytes = json_bytes(&manifest)?;
    install_artifact(
        request.store,
        &r1cs_sha256,
        request.zkey,
        &manifest_bytes,
        &transcript_bytes,
    )?;
    Ok(PackagedTrustedKey {
        artifact_dir: request.store.join(r1cs_sha256),
        manifest,
    })
}

fn validate_request(request: &PackageTrustedKey<'_>) -> Result<(), String> {
    if request.tool.trim().is_empty() {
        return Err("ceremony tool cannot be empty".to_string());
    }
    if request.phase1_source.trim().is_empty() {
        return Err("phase-1 source cannot be empty".to_string());
    }
    validate_hex(
        request.phase1_blake2b512,
        128,
        "phase-1 published BLAKE2b-512",
    )?;
    if request.contributors.is_empty() {
        return Err("at least one phase-2 contributor is required".to_string());
    }
    for contributor in request.contributors {
        validate_contributor(contributor)?;
    }
    validate_beacon(&request.final_beacon.metadata("0".repeat(64)))?;
    Ok(())
}

fn inspect_r1cs(file: &mut File) -> Result<R1csMetadata, String> {
    let sections = read_sections(file, b"r1cs", 1, "R1CS")?;
    let header = sections
        .get(&1)
        .ok_or_else(|| "R1CS is missing header section 1".to_string())?;
    let constraints = sections
        .get(&2)
        .ok_or_else(|| "R1CS is missing constraint section 2".to_string())?;
    let wire_map = sections
        .get(&3)
        .ok_or_else(|| "R1CS is missing wire-map section 3".to_string())?;
    if constraints.size == 0 {
        return Err("R1CS constraint section is empty".to_string());
    }
    file.seek(SeekFrom::Start(header.offset))
        .map_err(|error| format!("cannot read R1CS header: {error}"))?;
    let field_size = read_u32(file, "R1CS")? as usize;
    if field_size != 32 || header.size != 32 + field_size as u64 {
        return Err("R1CS header is not canonical BN254".to_string());
    }
    let mut modulus = vec![0u8; field_size];
    file.read_exact(&mut modulus)
        .map_err(|error| format!("truncated R1CS modulus: {error}"))?;
    if modulus != FieldElement::<Bn254Fr>::modulus_le_bytes() {
        return Err("R1CS scalar field is not BN254".to_string());
    }
    let variables = read_u32(file, "R1CS")? as u64;
    let public_outputs = read_u32(file, "R1CS")? as u64;
    let public_inputs = read_u32(file, "R1CS")? as u64;
    let private_inputs = read_u32(file, "R1CS")? as u64;
    let labels = read_u64(file, "R1CS")?;
    let constraint_count = read_u32(file, "R1CS")? as u64;
    if variables == 0
        || public_outputs != 0
        || variables != 1 + public_inputs + private_inputs
        || labels != variables
        || constraint_count == 0
        || wire_map.size != variables.saturating_mul(8)
    {
        return Err("R1CS header dimensions are not a canonical Achronyme export".to_string());
    }
    Ok(R1csMetadata {
        constraints: constraint_count,
        public_inputs,
        variables,
    })
}

fn inspect_zkey(file: &mut File, r1cs: R1csMetadata) -> Result<(), String> {
    let sections = read_sections(file, b"zkey", 1, "zkey")?;
    let header = sections
        .get(&2)
        .ok_or_else(|| "zkey is missing Groth16 header section 2".to_string())?;
    if header.size != 660 {
        return Err("zkey has an invalid Groth16 header size".to_string());
    }
    file.seek(SeekFrom::Start(header.offset))
        .map_err(|error| format!("cannot read zkey header: {error}"))?;
    if read_u32(file, "zkey")? != 32 {
        return Err("zkey base-field width is not BN254".to_string());
    }
    file.seek(SeekFrom::Current(32))
        .map_err(|error| format!("cannot read zkey base field: {error}"))?;
    if read_u32(file, "zkey")? != 32 {
        return Err("zkey scalar-field width is not BN254".to_string());
    }
    let mut scalar_modulus = [0u8; 32];
    file.read_exact(&mut scalar_modulus)
        .map_err(|error| format!("truncated zkey scalar field: {error}"))?;
    if scalar_modulus.as_slice() != FieldElement::<Bn254Fr>::modulus_le_bytes() {
        return Err("zkey scalar field is not BN254".to_string());
    }
    let variables = read_u32(file, "zkey")? as u64;
    let public_inputs = read_u32(file, "zkey")? as u64;
    let domain_size = read_u32(file, "zkey")? as u64;
    if variables != r1cs.variables
        || public_inputs != r1cs.public_inputs
        || domain_size < r1cs.constraints + r1cs.public_inputs + 1
    {
        return Err("zkey dimensions do not match R1CS".to_string());
    }
    Ok(())
}

fn read_sections(
    file: &mut File,
    expected_magic: &[u8; 4],
    expected_version: u32,
    label: &str,
) -> Result<BTreeMap<u32, Section>, String> {
    let file_len = file
        .metadata()
        .map_err(|error| format!("cannot inspect {label}: {error}"))?
        .len();
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("cannot read {label}: {error}"))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|error| format!("truncated {label} header: {error}"))?;
    if &magic != expected_magic || read_u32(file, label)? != expected_version {
        return Err(format!("unsupported {label} format or version"));
    }
    let section_count = read_u32(file, label)?;
    if section_count == 0 || section_count > 64 {
        return Err(format!("invalid {label} section count {section_count}"));
    }
    let mut sections = BTreeMap::new();
    for _ in 0..section_count {
        let id = read_u32(file, label)?;
        let size = read_u64(file, label)?;
        let offset = file
            .stream_position()
            .map_err(|error| format!("cannot inspect {label} section: {error}"))?;
        let end = offset
            .checked_add(size)
            .ok_or_else(|| format!("{label} section length overflow"))?;
        if end > file_len || sections.insert(id, Section { offset, size }).is_some() {
            return Err(format!("invalid or duplicate {label} section {id}"));
        }
        file.seek(SeekFrom::Start(end))
            .map_err(|error| format!("cannot seek over {label} section {id}: {error}"))?;
    }
    if file.stream_position().ok() != Some(file_len) {
        return Err(format!("{label} has trailing or unindexed bytes"));
    }
    Ok(sections)
}

fn read_u32(reader: &mut File, label: &str) -> Result<u32, String> {
    let mut bytes = [0u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| format!("truncated {label} integer: {error}"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut File, label: &str) -> Result<u64, String> {
    let mut bytes = [0u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| format!("truncated {label} integer: {error}"))?;
    Ok(u64::from_le_bytes(bytes))
}
