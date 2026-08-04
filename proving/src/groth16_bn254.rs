//! BN254-specific Groth16 proof generation and snarkjs-compatible serialization.
//!
//! Delegates proof generation to the generic `groth16` module, then applies
//! BN254-specific JSON serialization (snarkjs format) and Solidity calldata
//! formatting (EIP-197 coordinate order).

use std::path::Path;

use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ec::pairing::Pairing;

use akron::ProveResult;
use constraints::r1cs::ConstraintSystem;
use memory::FieldElement;

use crate::groth16;

// ============================================================================
// Public API (BN254-specialized wrappers)
// ============================================================================

/// Run trusted setup (or load cached keys) for BN254 Groth16.
pub fn setup_keys(
    cs: &ConstraintSystem,
    cache_dir: &Path,
    key_source: &groth16::ProvingKeySource,
) -> Result<
    (
        ark_groth16::ProvingKey<Bn254>,
        ark_groth16::VerifyingKey<Bn254>,
    ),
    String,
> {
    match key_source {
        groth16::ProvingKeySource::TrustedStore(store) => {
            let loaded = crate::trusted_setup::load_trusted_key(cs, store)?;
            let vk = loaded.proving_key.vk.clone();
            Ok((loaded.proving_key, vk))
        }
        _ => groth16::setup_keys::<_, Bn254>(cs, cache_dir, "bn254", key_source),
    }
}

/// Run trusted setup and return only the verifying key (BN254).
pub fn setup_vk_only(
    cs: &ConstraintSystem,
    cache_dir: &Path,
    key_source: &groth16::ProvingKeySource,
) -> Result<ark_groth16::VerifyingKey<Bn254>, String> {
    match key_source {
        groth16::ProvingKeySource::TrustedStore(store) => {
            Ok(crate::trusted_setup::load_trusted_key(cs, store)?
                .proving_key
                .vk)
        }
        _ => groth16::setup_vk_only::<_, Bn254>(cs, cache_dir, "bn254", key_source),
    }
}

/// Generate a BN254 Groth16 proof with snarkjs-compatible JSON output.
pub fn generate_proof(
    cs: &ConstraintSystem,
    witness: &[FieldElement],
    cache_dir: &Path,
    key_source: &groth16::ProvingKeySource,
) -> Result<ProveResult, String> {
    let (proof, vk, public_inputs) = match key_source {
        groth16::ProvingKeySource::TrustedStore(store) => {
            crate::trusted_setup::generate_proof(cs, witness, store)?
        }
        _ => groth16::generate_proof_raw::<_, Bn254>(cs, witness, cache_dir, "bn254", key_source)?,
    };

    Ok(serialize_proof_result(cs, &proof, &vk, &public_inputs))
}

pub fn generate_proof_with_loaded_trusted_key(
    cs: &ConstraintSystem,
    witness: &[FieldElement],
    loaded: &crate::trusted_setup::LoadedTrustedKey,
) -> Result<ProveResult, String> {
    let (proof, vk, public_inputs) =
        crate::trusted_setup::generate_proof_with_key(cs, witness, loaded)?;
    Ok(serialize_proof_result(cs, &proof, &vk, &public_inputs))
}

fn serialize_proof_result(
    cs: &ConstraintSystem,
    proof: &ark_groth16::Proof<Bn254>,
    vk: &ark_groth16::VerifyingKey<Bn254>,
    public_inputs: &[Fr],
) -> ProveResult {
    let proof_json = serialize_proof_json(proof);
    let public_json = serialize_public_json(public_inputs);
    let vkey_json = serialize_vkey_json(vk, cs.num_pub_inputs());

    ProveResult::Proof {
        proof_json,
        public_json,
        vkey_json,
    }
}

// ============================================================================
// JSON serialization (snarkjs-compatible, BN254)
// ============================================================================

/// Format a G1 affine point as a JSON array of 3 decimal strings [x, y, "1"].
fn g1_to_json(p: &<Bn254 as Pairing>::G1Affine) -> serde_json::Value {
    use ark_ec::AffineRepr;
    if p.is_zero() {
        return serde_json::json!(["0", "1", "0"]);
    }
    let x = p.x().expect("non-zero point has x");
    let y = p.y().expect("non-zero point has y");
    serde_json::json!([groth16::fr_to_decimal(&x), groth16::fr_to_decimal(&y), "1"])
}

/// Format a G2 affine point as a JSON array of 3 arrays, each with 2 decimal strings.
fn g2_to_json(p: &<Bn254 as Pairing>::G2Affine) -> serde_json::Value {
    use ark_ec::AffineRepr;
    if p.is_zero() {
        return serde_json::json!([["0", "0"], ["1", "0"], ["0", "0"]]);
    }
    let x = p.x().expect("non-zero point has x");
    let y = p.y().expect("non-zero point has y");
    serde_json::json!([
        [groth16::fr_to_decimal(&x.c0), groth16::fr_to_decimal(&x.c1)],
        [groth16::fr_to_decimal(&y.c0), groth16::fr_to_decimal(&y.c1)],
        ["1", "0"]
    ])
}

fn serialize_proof_json(proof: &ark_groth16::Proof<Bn254>) -> String {
    let obj = serde_json::json!({
        "pi_a": g1_to_json(&proof.a),
        "pi_b": g2_to_json(&proof.b),
        "pi_c": g1_to_json(&proof.c),
        "protocol": "groth16",
        "curve": "bn128"
    });
    serde_json::to_string_pretty(&obj).unwrap()
}

fn serialize_public_json(inputs: &[Fr]) -> String {
    let arr: Vec<String> = inputs.iter().map(groth16::fr_to_decimal).collect();
    serde_json::to_string_pretty(&arr).unwrap()
}

fn serialize_vkey_json(vk: &ark_groth16::VerifyingKey<Bn254>, num_pub: usize) -> String {
    let mut ic: Vec<serde_json::Value> = Vec::new();
    for p in &vk.gamma_abc_g1 {
        ic.push(g1_to_json(p));
    }
    let obj = serde_json::json!({
        "protocol": "groth16",
        "curve": "bn128",
        "nPublic": num_pub,
        "vk_alpha_1": g1_to_json(&vk.alpha_g1),
        "vk_beta_2": g2_to_json(&vk.beta_g2),
        "vk_gamma_2": g2_to_json(&vk.gamma_g2),
        "vk_delta_2": g2_to_json(&vk.delta_g2),
        "IC": ic
    });
    serde_json::to_string_pretty(&obj).unwrap()
}

// ============================================================================
// Solidity calldata formatting (BN254, EIP-197)
// ============================================================================

/// Format a Groth16 proof and public inputs as Solidity calldata strings.
///
/// Applies EIP-197 coordinate swaps for G2 points (π_B):
/// arkworks Fq2(c0=real, c1=imag) → EVM (c1=imag, c0=real).
pub fn format_solidity_calldata(
    proof: &ark_groth16::Proof<Bn254>,
    public_inputs: &[Fr],
) -> SolidityCalldata {
    use ark_ec::AffineRepr;

    let a = proof.a;
    let b = proof.b;
    let c = proof.c;

    let p_a = [
        groth16::fr_to_decimal(&a.x().expect("a.x")),
        groth16::fr_to_decimal(&a.y().expect("a.y")),
    ];

    // π_B (G2): SWAP c0 ↔ c1 for EIP-197
    let bx = b.x().expect("b.x");
    let by = b.y().expect("b.y");
    let p_b = [
        [
            groth16::fr_to_decimal(&bx.c1),
            groth16::fr_to_decimal(&bx.c0),
        ],
        [
            groth16::fr_to_decimal(&by.c1),
            groth16::fr_to_decimal(&by.c0),
        ],
    ];

    let p_c = [
        groth16::fr_to_decimal(&c.x().expect("c.x")),
        groth16::fr_to_decimal(&c.y().expect("c.y")),
    ];

    let pub_signals: Vec<String> = public_inputs.iter().map(groth16::fr_to_decimal).collect();

    SolidityCalldata {
        p_a,
        p_b,
        p_c,
        pub_signals,
    }
}

/// Structured Solidity calldata for a Groth16 proof.
pub struct SolidityCalldata {
    pub p_a: [String; 2],
    pub p_b: [[String; 2]; 2],
    pub p_c: [String; 2],
    pub pub_signals: Vec<String>,
}

// ============================================================================
// JSON deserialization (for verify_proof, BN254)
// ============================================================================

fn decimal_to_fr(s: &str) -> Result<Fr, String> {
    use std::str::FromStr;
    Fr::from_str(s).map_err(|_| format!("invalid field element: {s}"))
}

fn decimal_to_fq(s: &str) -> Result<Fq, String> {
    use std::str::FromStr;
    Fq::from_str(s).map_err(|_| format!("invalid base field element: {s}"))
}

fn json_to_g1(val: &serde_json::Value) -> Result<G1Affine, String> {
    let arr = val.as_array().ok_or("expected array for G1 point")?;
    if arr.len() != 3 {
        return Err("G1 point must have 3 elements".into());
    }
    let x_str = arr[0].as_str().ok_or("G1 x must be string")?;
    let y_str = arr[1].as_str().ok_or("G1 y must be string")?;
    let z_str = arr[2].as_str().ok_or("G1 z must be string")?;

    if z_str == "0" {
        use ark_ec::AffineRepr;
        return Ok(G1Affine::zero());
    }
    if z_str != "1" {
        return Err("G1 z must be 0 or 1".to_string());
    }

    let x = decimal_to_fq(x_str)?;
    let y = decimal_to_fq(y_str)?;
    let point = G1Affine::new_unchecked(x, y);
    if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err("G1 point is not in the BN254 prime-order subgroup".to_string());
    }
    Ok(point)
}

fn json_to_g2(val: &serde_json::Value) -> Result<G2Affine, String> {
    let arr = val.as_array().ok_or("expected array for G2 point")?;
    if arr.len() != 3 {
        return Err("G2 point must have 3 elements".into());
    }

    let x_arr = arr[0].as_array().ok_or("G2 x must be array")?;
    let y_arr = arr[1].as_array().ok_or("G2 y must be array")?;
    let z_arr = arr[2].as_array().ok_or("G2 z must be array")?;

    if x_arr.len() != 2 {
        return Err("G2 x must have 2 elements".to_string());
    }
    if y_arr.len() != 2 {
        return Err("G2 y must have 2 elements".to_string());
    }
    if z_arr.len() != 2 {
        return Err("G2 z must have 2 elements".to_string());
    }

    let z0 = z_arr[0].as_str().ok_or("G2 z.c0 must be string")?;
    let z1 = z_arr[1].as_str().ok_or("G2 z.c1 must be string")?;
    if z0 == "0" && z1 == "0" {
        use ark_ec::AffineRepr;
        return Ok(G2Affine::zero());
    }
    if z0 != "1" || z1 != "0" {
        return Err("G2 z must be [\"0\",\"0\"] or [\"1\",\"0\"]".to_string());
    }

    let x_c0 = decimal_to_fq(x_arr[0].as_str().ok_or("x.c0 must be string")?)?;
    let x_c1 = decimal_to_fq(x_arr[1].as_str().ok_or("x.c1 must be string")?)?;
    let y_c0 = decimal_to_fq(y_arr[0].as_str().ok_or("y.c0 must be string")?)?;
    let y_c1 = decimal_to_fq(y_arr[1].as_str().ok_or("y.c1 must be string")?)?;

    let x = Fq2::new(x_c0, x_c1);
    let y = Fq2::new(y_c0, y_c1);
    let point = G2Affine::new_unchecked(x, y);
    if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err("G2 point is not in the BN254 prime-order subgroup".to_string());
    }
    Ok(point)
}

/// Deserialize a snarkjs-format proof JSON string into an ark Proof.
pub fn deserialize_proof_json(json_str: &str) -> Result<ark_groth16::Proof<Bn254>, String> {
    let obj: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("invalid proof JSON: {e}"))?;
    crate::json_artifact::validate_groth16_metadata(&obj, "proof", &["bn128", "bn254"])?;
    let a = json_to_g1(&obj["pi_a"])?;
    let b = json_to_g2(&obj["pi_b"])?;
    let c = json_to_g1(&obj["pi_c"])?;
    Ok(ark_groth16::Proof { a, b, c })
}

/// Deserialize a snarkjs-format public inputs JSON string into ark Fr values.
pub fn deserialize_public_json(json_str: &str) -> Result<Vec<Fr>, String> {
    let arr: Vec<String> =
        serde_json::from_str(json_str).map_err(|e| format!("invalid public JSON: {e}"))?;
    arr.iter().map(|s| decimal_to_fr(s)).collect()
}

/// Deserialize a snarkjs-format verifying key JSON string into an ark VerifyingKey.
pub fn deserialize_vkey_json(json_str: &str) -> Result<ark_groth16::VerifyingKey<Bn254>, String> {
    let obj: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("invalid vkey JSON: {e}"))?;
    crate::json_artifact::validate_groth16_metadata(&obj, "vkey", &["bn128", "bn254"])?;

    let alpha_g1 = json_to_g1(&obj["vk_alpha_1"])?;
    let beta_g2 = json_to_g2(&obj["vk_beta_2"])?;
    let gamma_g2 = json_to_g2(&obj["vk_gamma_2"])?;
    let delta_g2 = json_to_g2(&obj["vk_delta_2"])?;

    let ic_arr = obj["IC"].as_array().ok_or("vkey IC must be an array")?;
    let declared_public = crate::json_artifact::declared_public_inputs(&obj)?;
    if ic_arr.len() != declared_public + 1 {
        return Err("vkey nPublic does not match IC length".to_string());
    }
    let mut gamma_abc_g1 = Vec::with_capacity(ic_arr.len());
    for ic in ic_arr {
        gamma_abc_g1.push(json_to_g1(ic)?);
    }

    Ok(ark_groth16::VerifyingKey {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        gamma_abc_g1,
    })
}

/// Verify a proof using deserialized JSON components (BN254).
pub fn verify_proof_from_json(
    proof_json: &str,
    public_json: &str,
    vkey_json: &str,
) -> Result<bool, String> {
    use ark_groth16::Groth16;
    use ark_snark::SNARK;
    let proof = deserialize_proof_json(proof_json)?;
    let public_inputs = deserialize_public_json(public_json)?;
    let vk = deserialize_vkey_json(vkey_json)?;
    if vk.gamma_abc_g1.len() != public_inputs.len() + 1 {
        return Err("verification key public-input count does not match public.json".to_string());
    }
    Groth16::<Bn254>::verify(&vk, &public_inputs, &proof)
        .map_err(|e| format!("verification error: {e}"))
}
