use std::path::Path;

use anyhow::{Context, Result};
use proving::trusted_setup::{CeremonyContributor, PackageTrustedKey};

#[allow(clippy::too_many_arguments)]
pub fn package(
    r1cs: &str,
    zkey: &str,
    phase1: &str,
    store: &str,
    tool: &str,
    phase1_source: &str,
    phase1_blake2b512: &str,
    contributors: &[String],
    format: &str,
) -> Result<()> {
    let contributors = parse_contributors(contributors)?;
    let packaged = proving::trusted_setup::package_trusted_key(&PackageTrustedKey {
        r1cs: Path::new(r1cs),
        zkey: Path::new(zkey),
        phase1: Path::new(phase1),
        store: Path::new(store),
        tool,
        phase1_source,
        phase1_blake2b512,
        contributors: &contributors,
    })
    .map_err(anyhow::Error::msg)?;

    match format {
        "json" => println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "format": packaged.manifest.format,
                "version": packaged.manifest.version,
                "artifact_dir": packaged.artifact_dir,
                "r1cs_sha256": packaged.manifest.r1cs_sha256,
                "zkey_sha256": packaged.manifest.zkey_sha256,
                "phase1_sha256": packaged.manifest.ceremony.phase1_sha256,
            }))?
        ),
        "text" => {
            println!("trusted key packaged: {}", packaged.artifact_dir.display());
            println!("r1cs sha256: {}", packaged.manifest.r1cs_sha256);
            println!("zkey sha256: {}", packaged.manifest.zkey_sha256);
            println!(
                "phase1 sha256: {}",
                packaged.manifest.ceremony.phase1_sha256
            );
        }
        _ => unreachable!("clap validates trusted-setup output formats"),
    }
    Ok(())
}

fn parse_contributors(values: &[String]) -> Result<Vec<CeremonyContributor>> {
    values
        .iter()
        .map(|value| {
            let (id, contribution_hash) = value
                .split_once('=')
                .context("invalid --contributor value (expected ID=HASH)")?;
            Ok(CeremonyContributor {
                id: id.to_string(),
                contribution_hash: contribution_hash.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributor_ids_may_contain_spaces_but_not_omit_the_hash() {
        let values = vec![format!("independent operator={}", "a".repeat(128))];
        let contributors = parse_contributors(&values).unwrap();
        assert_eq!(contributors[0].id, "independent operator");
        assert_eq!(contributors[0].contribution_hash.len(), 128);
        assert!(parse_contributors(&["missing-hash".to_string()]).is_err());
    }
}
