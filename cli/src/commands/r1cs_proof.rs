use std::path::Path;

use constraints::r1cs::ConstraintSystem;
use memory::{FieldBackend, FieldElement};

pub(crate) enum PreparedGroth16Key {
    Bn254(proving::trusted_setup::LoadedTrustedKey),
}

pub(crate) trait Groth16Field: FieldBackend + Sized {
    fn prepare_groth16_key(
        cs: &ConstraintSystem<Self>,
        key_source: &proving::groth16::ProvingKeySource,
    ) -> Result<Option<PreparedGroth16Key>, String>;

    fn generate_groth16_proof(
        cs: &ConstraintSystem<Self>,
        witness: &[FieldElement<Self>],
        cache_dir: &Path,
        key_source: &proving::groth16::ProvingKeySource,
        prepared_key: Option<&PreparedGroth16Key>,
    ) -> Result<akron::ProveResult, String>;
}

impl Groth16Field for memory::Bn254Fr {
    fn prepare_groth16_key(
        cs: &ConstraintSystem<Self>,
        key_source: &proving::groth16::ProvingKeySource,
    ) -> Result<Option<PreparedGroth16Key>, String> {
        match key_source {
            proving::groth16::ProvingKeySource::TrustedStore(store) => {
                proving::trusted_setup::load_trusted_key(cs, store)
                    .map(PreparedGroth16Key::Bn254)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }

    fn generate_groth16_proof(
        cs: &ConstraintSystem<Self>,
        witness: &[FieldElement<Self>],
        cache_dir: &Path,
        key_source: &proving::groth16::ProvingKeySource,
        prepared_key: Option<&PreparedGroth16Key>,
    ) -> Result<akron::ProveResult, String> {
        match (key_source, prepared_key) {
            (
                proving::groth16::ProvingKeySource::TrustedStore(_),
                Some(PreparedGroth16Key::Bn254(key)),
            ) => proving::groth16_bn254::generate_proof_with_loaded_trusted_key(cs, witness, key),
            (proving::groth16::ProvingKeySource::TrustedStore(_), None) => {
                Err("trusted proving key was not preflighted".to_string())
            }
            (_, None) => proving::groth16_bn254::generate_proof(cs, witness, cache_dir, key_source),
            (_, Some(_)) => Err("prepared proving key does not match key policy".to_string()),
        }
    }
}

impl Groth16Field for memory::Bls12_381Fr {
    fn prepare_groth16_key(
        _cs: &ConstraintSystem<Self>,
        key_source: &proving::groth16::ProvingKeySource,
    ) -> Result<Option<PreparedGroth16Key>, String> {
        match key_source {
            proving::groth16::ProvingKeySource::TrustedStore(_) => {
                Err("trusted zkey stores currently support only BN254".to_string())
            }
            _ => Ok(None),
        }
    }

    fn generate_groth16_proof(
        cs: &ConstraintSystem<Self>,
        witness: &[FieldElement<Self>],
        cache_dir: &Path,
        key_source: &proving::groth16::ProvingKeySource,
        prepared_key: Option<&PreparedGroth16Key>,
    ) -> Result<akron::ProveResult, String> {
        if prepared_key.is_some() {
            return Err("prepared proving key does not match BLS12-381".to_string());
        }
        proving::groth16_bls12_381::generate_proof(cs, witness, cache_dir, key_source)
    }
}

impl Groth16Field for memory::GoldilocksFr {
    fn prepare_groth16_key(
        _cs: &ConstraintSystem<Self>,
        _key_source: &proving::groth16::ProvingKeySource,
    ) -> Result<Option<PreparedGroth16Key>, String> {
        Err("Groth16 proving is not supported for goldilocks".to_string())
    }

    fn generate_groth16_proof(
        _cs: &ConstraintSystem<Self>,
        _witness: &[FieldElement<Self>],
        _cache_dir: &Path,
        _key_source: &proving::groth16::ProvingKeySource,
        _prepared_key: Option<&PreparedGroth16Key>,
    ) -> Result<akron::ProveResult, String> {
        Err("Groth16 proving is not supported for goldilocks".to_string())
    }
}
