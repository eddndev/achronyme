#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{load_toml, TOML_FILENAME};
    use std::fs;

    #[test]
    fn resolve_cli_overrides_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            "[project]\nname = \"t\"\nversion = \"0.1.0\"\n\n[build]\nbackend = \"plonkish\"\n",
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
            backend: Some("r1cs".to_string()),
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, Some(&toml), Some(tmp.path()));
        assert_eq!(config.backend, "r1cs"); // CLI wins
    }

    #[test]
    fn resolve_defaults_no_toml() {
        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, None, None);
        assert_eq!(config.prime, "bn254"); // default
        assert_eq!(config.backend, "r1cs");
        assert!(config.optimize);
        assert_eq!(config.error_format, "human");
        assert_eq!(config.r1cs_path, "circuit.r1cs");
        assert_eq!(config.wtns_path, "witness.wtns");
        assert_eq!(
            config.proving_key_source,
            proving::groth16::ProvingKeySource::DenyInsecureSetup
        );
    }

    #[test]
    fn resolve_insecure_development_setup_requires_cli_opt_in() {
        let cli = CliOverrides {
            insecure_dev_setup: true,
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, None, None);
        assert_eq!(
            config.proving_key_source,
            proving::groth16::ProvingKeySource::InsecureLocal
        );
    }

    #[test]
    fn resolve_trusted_key_store_relative_to_project_root() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            concat!(
                "[project]\nname = \"t\"\nversion = \"0.1.0\"\n\n",
                "[proving]\ntrusted_key_dir = \"ceremony/keys\"\n"
            ),
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let config = resolve_config(&CliOverrides::default(), Some(&toml), Some(tmp.path()));
        assert_eq!(
            config.proving_key_source,
            proving::groth16::ProvingKeySource::TrustedStore(
                tmp.path().join("ceremony/keys")
            )
        );
    }

    #[test]
    fn resolve_prime_from_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            "[project]\nname = \"t\"\nversion = \"0.1.0\"\n\n[circuit]\nprime = \"goldilocks\"\n",
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, Some(&toml), Some(tmp.path()));
        assert_eq!(config.prime, "goldilocks");
    }

    #[test]
    fn resolve_prime_cli_overrides_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            "[project]\nname = \"t\"\nversion = \"0.1.0\"\n\n[circuit]\nprime = \"goldilocks\"\n",
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: Some("bls12-381".to_string()),
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, Some(&toml), Some(tmp.path()));
        assert_eq!(config.prime, "bls12-381"); // CLI wins
    }

    #[test]
    fn resolve_entry_from_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            "[project]\nname = \"t\"\nversion = \"0.1.0\"\nentry = \"src/main.ach\"\n",
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, Some(&toml), Some(tmp.path()));
        let expected = tmp
            .path()
            .join("src/main.ach")
            .to_string_lossy()
            .into_owned();
        assert_eq!(config.entry.unwrap(), expected);
    }

    #[test]
    fn resolve_circom_libs_from_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            "[project]\nname = \"t\"\nversion = \"0.1.0\"\n\n[circom]\nlibs = [\"vendor/circomlib/circuits\"]\n",
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, Some(&toml), Some(tmp.path()));
        assert_eq!(config.circom_lib_dirs.len(), 1);
        assert_eq!(
            config.circom_lib_dirs[0],
            tmp.path().join("vendor/circomlib/circuits")
        );
    }

    #[test]
    fn resolve_circom_libs_empty_without_section() {
        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, None, None);
        assert!(config.circom_lib_dirs.is_empty());
    }

    #[test]
    fn resolve_name_template_in_binary_path() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            "[project]\nname = \"foo\"\nversion = \"0.1.0\"\n\n[build.output]\nbinary = \"build/{name}.achb\"\n",
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();

        let cli = CliOverrides {
            path: None,
            error_format: None,
            prime: None,
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
            ..CliOverrides::default()
        };

        let config = resolve_config(&cli, Some(&toml), Some(tmp.path()));
        assert_eq!(config.binary_path.unwrap(), "build/foo.achb");
    }

    #[test]
    fn resolves_scoped_host_grants_and_limits_from_vm_config() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        fs::create_dir(&data).unwrap();
        let path = tmp.path().join(TOML_FILENAME);
        fs::write(
            &path,
            r#"
[project]
name = "server"
version = "0.1.0"

[vm]
allow_read = ["data"]
allow_connect = ["127.0.0.1:443"]
max_tasks = 32
max_resources = 16
max_task_scopes = 8
max_pending_native_requests = 6
max_retained_task_results = 7
max_channels = 4
max_channel_operations = 64
blocking_workers = 2
blocking_queue_capacity = 8
"#,
        )
        .unwrap();
        let toml = load_toml(&path).unwrap();
        let config = resolve_config(&CliOverrides::default(), Some(&toml), Some(tmp.path()));

        assert_eq!(config.allow_read, vec![data]);
        assert_eq!(config.allow_connect, vec!["127.0.0.1:443"]);
        assert_eq!(config.max_tasks, Some(32));
        assert_eq!(config.max_resources, Some(16));
        assert_eq!(config.max_task_scopes, Some(8));
        assert_eq!(config.max_pending_native_requests, Some(6));
        assert_eq!(config.max_retained_task_results, Some(7));
        assert_eq!(config.max_channels, Some(4));
        assert_eq!(config.max_channel_operations, Some(64));
        assert_eq!(config.blocking_workers, Some(2));
        assert_eq!(config.blocking_queue_capacity, Some(8));
    }
}
