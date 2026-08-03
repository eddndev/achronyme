//! Ceremony-derived BN254 Groth16 key loading and proving.

mod artifact;
mod zkey;

use std::path::Path;

use ark_bn254::{Bn254, Fr};
use ark_groth16::{Proof, ProvingKey, VerifyingKey};
use ark_snark::SNARK;
use constraints::r1cs::ConstraintSystem;
use memory::{FieldElement, PrimeId};

use crate::groth16::{fe_to_ark, AchronymeCircuit};

pub use artifact::{
    CeremonyContributor, CeremonyProvenance, TrustedKeyManifest, MANIFEST_FILE, TRANSCRIPT_FILE,
    TRUSTED_KEY_FORMAT, TRUSTED_KEY_VERSION, ZKEY_FILE,
};

pub struct LoadedTrustedKey {
    pub proving_key: ProvingKey<Bn254>,
    pub manifest: TrustedKeyManifest,
}

pub type TrustedProof = (Proof<Bn254>, VerifyingKey<Bn254>, Vec<Fr>);

pub fn r1cs_sha256(cs: &ConstraintSystem) -> String {
    let bytes = constraints::write_r1cs(cs, PrimeId::Bn254);
    artifact::sha256_bytes(&bytes)
}

pub fn load_trusted_key(cs: &ConstraintSystem, store: &Path) -> Result<LoadedTrustedKey, String> {
    let digest = r1cs_sha256(cs);
    let mut artifact = artifact::load(store, cs, &digest)?;
    let proving_key = zkey::parse_and_validate(&mut artifact.zkey_file, cs)?;
    Ok(LoadedTrustedKey {
        proving_key,
        manifest: artifact.manifest,
    })
}

pub fn generate_proof(
    cs: &ConstraintSystem,
    witness: &[FieldElement],
    store: &Path,
) -> Result<TrustedProof, String> {
    let loaded = load_trusted_key(cs, store)?;
    generate_proof_with_key(cs, witness, &loaded)
}

pub fn generate_proof_with_key(
    cs: &ConstraintSystem,
    witness: &[FieldElement],
    loaded: &LoadedTrustedKey,
) -> Result<TrustedProof, String> {
    if witness.len() != cs.num_variables() {
        return Err(format!(
            "witness length {} does not match circuit variable count {}",
            witness.len(),
            cs.num_variables()
        ));
    }
    let vk = loaded.proving_key.vk.clone();
    let circuit = AchronymeCircuit {
        cs: cs.clone(),
        witness: Some(witness.to_vec()),
    };
    let proof = ark_groth16::Groth16::<Bn254, semaphore_rs_ark_circom::CircomReduction>::prove(
        &loaded.proving_key,
        circuit,
        &mut rand::rngs::OsRng,
    )
    .map_err(|error| format!("trusted Groth16 prove failed: {error}"))?;
    let public_inputs = (1..=cs.num_pub_inputs())
        .map(|index| fe_to_ark(&witness[index]))
        .collect::<Vec<Fr>>();
    let valid = ark_groth16::Groth16::<Bn254, semaphore_rs_ark_circom::CircomReduction>::verify(
        &vk,
        &public_inputs,
        &proof,
    )
    .map_err(|error| format!("trusted Groth16 verify failed: {error}"))?;
    if !valid {
        return Err("trusted Groth16 proof verification failed".to_string());
    }
    Ok((proof, vk, public_inputs))
}
