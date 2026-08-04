use anyhow::Result;
use clap::Parser;
use cli::commands::ErrorFormat;
use cli::config;

/// The compile pipeline allocates and frees millions of small IR nodes
/// and env strings; the system allocator's heap is left fragmented, and
/// every allocation-heavy phase that follows (witness hint walk, R1CS
/// linear elimination, witness replay) runs about 2x slower than on a
/// fresh heap. jemalloc's arena design keeps those phases at fresh-heap
/// speed for the lifetime of the process. MSVC is excluded because
/// tikv-jemallocator does not build there; those builds keep the system
/// allocator.
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;
mod bootstrap;

use args::{Cli, Commands, TrustedSetupCommand};
use bootstrap::{build_overrides, command_start_dir, validate_prime_backend};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // ── Init is self-contained, no config loading needed ──
    if let Commands::Init {
        ref name,
        ref template,
    } = cli.command
    {
        let cwd = std::env::current_dir()?;
        return cli::init::init_project(name, template, &cwd);
    }

    // Detached verification is self-contained and never reads project config.
    if let Commands::Verify {
        proof,
        public,
        vkey,
        curve,
        format,
    } = &cli.command
    {
        return cli::commands::verify::verify_files(proof, public, vkey, curve, format);
    }

    // Trusted-key packaging is self-contained and never reads project config.
    if let Commands::TrustedSetup {
        command:
            TrustedSetupCommand::Package {
                r1cs,
                zkey,
                contributed_zkey,
                phase1,
                store,
                tool,
                phase1_source,
                phase1_blake2b512,
                contributor,
                beacon_source,
                beacon_round,
                beacon_randomness,
                beacon_evidence_sha256,
                beacon_commitment_publication,
                beacon_commitment_sha256,
                beacon_iterations,
                beacon_contribution_hash,
                format,
            },
    } = &cli.command
    {
        return cli::commands::trusted_setup::package(
            r1cs,
            zkey,
            contributed_zkey,
            phase1,
            store,
            tool,
            phase1_source,
            phase1_blake2b512,
            contributor,
            beacon_source,
            *beacon_round,
            beacon_randomness,
            beacon_evidence_sha256,
            beacon_commitment_publication,
            beacon_commitment_sha256,
            *beacon_iterations,
            beacon_contribution_hash,
            format,
        );
    }

    // ── Find and load achronyme.toml (unless --no-config) ──
    let (toml, project_root) = if cli.no_config {
        (None, None)
    } else {
        let start_dir = command_start_dir(&cli.command);
        match config::find_project_toml(&start_dir) {
            Some(toml_path) => {
                let root = toml_path.parent().unwrap().to_path_buf();
                let toml = config::load_toml(&toml_path)?;
                (Some(toml), Some(root))
            }
            None => (None, None),
        }
    };

    // ── Build CLI overrides from command-specific fields ──
    let overrides = build_overrides(&cli);

    // ── Resolve merged config ──
    let cfg = config::resolve_config(&overrides, toml.as_ref(), project_root.as_deref());

    // ── Parse error format ──
    let ef = match cfg.error_format.as_str() {
        "json" => ErrorFormat::Json,
        "short" => ErrorFormat::Short,
        "human" => ErrorFormat::Human,
        other => {
            return Err(anyhow::anyhow!(
                "invalid error-format value: `{other}` (expected `human`, `json`, or `short`)"
            ))
        }
    };

    // ── Parse and validate prime ──
    let prime_id = memory::field::PrimeId::from_name(&cfg.prime).ok_or_else(|| {
        anyhow::anyhow!(
            "invalid prime `{}` (expected \"bn254\", \"bls12-381\", or \"goldilocks\")",
            cfg.prime
        )
    })?;

    // ── Dispatch ──
    match &cli.command {
        Commands::Init { .. } | Commands::Verify { .. } | Commands::TrustedSetup { .. } => {
            unreachable!()
        }

        Commands::Run {
            ptau,
            engine,
            max_instructions,
            ..
        } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            validate_prime_backend(prime_id, &cfg.prove_backend)?;
            let runtime_security = cli::commands::runtime::RuntimeSecurity {
                allow_read: cfg.allow_read.clone(),
                allow_write: cfg.allow_write.clone(),
                allow_connect: cfg.allow_connect.clone(),
                allow_listen: cfg.allow_listen.clone(),
                max_tasks: cfg.max_tasks,
                max_resources: cfg.max_resources,
                max_task_scopes: cfg.max_task_scopes,
                max_pending_native_requests: cfg.max_pending_native_requests,
                max_retained_task_results: cfg.max_retained_task_results,
                max_channels: cfg.max_channels,
                max_channel_operations: cfg.max_channel_operations,
                blocking_workers: cfg.blocking_workers,
                blocking_queue_capacity: cfg.blocking_queue_capacity,
            };
            cli::commands::run::run_file_with_engine_and_key_source(
                path,
                cfg.stress_gc,
                ptau.as_deref(),
                &cfg.prove_backend,
                prime_id,
                cfg.max_heap.as_deref(),
                *max_instructions,
                cfg.gc_stats,
                cfg.circuit_stats,
                ef,
                &cfg.circom_lib_dirs,
                *engine,
                &runtime_security,
                &cfg.proving_key_source,
            )
        }

        Commands::Disassemble { .. } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            cli::commands::disassemble::disassemble_file_with_options(
                path,
                ef,
                prime_id,
                &cfg.circom_lib_dirs,
            )
        }

        Commands::Compile { output, .. } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            let out = output.as_deref().or(cfg.binary_path.as_deref());
            cli::commands::compile::compile_file_with_lib_dirs(
                path,
                out,
                prime_id,
                ef,
                &cfg.circom_lib_dirs,
            )
        }

        Commands::Aot {
            output,
            runtime,
            clang,
            ..
        } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            cli::commands::aot::aot_file(
                path,
                output.as_deref(),
                runtime.as_deref(),
                clang,
                prime_id,
                ef,
                &cfg.circom_lib_dirs,
            )
        }

        Commands::Inspect {
            inputs,
            input_file,
            prove,
            port,
            bind,
            no_open,
            manifest,
            ..
        } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            if *manifest {
                let runtime_security = cli::commands::runtime::RuntimeSecurity {
                    allow_read: cfg.allow_read.clone(),
                    allow_write: cfg.allow_write.clone(),
                    allow_connect: cfg.allow_connect.clone(),
                    allow_listen: cfg.allow_listen.clone(),
                    max_tasks: cfg.max_tasks,
                    max_resources: cfg.max_resources,
                    max_task_scopes: cfg.max_task_scopes,
                    max_pending_native_requests: cfg.max_pending_native_requests,
                    max_retained_task_results: cfg.max_retained_task_results,
                    max_channels: cfg.max_channels,
                    max_channel_operations: cfg.max_channel_operations,
                    blocking_workers: cfg.blocking_workers,
                    blocking_queue_capacity: cfg.blocking_queue_capacity,
                };
                return cli::commands::inspect::inspect_manifest(
                    path,
                    prime_id,
                    &cfg.circom_lib_dirs,
                    &runtime_security,
                    &cfg.proving_key_source,
                    ef,
                );
            }
            cli::commands::inspect::inspect_command(
                path,
                inputs.as_deref(),
                input_file.as_deref(),
                prove.as_deref(),
                *port,
                bind,
                *no_open,
                ef,
            )
        }

        Commands::Circom {
            inputs,
            input_file,
            prove,
            low_memory,
            dump_ir,
            lib_dirs,
            ..
        } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            validate_prime_backend(prime_id, &cfg.backend)?;
            // Merge CLI --lib dirs with [circom].libs from achronyme.toml.
            // CLI dirs take precedence (searched first).
            let mut merged_lib_dirs: Vec<String> = lib_dirs.clone();
            for toml_dir in &cfg.circom_lib_dirs {
                let s = toml_dir.to_string_lossy().into_owned();
                if !merged_lib_dirs.contains(&s) {
                    merged_lib_dirs.push(s);
                }
            }
            cli::commands::circom::circom_command_with_key_source(
                path,
                &cfg.r1cs_path,
                &cfg.wtns_path,
                inputs.as_deref(),
                input_file,
                !cfg.optimize,
                &cfg.backend,
                prime_id,
                *prove,
                *low_memory,
                cfg.solidity_path.as_deref(),
                cfg.plonkish_json_path.as_deref(),
                *dump_ir,
                cfg.circuit_stats,
                &merged_lib_dirs,
                ef,
                &cfg.proving_key_source,
            )
        }

        Commands::Circuit {
            inputs,
            input_file,
            prove,
            dump_ir,
            ..
        } => {
            let path = cfg.entry.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no input file specified and no `entry` in achronyme.toml")
            })?;
            validate_prime_backend(prime_id, &cfg.backend)?;
            cli::commands::circuit::circuit_command_with_key_source(
                path,
                &cfg.r1cs_path,
                &cfg.wtns_path,
                inputs.as_deref(),
                input_file.as_deref(),
                !cfg.optimize,
                &cfg.backend,
                prime_id,
                *prove,
                cfg.solidity_path.as_deref(),
                cfg.plonkish_json_path.as_deref(),
                *dump_ir,
                cfg.circuit_stats,
                ef,
                &cfg.proving_key_source,
            )
        }
    }
}
