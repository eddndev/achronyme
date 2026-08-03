use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use constraints::{write_wtns, PoseidonParamsProvider};
use memory::field::PrimeId;
use memory::{FieldBackend, FieldElement};
use zkc::witness::WitnessGenerator;

use crate::commands::r1cs_proof::{Groth16Field, PreparedGroth16Key};
use crate::style::{format_number, Styler};

/// Optimized constraints, witness replay, and a preflighted key for repeated
/// proofs of one circuit with independent input sets.
pub(super) struct ReusableProver<F: FieldBackend = memory::Bn254Fr> {
    cs: constraints::r1cs::ConstraintSystem<F>,
    prime_id: PrimeId,
    generator: WitnessGenerator<F>,
    prepared_key: Option<PreparedGroth16Key>,
}

impl<F: FieldBackend> ReusableProver<F> {
    pub(super) fn new(
        cs: constraints::r1cs::ConstraintSystem<F>,
        prime_id: PrimeId,
        generator: WitnessGenerator<F>,
        prepared_key: Option<PreparedGroth16Key>,
    ) -> Self {
        Self {
            cs,
            prime_id,
            generator,
            prepared_key,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_r1cs_repeat<F: FieldBackend + PoseidonParamsProvider + Groth16Field>(
    prover: &ReusableProver<F>,
    all_inputs: &HashMap<String, FieldElement<F>>,
    memo: &mut artik::ArtikMemo<F>,
    label: &str,
    r1cs_path: &str,
    wtns_path: &str,
    prove: bool,
    key_source: &proving::groth16::ProvingKeySource,
    style: &Styler,
    verbose: bool,
) -> Result<()> {
    let witness_vec = prover
        .generator
        .generate_with_memo(all_inputs, memo)
        .map_err(|e| anyhow::anyhow!("witness generation failed for `{label}`: {e}"))?;
    prover
        .cs
        .verify(&witness_vec)
        .map_err(|e| anyhow::anyhow!("witness verification failed for `{label}`: {e}"))?;

    let out_wtns = suffixed_path(wtns_path, label);
    let wtns_data = write_wtns(&witness_vec, prover.prime_id);
    fs::write(&out_wtns, &wtns_data).with_context(|| format!("cannot write {out_wtns}"))?;
    if verbose {
        eprintln!(
            "{} `{}` - wrote {} ({} values) {} {}",
            style.success("Reused circuit for"),
            label,
            style.bold(&out_wtns),
            format_number(witness_vec.len()),
            style.dim("-"),
            style.green("verified OK")
        );
    } else {
        eprintln!(
            "wrote {} ({} values) - verified OK",
            out_wtns,
            witness_vec.len()
        );
    }

    if prove {
        let cache_dir = crate::cache_dir();
        let result = F::generate_groth16_proof(
            &prover.cs,
            &witness_vec,
            &cache_dir,
            key_source,
            prover.prepared_key.as_ref(),
        )
        .map_err(|e| anyhow::anyhow!("proof generation failed for `{label}`: {e}"))?;

        if let akron::ProveResult::Proof {
            proof_json,
            public_json,
            ..
        } = result
        {
            let out_dir = std::path::Path::new(r1cs_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let proof_path = out_dir.join(format!("proof.{label}.json"));
            let public_path = out_dir.join(format!("public.{label}.json"));
            fs::write(&proof_path, proof_json)?;
            fs::write(&public_path, public_json)?;
            if verbose {
                eprintln!(
                    "    Wrote {} and {}",
                    style.bold(&proof_path.display().to_string()),
                    style.bold(&public_path.display().to_string())
                );
            } else {
                eprintln!("wrote {}, {}", proof_path.display(), public_path.display());
            }
        }
    }

    Ok(())
}

fn suffixed_path(base: &str, label: &str) -> String {
    let path = std::path::Path::new(base);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let name = if extension.is_empty() {
        format!("{stem}.{label}")
    } else {
        format!("{stem}.{label}.{extension}")
    };
    path.with_file_name(name).to_string_lossy().into_owned()
}
