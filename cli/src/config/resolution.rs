use std::path::{Path, PathBuf};

use super::{AchronymeToml, ProjectConfig};

/// CLI values extracted for config resolution.
/// All fields are `Option` — `None` means "not explicitly set by the user".
#[derive(Default)]
pub struct CliOverrides {
    pub path: Option<String>,
    pub error_format: Option<String>,
    pub prime: Option<String>,
    pub backend: Option<String>,
    pub prove_backend: Option<String>,
    pub optimize: Option<bool>,
    pub r1cs_path: Option<String>,
    pub wtns_path: Option<String>,
    pub solidity_path: Option<String>,
    pub plonkish_json_path: Option<String>,
    pub max_heap: Option<String>,
    pub stress_gc: bool,
    pub gc_stats: bool,
    pub circuit_stats: bool,
    pub insecure_dev_setup: bool,
    pub trusted_key_dir: Option<String>,
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub allow_connect: Vec<String>,
    pub allow_listen: Vec<String>,
    pub max_tasks: Option<usize>,
    pub max_resources: Option<usize>,
    pub max_task_scopes: Option<usize>,
    pub max_pending_native_requests: Option<usize>,
    pub max_retained_task_results: Option<usize>,
    pub max_channels: Option<usize>,
    pub max_channel_operations: Option<usize>,
    pub blocking_workers: Option<usize>,
    pub blocking_queue_capacity: Option<usize>,
}

/// Merge CLI overrides + TOML + defaults into a `ProjectConfig`.
pub fn resolve_config(
    cli: &CliOverrides,
    toml: Option<&AchronymeToml>,
    project_root: Option<&Path>,
) -> ProjectConfig {
    let project_name = toml.map(|t| t.project.name.clone());

    // Entry: CLI path > toml entry
    let entry = cli.path.clone().or_else(|| {
        toml.and_then(|t| {
            t.project.entry.as_ref().map(|e| {
                if let Some(root) = project_root {
                    root.join(e).to_string_lossy().into_owned()
                } else {
                    e.clone()
                }
            })
        })
    });

    // Prime: CLI > toml circuit.prime > default "bn254"
    let prime = cli
        .prime
        .clone()
        .or_else(|| toml.and_then(|t| t.circuit.as_ref()?.prime.clone()))
        .unwrap_or_else(|| "bn254".to_string());

    // Backend: CLI > toml > default
    let backend = cli
        .backend
        .clone()
        .or_else(|| toml.and_then(|t| t.build.as_ref()?.backend.clone()))
        .unwrap_or_else(|| "r1cs".to_string());

    // Prove backend: CLI > toml vm.prove_backend > default "r1cs"
    let prove_backend = cli
        .prove_backend
        .clone()
        .or_else(|| toml.and_then(|t| t.vm.as_ref()?.prove_backend.clone()))
        .unwrap_or_else(|| "r1cs".to_string());

    // Optimize: CLI > toml > default (true)
    let optimize = cli
        .optimize
        .or_else(|| toml.and_then(|t| t.build.as_ref()?.optimize))
        .unwrap_or(true);

    // Error format: CLI > toml > default
    let error_format = cli
        .error_format
        .clone()
        .or_else(|| toml.and_then(|t| t.build.as_ref()?.error_format.clone()))
        .unwrap_or_else(|| "human".to_string());

    // Output paths: CLI > toml > defaults
    let r1cs_path = cli
        .r1cs_path
        .clone()
        .or_else(|| toml.and_then(|t| t.build.as_ref()?.output.as_ref()?.r1cs.clone()))
        .unwrap_or_else(|| "circuit.r1cs".to_string());

    let wtns_path = cli
        .wtns_path
        .clone()
        .or_else(|| toml.and_then(|t| t.build.as_ref()?.output.as_ref()?.wtns.clone()))
        .unwrap_or_else(|| "witness.wtns".to_string());

    let binary_path = if cli.path.is_some() {
        None
    } else {
        toml.and_then(|t| {
            let tmpl = t.build.as_ref()?.output.as_ref()?.binary.as_ref()?;
            let name = &t.project.name;
            Some(tmpl.replace("{name}", name))
        })
    };

    let solidity_path = cli.solidity_path.clone().or_else(|| {
        toml.and_then(|t| {
            let p = t.build.as_ref()?.output.as_ref()?.solidity.as_ref()?;
            if p.is_empty() {
                None
            } else {
                Some(p.clone())
            }
        })
    });

    let plonkish_json_path = cli.plonkish_json_path.clone().or_else(|| {
        toml.and_then(|t| {
            let p = t.build.as_ref()?.output.as_ref()?.plonkish_json.as_ref()?;
            if p.is_empty() {
                None
            } else {
                Some(p.clone())
            }
        })
    });

    // VM settings: CLI > toml > defaults
    let max_heap = cli.max_heap.clone().or_else(|| {
        toml.and_then(|t| {
            let mh = t.vm.as_ref()?.max_heap.as_ref()?;
            if mh.is_empty() {
                None
            } else {
                Some(mh.clone())
            }
        })
    });

    let stress_gc = if cli.stress_gc {
        true
    } else {
        toml.and_then(|t| t.vm.as_ref()?.stress_gc).unwrap_or(false)
    };

    let gc_stats = if cli.gc_stats {
        true
    } else {
        toml.and_then(|t| t.vm.as_ref()?.gc_stats).unwrap_or(false)
    };

    let circuit_stats = cli.circuit_stats;

    let resolve_project_path = |raw: &str| {
        let path = PathBuf::from(raw);
        if path.is_absolute() {
            path
        } else if let Some(root) = project_root {
            root.join(path)
        } else {
            path
        }
    };
    let proving = toml.and_then(|value| value.proving.as_ref());
    let proving_key_source = if let Some(path) = cli.trusted_key_dir.as_deref() {
        proving::groth16::ProvingKeySource::TrustedStore(resolve_project_path(path))
    } else if cli.insecure_dev_setup {
        proving::groth16::ProvingKeySource::InsecureLocal
    } else if let Some(path) = proving.and_then(|section| section.trusted_key_dir.as_deref()) {
        proving::groth16::ProvingKeySource::TrustedStore(resolve_project_path(path))
    } else if proving
        .and_then(|section| section.insecure_dev_setup)
        .unwrap_or(false)
    {
        proving::groth16::ProvingKeySource::InsecureLocal
    } else {
        proving::groth16::ProvingKeySource::DenyInsecureSetup
    };

    let resolve_grant_paths = |cli_paths: &[String], toml_paths: Option<&Vec<String>>| {
        let selected = if cli_paths.is_empty() {
            toml_paths.map(Vec::as_slice).unwrap_or_default()
        } else {
            cli_paths
        };
        selected
            .iter()
            .map(|path| {
                let path = PathBuf::from(path);
                if path.is_absolute() {
                    path
                } else if let Some(root) = project_root {
                    root.join(path)
                } else {
                    path
                }
            })
            .collect::<Vec<_>>()
    };
    let vm = toml.and_then(|value| value.vm.as_ref());
    let allow_read = resolve_grant_paths(&cli.allow_read, vm.and_then(|vm| vm.allow_read.as_ref()));
    let allow_write =
        resolve_grant_paths(&cli.allow_write, vm.and_then(|vm| vm.allow_write.as_ref()));
    let allow_connect = if cli.allow_connect.is_empty() {
        vm.and_then(|vm| vm.allow_connect.clone())
            .unwrap_or_default()
    } else {
        cli.allow_connect.clone()
    };
    let allow_listen = if cli.allow_listen.is_empty() {
        vm.and_then(|vm| vm.allow_listen.clone())
            .unwrap_or_default()
    } else {
        cli.allow_listen.clone()
    };
    let max_tasks = cli.max_tasks.or_else(|| vm.and_then(|vm| vm.max_tasks));
    let max_resources = cli
        .max_resources
        .or_else(|| vm.and_then(|vm| vm.max_resources));
    let max_task_scopes = cli
        .max_task_scopes
        .or_else(|| vm.and_then(|vm| vm.max_task_scopes));
    let max_pending_native_requests = cli
        .max_pending_native_requests
        .or_else(|| vm.and_then(|vm| vm.max_pending_native_requests));
    let max_retained_task_results = cli
        .max_retained_task_results
        .or_else(|| vm.and_then(|vm| vm.max_retained_task_results));
    let max_channels = cli
        .max_channels
        .or_else(|| vm.and_then(|vm| vm.max_channels));
    let max_channel_operations = cli
        .max_channel_operations
        .or_else(|| vm.and_then(|vm| vm.max_channel_operations));
    let blocking_workers = cli
        .blocking_workers
        .or_else(|| vm.and_then(|vm| vm.blocking_workers));
    let blocking_queue_capacity = cli
        .blocking_queue_capacity
        .or_else(|| vm.and_then(|vm| vm.blocking_queue_capacity));

    // Circom libs: TOML circom.libs (resolved relative to project root)
    let circom_lib_dirs: Vec<PathBuf> = toml
        .and_then(|t| t.circom.as_ref()?.libs.as_ref())
        .map(|libs| {
            libs.iter()
                .map(|l| {
                    let p = PathBuf::from(l);
                    if p.is_absolute() {
                        p
                    } else if let Some(root) = project_root {
                        root.join(&p)
                    } else {
                        p
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    ProjectConfig {
        project_root: project_root.map(|p| p.to_path_buf()),
        project_name,
        entry,
        prime,
        backend,
        prove_backend,
        optimize,
        error_format,
        r1cs_path,
        wtns_path,
        binary_path,
        solidity_path,
        plonkish_json_path,
        max_heap,
        stress_gc,
        gc_stats,
        circuit_stats,
        proving_key_source,
        allow_read,
        allow_write,
        allow_connect,
        allow_listen,
        max_tasks,
        max_resources,
        max_task_scopes,
        max_pending_native_requests,
        max_retained_task_results,
        max_channels,
        max_channel_operations,
        blocking_workers,
        blocking_queue_capacity,
        circom_lib_dirs,
    }
}

include!("resolution/tests.rs");
