use anyhow::Result;
use cli::config::CliOverrides;
use memory::field::PrimeId;

use crate::args::{Cli, Commands};

/// Determine the starting directory for toml walk-up search.
pub(super) fn command_start_dir(cmd: &Commands) -> std::path::PathBuf {
    let path_arg = match cmd {
        Commands::Run { path, .. }
        | Commands::Disassemble { path }
        | Commands::Compile { path, .. }
        | Commands::Aot { path, .. }
        | Commands::Inspect { path, .. }
        | Commands::Circuit { path, .. }
        | Commands::Circom { path, .. } => path.as_deref(),
        Commands::Init { .. } | Commands::Verify { .. } => None,
    };

    if let Some(p) = path_arg {
        let p = std::path::Path::new(p);
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                return parent.to_path_buf();
            }
        }
    }

    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Extract CLI overrides from parsed arguments.
pub(super) fn build_overrides(cli: &Cli) -> CliOverrides {
    match &cli.command {
        Commands::Run {
            path,
            stress_gc,
            prove_backend,
            max_heap,
            gc_stats,
            circuit_stats,
            ..
        } => CliOverrides {
            path: path.clone(),
            error_format: cli.error_format.clone(),
            prime: cli.prime.clone(),
            backend: None,
            prove_backend: prove_backend.clone(),
            optimize: None,
            r1cs_path: None,
            wtns_path: None,
            solidity_path: None,
            plonkish_json_path: None,
            max_heap: max_heap.clone(),
            stress_gc: *stress_gc,
            gc_stats: *gc_stats,
            circuit_stats: *circuit_stats,
            insecure_dev_setup: cli.insecure_dev_setup,
            trusted_key_dir: cli.trusted_key_dir.clone(),
            allow_read: cli.allow_read.clone(),
            allow_write: cli.allow_write.clone(),
            allow_connect: cli.allow_connect.clone(),
            allow_listen: cli.allow_listen.clone(),
            max_tasks: cli.max_tasks,
            max_resources: cli.max_resources,
            max_task_scopes: cli.max_task_scopes,
            max_pending_native_requests: cli.max_pending_native_requests,
            max_retained_task_results: cli.max_retained_task_results,
            max_channels: cli.max_channels,
            max_channel_operations: cli.max_channel_operations,
            blocking_workers: cli.blocking_workers,
            blocking_queue_capacity: cli.blocking_queue_capacity,
        },

        Commands::Disassemble { path } => CliOverrides {
            path: path.clone(),
            error_format: cli.error_format.clone(),
            prime: cli.prime.clone(),
            backend: None,
            prove_backend: None,
            optimize: None,
            r1cs_path: None,
            wtns_path: None,
            solidity_path: None,
            plonkish_json_path: None,
            max_heap: None,
            stress_gc: false,
            gc_stats: false,
            circuit_stats: false,
            insecure_dev_setup: cli.insecure_dev_setup,
            trusted_key_dir: cli.trusted_key_dir.clone(),
            ..CliOverrides::default()
        },

        Commands::Compile { path, .. } | Commands::Aot { path, .. } => CliOverrides {
            path: path.clone(),
            error_format: cli.error_format.clone(),
            prime: cli.prime.clone(),
            backend: None,
            prove_backend: None,
            optimize: None,
            r1cs_path: None,
            wtns_path: None,
            solidity_path: None,
            plonkish_json_path: None,
            max_heap: None,
            stress_gc: false,
            gc_stats: false,
            circuit_stats: false,
            insecure_dev_setup: cli.insecure_dev_setup,
            trusted_key_dir: cli.trusted_key_dir.clone(),
            ..CliOverrides::default()
        },

        Commands::Circuit {
            path,
            r1cs,
            wtns,
            backend,
            no_optimize,
            solidity,
            plonkish_json,
            circuit_stats,
            ..
        } => CliOverrides {
            path: path.clone(),
            error_format: cli.error_format.clone(),
            prime: cli.prime.clone(),
            backend: backend.clone(),
            prove_backend: None,
            optimize: no_optimize.map(|no| !no),
            r1cs_path: r1cs.clone(),
            wtns_path: wtns.clone(),
            solidity_path: solidity.clone(),
            plonkish_json_path: plonkish_json.clone(),
            max_heap: None,
            stress_gc: false,
            gc_stats: false,
            circuit_stats: *circuit_stats,
            insecure_dev_setup: cli.insecure_dev_setup,
            trusted_key_dir: cli.trusted_key_dir.clone(),
            ..CliOverrides::default()
        },

        Commands::Inspect { path, .. } => CliOverrides {
            path: path.clone(),
            error_format: cli.error_format.clone(),
            prime: cli.prime.clone(),
            backend: None,
            prove_backend: None,
            optimize: None,
            r1cs_path: None,
            wtns_path: None,
            solidity_path: None,
            plonkish_json_path: None,
            max_heap: None,
            stress_gc: false,
            gc_stats: false,
            circuit_stats: false,
            insecure_dev_setup: cli.insecure_dev_setup,
            trusted_key_dir: cli.trusted_key_dir.clone(),
            allow_read: cli.allow_read.clone(),
            allow_write: cli.allow_write.clone(),
            allow_connect: cli.allow_connect.clone(),
            allow_listen: cli.allow_listen.clone(),
            max_tasks: cli.max_tasks,
            max_resources: cli.max_resources,
            max_task_scopes: cli.max_task_scopes,
            max_pending_native_requests: cli.max_pending_native_requests,
            max_retained_task_results: cli.max_retained_task_results,
            max_channels: cli.max_channels,
            max_channel_operations: cli.max_channel_operations,
            blocking_workers: cli.blocking_workers,
            blocking_queue_capacity: cli.blocking_queue_capacity,
        },

        Commands::Circom {
            path,
            backend,
            no_optimize,
            r1cs,
            wtns,
            solidity,
            plonkish_json,
            circuit_stats,
            ..
        } => CliOverrides {
            path: path.clone(),
            error_format: cli.error_format.clone(),
            prime: cli.prime.clone(),
            backend: backend.clone(),
            prove_backend: None,
            optimize: no_optimize.map(|no| !no),
            r1cs_path: r1cs.clone(),
            wtns_path: wtns.clone(),
            solidity_path: solidity.clone(),
            plonkish_json_path: plonkish_json.clone(),
            max_heap: None,
            stress_gc: false,
            gc_stats: false,
            circuit_stats: *circuit_stats,
            insecure_dev_setup: cli.insecure_dev_setup,
            trusted_key_dir: cli.trusted_key_dir.clone(),
            ..CliOverrides::default()
        },

        Commands::Init { .. } | Commands::Verify { .. } => unreachable!(),
    }
}

/// Validate that the (prime, backend) combination is supported.
///
/// Goldilocks+r1cs is allowed for constraint generation and witness, but
/// proof generation will fail at runtime (no pairing-friendly prover).
pub(super) fn validate_prime_backend(prime_id: PrimeId, backend: &str) -> Result<()> {
    match (prime_id, backend) {
        (PrimeId::Bn254, "r1cs") => Ok(()),      // groth16-bn254
        (PrimeId::Bn254, "plonkish") => Ok(()),  // plonk-bn254
        (PrimeId::Bls12_381, "r1cs") => Ok(()),  // groth16-bls12-381
        (PrimeId::Goldilocks, "r1cs") => Ok(()), // constraints only (no prover)
        _ => Err(anyhow::anyhow!(
            "unsupported combination: prime `{}` with backend `{backend}`\n  \
             Supported: bn254+r1cs, bn254+plonkish, bls12-381+r1cs, goldilocks+r1cs",
            prime_id.name()
        )),
    }
}
