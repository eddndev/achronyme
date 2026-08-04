use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use ark_bn254::{Bn254, Fq, Fr};
use ark_ff::{BigInteger, PrimeField, Zero};
use ark_groth16::ProvingKey;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem as ArkConstraintSystem};
use constraints::r1cs::ConstraintSystem;

use crate::groth16::AchronymeCircuit;

#[derive(Clone, Copy, Debug)]
struct Section {
    offset: u64,
    size: u64,
}

#[derive(Clone, Copy, Debug)]
struct ZkeyHeader {
    n_vars: usize,
    n_public: usize,
    domain_size: usize,
}

pub(super) fn parse_and_validate(
    file: &mut File,
    cs: &ConstraintSystem,
) -> Result<ProvingKey<Bn254>, String> {
    let header = validate_envelope(file, cs)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("cannot rewind trusted proving key: {error}"))?;
    let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        semaphore_rs_ark_circom::read_zkey(file)
    }))
    .map_err(|_| "trusted proving-key parser rejected malformed data".to_string())?
    .map_err(|error| format!("cannot parse trusted proving key: {error}"))?;
    let (proving_key, zkey_matrices) = parsed;

    validate_key_shape(&proving_key, header, cs)?;
    validate_constraint_binding(cs, zkey_matrices)?;
    Ok(proving_key)
}

fn validate_envelope(file: &mut File, cs: &ConstraintSystem) -> Result<ZkeyHeader, String> {
    let file_len = file
        .metadata()
        .map_err(|error| format!("cannot inspect trusted proving key: {error}"))?
        .len();
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("cannot read trusted proving key: {error}"))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|error| format!("truncated trusted proving-key header: {error}"))?;
    if &magic != b"zkey" {
        return Err("trusted proving key has invalid zkey magic".to_string());
    }
    let version = read_u32(file)?;
    if version != 1 {
        return Err(format!("unsupported zkey version {version}"));
    }
    let section_count = read_u32(file)?;
    if !(8..=32).contains(&section_count) {
        return Err(format!("invalid zkey section count {section_count}"));
    }

    let mut sections = BTreeMap::new();
    for _ in 0..section_count {
        let id = read_u32(file)?;
        let size = read_u64(file)?;
        let offset = file
            .stream_position()
            .map_err(|error| format!("cannot inspect zkey section: {error}"))?;
        let end = offset
            .checked_add(size)
            .ok_or_else(|| "zkey section length overflow".to_string())?;
        if end > file_len {
            return Err(format!("zkey section {id} extends past end of file"));
        }
        if sections.insert(id, Section { offset, size }).is_some() {
            return Err(format!("duplicate zkey section {id}"));
        }
        file.seek(SeekFrom::Start(end))
            .map_err(|error| format!("cannot seek over zkey section {id}: {error}"))?;
    }
    if file.stream_position().ok() != Some(file_len) {
        return Err("zkey has trailing or unindexed bytes".to_string());
    }
    for id in 1..=9 {
        if !sections.contains_key(&id) {
            return Err(format!("zkey is missing required section {id}"));
        }
    }
    validate_protocol_section(file, sections[&1])?;
    let header = read_groth16_header(file, sections[&2])?;
    if header.n_vars != cs.num_variables()
        || header.n_public != cs.num_pub_inputs()
        || header.domain_size < cs.num_constraints() + cs.num_pub_inputs() + 1
    {
        return Err("zkey dimensions do not match exact R1CS".to_string());
    }
    validate_query_sizes(&sections, header)?;
    validate_coefficient_section(file, sections[&4], header, cs.num_constraints())?;
    Ok(header)
}

fn validate_protocol_section(file: &mut File, section: Section) -> Result<(), String> {
    if section.size != 4 {
        return Err("invalid zkey protocol section size".to_string());
    }
    file.seek(SeekFrom::Start(section.offset))
        .map_err(|error| format!("cannot read zkey protocol: {error}"))?;
    if read_u32(file)? != 1 {
        return Err("zkey protocol is not Groth16".to_string());
    }
    Ok(())
}

fn read_groth16_header(file: &mut File, section: Section) -> Result<ZkeyHeader, String> {
    if section.size != 660 {
        return Err("invalid zkey Groth16 header size".to_string());
    }
    file.seek(SeekFrom::Start(section.offset))
        .map_err(|error| format!("cannot read zkey Groth16 header: {error}"))?;
    if read_u32(file)? != 32 {
        return Err("zkey BN254 base-field width must be 32 bytes".to_string());
    }
    let mut q = [0u8; 32];
    file.read_exact(&mut q)
        .map_err(|error| format!("truncated zkey base-field modulus: {error}"))?;
    if q.as_slice() != Fq::MODULUS.to_bytes_le() {
        return Err("zkey base-field modulus is not BN254".to_string());
    }
    if read_u32(file)? != 32 {
        return Err("zkey BN254 scalar-field width must be 32 bytes".to_string());
    }
    let mut r = [0u8; 32];
    file.read_exact(&mut r)
        .map_err(|error| format!("truncated zkey scalar-field modulus: {error}"))?;
    if r.as_slice() != Fr::MODULUS.to_bytes_le() {
        return Err("zkey scalar-field modulus is not BN254".to_string());
    }
    let n_vars = read_u32(file)? as usize;
    let n_public = read_u32(file)? as usize;
    let domain_size = read_u32(file)? as usize;
    if n_vars <= n_public || domain_size == 0 || !domain_size.is_power_of_two() {
        return Err("invalid zkey Groth16 dimensions".to_string());
    }
    Ok(ZkeyHeader {
        n_vars,
        n_public,
        domain_size,
    })
}

fn validate_query_sizes(
    sections: &BTreeMap<u32, Section>,
    header: ZkeyHeader,
) -> Result<(), String> {
    let expected = [
        (3, checked_size(header.n_public + 1, 64)?),
        (5, checked_size(header.n_vars, 64)?),
        (6, checked_size(header.n_vars, 64)?),
        (7, checked_size(header.n_vars, 128)?),
        (8, checked_size(header.n_vars - header.n_public - 1, 64)?),
        (9, checked_size(header.domain_size, 64)?),
    ];
    for (id, size) in expected {
        if sections[&id].size != size {
            return Err(format!("invalid zkey section {id} size"));
        }
    }
    Ok(())
}

fn validate_coefficient_section(
    file: &mut File,
    section: Section,
    header: ZkeyHeader,
    expected_constraints: usize,
) -> Result<(), String> {
    if section.size < 4 {
        return Err("truncated zkey coefficient section".to_string());
    }
    file.seek(SeekFrom::Start(section.offset))
        .map_err(|error| format!("cannot read zkey coefficients: {error}"))?;
    let coefficient_count = read_u32(file)? as u64;
    let expected_size = 4u64
        .checked_add(
            coefficient_count
                .checked_mul(44)
                .ok_or_else(|| "zkey coefficient count overflow".to_string())?,
        )
        .ok_or_else(|| "zkey coefficient section overflow".to_string())?;
    if section.size != expected_size {
        return Err("zkey coefficient section length is inconsistent".to_string());
    }
    let mut max_constraint = 0usize;
    for _ in 0..coefficient_count {
        let matrix = read_u32(file)? as usize;
        let constraint = read_u32(file)? as usize;
        let signal = read_u32(file)? as usize;
        if matrix > 1 || constraint >= header.domain_size || signal >= header.n_vars {
            return Err("zkey coefficient index is out of bounds".to_string());
        }
        max_constraint = max_constraint.max(constraint);
        file.seek(SeekFrom::Current(32))
            .map_err(|error| format!("truncated zkey coefficient value: {error}"))?;
    }
    let constraints = max_constraint
        .checked_sub(header.n_public)
        .ok_or_else(|| "zkey coefficient constraints are malformed".to_string())?;
    if constraints != expected_constraints {
        return Err("zkey coefficient count does not match exact R1CS".to_string());
    }
    Ok(())
}

fn validate_key_shape(
    key: &ProvingKey<Bn254>,
    header: ZkeyHeader,
    cs: &ConstraintSystem,
) -> Result<(), String> {
    if key.vk.gamma_abc_g1.len() != cs.num_pub_inputs() + 1
        || key.a_query.len() != header.n_vars
        || key.b_g1_query.len() != header.n_vars
        || key.b_g2_query.len() != header.n_vars
        || key.l_query.len() != header.n_vars - header.n_public - 1
        || key.h_query.len() != header.domain_size
    {
        return Err("parsed zkey query dimensions do not match header".to_string());
    }
    Ok(())
}

fn validate_constraint_binding(
    cs: &ConstraintSystem,
    zkey: ark_relations::r1cs::ConstraintMatrices<Fr>,
) -> Result<(), String> {
    let ark_cs = ArkConstraintSystem::<Fr>::new_ref();
    AchronymeCircuit {
        cs: cs.clone(),
        witness: None,
    }
    .generate_constraints(ark_cs.clone())
    .map_err(|error| format!("cannot synthesize R1CS for key binding: {error}"))?;
    ark_cs.finalize();
    let local = ark_cs
        .to_matrices()
        .ok_or_else(|| "cannot materialize R1CS matrices for key binding".to_string())?;
    if normalized_matrix(local.a) != normalized_matrix(zkey.a)
        || normalized_matrix(local.b) != normalized_matrix(zkey.b)
    {
        return Err("zkey constraint matrices do not match exact R1CS".to_string());
    }
    Ok(())
}

fn normalized_matrix(mut matrix: Vec<Vec<(Fr, usize)>>) -> Vec<Vec<(Fr, usize)>> {
    for row in &mut matrix {
        row.retain(|(coefficient, _)| !coefficient.is_zero());
        row.sort_by_key(|(_, index)| *index);
    }
    matrix
}

fn checked_size(items: usize, item_size: u64) -> Result<u64, String> {
    (items as u64)
        .checked_mul(item_size)
        .ok_or_else(|| "zkey query size overflow".to_string())
}

fn read_u32(reader: &mut File) -> Result<u32, String> {
    let mut bytes = [0u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| format!("truncated zkey integer: {error}"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut File) -> Result<u64, String> {
    let mut bytes = [0u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| format!("truncated zkey integer: {error}"))?;
    Ok(u64::from_le_bytes(bytes))
}
