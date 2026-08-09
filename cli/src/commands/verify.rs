use std::path::Path;

use anyhow::{anyhow, Result};

const MAX_JSON_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;

pub fn verify_files(
    proof_path: &str,
    public_path: &str,
    vkey_path: &str,
    curve: &str,
    format: &str,
) -> Result<()> {
    let result = verify_files_inner(proof_path, public_path, vkey_path, curve);
    match result {
        Ok(valid) => {
            print_result(curve, format, valid, None);
            if valid {
                Ok(())
            } else {
                Err(anyhow!("proof is invalid for curve `{curve}`"))
            }
        }
        Err(error) => {
            if format == "json" {
                print_result(curve, format, false, Some(&error));
            }
            Err(anyhow!(
                "proof verification failed for curve `{curve}`: {error}"
            ))
        }
    }
}

fn verify_files_inner(
    proof_path: &str,
    public_path: &str,
    vkey_path: &str,
    curve: &str,
) -> Result<bool, String> {
    let proof = read_json_artifact(Path::new(proof_path), "proof")?;
    let public = read_json_artifact(Path::new(public_path), "public inputs")?;
    let vkey = read_json_artifact(Path::new(vkey_path), "verification key")?;

    match curve {
        "bn254" => proving::groth16_bn254::verify_proof_from_json(&proof, &public, &vkey),
        "bls12-381" => proving::groth16_bls12_381::verify_proof_from_json(&proof, &public, &vkey),
        _ => Err(format!("unsupported verification curve `{curve}`")),
    }
}

fn read_json_artifact(path: &Path, label: &str) -> Result<String, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {label} `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} `{}` must be a regular file, not a symlink",
            path.display()
        ));
    }
    if metadata.len() > MAX_JSON_ARTIFACT_BYTES {
        return Err(format!(
            "{label} `{}` exceeds {MAX_JSON_ARTIFACT_BYTES} bytes",
            path.display()
        ));
    }
    std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read {label} `{}`: {error}", path.display()))
}

fn print_result(curve: &str, format: &str, valid: bool, error: Option<&str>) {
    if format == "json" {
        let mut result = serde_json::json!({
            "curve": curve,
            "valid": valid,
        });
        if let Some(error) = error {
            result["error"] = error.into();
        }
        println!("{result}");
    } else if valid {
        println!("proof valid ({curve})");
    } else {
        println!("proof invalid ({curve})");
    }
}
