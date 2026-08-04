use super::*;

#[test]
fn circuit_r1cs_with_witness() {
    let tmpdir = tempfile::tempdir().unwrap();
    let r1cs = tmpdir.path().join("out.r1cs");
    let wtns = tmpdir.path().join("out.wtns");

    let result = cli::commands::circuit::circuit_command(
        &fixture("basic_arithmetic.ach"),
        r1cs.to_str().unwrap(),
        wtns.to_str().unwrap(),
        Some("out=42,a=6,b=7"),
        None,
        false,
        "r1cs",
        PrimeId::Bn254,
        false,
        None,
        None,
        false,
        false,
        EF,
    );
    assert!(
        result.is_ok(),
        "circuit with witness failed: {:?}",
        result.err()
    );
    assert!(r1cs.exists(), "R1CS file was not created");
    assert!(wtns.exists(), "wtns file was not created");

    // Verify file sizes are non-trivial
    let r1cs_size = std::fs::metadata(&r1cs).unwrap().len();
    let wtns_size = std::fs::metadata(&wtns).unwrap().len();
    assert!(r1cs_size > 0, "R1CS file is empty");
    assert!(wtns_size > 0, "wtns file is empty");
}

#[test]
fn circuit_plonkish_with_witness() {
    let tmpdir = tempfile::tempdir().unwrap();
    let r1cs = tmpdir.path().join("out.r1cs");
    let wtns = tmpdir.path().join("out.wtns");

    let result = cli::commands::circuit::circuit_command(
        &fixture("basic_arithmetic.ach"),
        r1cs.to_str().unwrap(),
        wtns.to_str().unwrap(),
        Some("out=42,a=6,b=7"),
        None,
        false,
        "plonkish",
        PrimeId::Bn254,
        false,
        None,
        None,
        false,
        false,
        EF,
    );
    assert!(
        result.is_ok(),
        "plonkish with witness failed: {:?}",
        result.err()
    );
}

#[test]
fn circuit_r1cs_wrong_witness_rejected() {
    let tmpdir = tempfile::tempdir().unwrap();
    let r1cs = tmpdir.path().join("out.r1cs");
    let wtns = tmpdir.path().join("out.wtns");

    // out=99 but a*b=42, constraint violation
    let result = cli::commands::circuit::circuit_command(
        &fixture("basic_arithmetic.ach"),
        r1cs.to_str().unwrap(),
        wtns.to_str().unwrap(),
        Some("out=99,a=6,b=7"),
        None,
        false,
        "r1cs",
        PrimeId::Bn254,
        false,
        None,
        None,
        false,
        false,
        EF,
    );
    assert!(result.is_err(), "wrong witness should fail verification");
}

fn assert_r1cs_proof_roundtrip(prime_id: PrimeId) {
    let tmpdir = tempfile::tempdir().unwrap();
    let r1cs = tmpdir.path().join("out.r1cs");
    let wtns = tmpdir.path().join("out.wtns");

    cli::commands::circuit::circuit_command_with_key_source(
        &fixture("basic_arithmetic.ach"),
        r1cs.to_str().unwrap(),
        wtns.to_str().unwrap(),
        Some("out=42,a=6,b=7"),
        None,
        false,
        "r1cs",
        prime_id,
        true,
        None,
        None,
        false,
        false,
        EF,
        &proving::groth16::ProvingKeySource::InsecureLocal,
    )
    .unwrap();

    let proof = std::fs::read_to_string(tmpdir.path().join("proof.json")).unwrap();
    let public = std::fs::read_to_string(tmpdir.path().join("public.json")).unwrap();
    let vkey = std::fs::read_to_string(tmpdir.path().join("vkey.json")).unwrap();
    let valid = match prime_id {
        PrimeId::Bn254 => proving::groth16_bn254::verify_proof_from_json(&proof, &public, &vkey),
        PrimeId::Bls12_381 => {
            proving::groth16_bls12_381::verify_proof_from_json(&proof, &public, &vkey)
        }
        _ => unreachable!(),
    }
    .unwrap();
    assert!(valid);
}

#[test]
fn circuit_r1cs_prove_generates_bn254_artifacts() {
    assert_r1cs_proof_roundtrip(PrimeId::Bn254);
}

#[test]
fn circuit_r1cs_prove_generates_bls12_381_artifacts() {
    assert_r1cs_proof_roundtrip(PrimeId::Bls12_381);
}

#[test]
fn trusted_key_is_preflighted_before_witness_generation() {
    let tmpdir = tempfile::tempdir().unwrap();
    let store = tmpdir.path().join("empty-trusted-store");
    std::fs::create_dir_all(&store).unwrap();
    let result = cli::commands::circuit::circuit_command_with_key_source(
        &fixture("basic_arithmetic.ach"),
        tmpdir.path().join("out.r1cs").to_str().unwrap(),
        tmpdir.path().join("out.wtns").to_str().unwrap(),
        Some("out=42,a=6"),
        None,
        false,
        "r1cs",
        PrimeId::Bn254,
        true,
        None,
        None,
        false,
        false,
        EF,
        &proving::groth16::ProvingKeySource::TrustedStore(store),
    );

    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("trusted-key artifact directory"),
        "unexpected error: {error}"
    );
    assert!(
        !error.contains("missing input"),
        "witness ran first: {error}"
    );
}
