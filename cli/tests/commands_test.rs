use cli::commands::ErrorFormat;
use memory::field::PrimeId;
use std::io::Write;
use tempfile::NamedTempFile;

const EF: ErrorFormat = ErrorFormat::Human;

fn write_temp_source(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".ach").unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

// ======================================================================
// compile_file
// ======================================================================

#[test]
fn compile_valid_source_with_output() {
    let src = write_temp_source("let x = 1 + 2\nprint(x)");
    let out = tempfile::NamedTempFile::with_suffix(".achb").unwrap();
    let out_path = out.path().to_str().unwrap().to_string();

    let result = cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        Some(&out_path),
        PrimeId::Bn254,
        EF,
    );
    assert!(result.is_ok(), "compile_file failed: {:?}", result.err());

    // Verify .achb was created with the ACH magic header
    let bytes = std::fs::read(&out_path).unwrap();
    assert!(bytes.len() >= 4, "output file too small");
    assert_eq!(
        &bytes[..4],
        &[b'A', b'C', b'H', akron::EXECUTABLE_FORMAT_VERSION],
        "wrong magic header"
    );
}

#[test]
fn compile_valid_source_no_output() {
    let src = write_temp_source("let x = 42");
    let result = cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        None,
        PrimeId::Bn254,
        EF,
    );
    assert!(
        result.is_ok(),
        "compile_file (no output) failed: {:?}",
        result.err()
    );
}

#[test]
fn compile_invalid_source_returns_error() {
    let src = write_temp_source("let = ???");
    let result = cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        None,
        PrimeId::Bn254,
        EF,
    );
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("error"), "expected compile error, got: {err}");
}

#[test]
fn compile_nonexistent_file_returns_error() {
    let result = cli::commands::compile::compile_file(
        "/tmp/nonexistent_achronyme_test.ach",
        None,
        PrimeId::Bn254,
        EF,
    );
    assert!(result.is_err());
}

// ======================================================================
// run_file
// ======================================================================

#[test]
fn run_valid_arithmetic_source() {
    let src = write_temp_source("let x = 2 + 3\nprint(x)");
    let result = cli::commands::run::run_file(
        src.path().to_str().unwrap(),
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        false,
        false,
        EF,
        &[],
    );
    assert!(result.is_ok(), "run_file failed: {:?}", result.err());
}

#[test]
fn run_source_with_runtime_error() {
    let src = write_temp_source("let x = 1 / 0");
    let result = cli::commands::run::run_file(
        src.path().to_str().unwrap(),
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        false,
        false,
        EF,
        &[],
    );
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("division by zero"),
        "expected runtime error, got: {err}"
    );
}

#[test]
fn run_nonexistent_file_returns_error() {
    let result = cli::commands::run::run_file(
        "/tmp/nonexistent_achronyme_test.ach",
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        false,
        false,
        EF,
        &[],
    );
    assert!(result.is_err());
}

#[test]
fn run_compiled_binary() {
    // First compile to .achb, then run the binary
    let src = write_temp_source("let x = 10\nprint(x)");
    let out = tempfile::NamedTempFile::with_suffix(".achb").unwrap();
    let out_path = out.path().to_str().unwrap().to_string();

    cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        Some(&out_path),
        PrimeId::Bn254,
        EF,
    )
    .expect("compile should succeed");

    let result = cli::commands::run::run_file(
        &out_path,
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        false,
        false,
        EF,
        &[],
    );
    assert!(
        result.is_ok(),
        "run compiled binary failed: {:?}",
        result.err()
    );
}

#[test]
fn cli_denies_implicit_local_proving_setup() {
    let src = write_temp_source(include_str!("../../test/prove/basic_prove.ach"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "--no-config",
            "run",
            src.path().to_str().unwrap(),
            "--engine",
            "interpreter",
        ])
        .output()
        .expect("run ach");

    assert!(
        !output.status.success(),
        "implicit setup unexpectedly succeeded"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("insecure local trusted setup is disabled"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn run_preflights_file_authority_and_accepts_an_explicit_root_grant() {
    use cli::commands::engine::ExecutionEngine;
    use cli::commands::runtime::RuntimeSecurity;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.txt");
    std::fs::write(&input, "granted").unwrap();
    let source = format!("return await read_file({:?})", input.to_string_lossy());
    let src = write_temp_source(&source);

    let denied = cli::commands::run::run_file_with_engine(
        src.path().to_str().unwrap(),
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        None,
        false,
        false,
        EF,
        &[],
        ExecutionEngine::Interpreter,
        &RuntimeSecurity::default(),
    )
    .unwrap_err();
    assert!(
        denied.to_string().contains("file.read"),
        "unexpected preflight error: {denied}"
    );

    let allowed = RuntimeSecurity {
        allow_read: vec![directory.path().to_path_buf()],
        ..RuntimeSecurity::default()
    };
    cli::commands::run::run_file_with_engine(
        src.path().to_str().unwrap(),
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        None,
        false,
        false,
        EF,
        &[],
        ExecutionEngine::Interpreter,
        &allowed,
    )
    .unwrap();
}

#[test]
fn compiled_binary_preserves_runtime_error_location() {
    let src = write_temp_source("let a = 10\nlet b = 0\nlet c = a / b");
    let out = tempfile::NamedTempFile::with_suffix(".achb").unwrap();
    let out_path = out.path().to_str().unwrap().to_string();

    cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        Some(&out_path),
        PrimeId::Bn254,
        EF,
    )
    .expect("compile should succeed");

    let result = cli::commands::run::run_file(
        &out_path,
        false,
        None,
        "r1cs",
        PrimeId::Bn254,
        None,
        false,
        false,
        EF,
        &[],
    );
    let error = result
        .expect_err("compiled program should fail")
        .to_string();
    assert!(error.contains("[line 3] in main"), "{error}");
    assert!(error.contains("division by zero"), "{error}");
}

// ======================================================================
// disassemble_file
// ======================================================================

#[test]
fn disassemble_valid_source() {
    let src = write_temp_source("let x = 1 + 2\nprint(x)");
    let result = cli::commands::disassemble::disassemble_file(src.path().to_str().unwrap(), EF);
    assert!(result.is_ok(), "disassemble failed: {:?}", result.err());
}

#[test]
fn disassemble_invalid_source_returns_error() {
    let src = write_temp_source("let = ???");
    let result = cli::commands::disassemble::disassemble_file(src.path().to_str().unwrap(), EF);
    assert!(result.is_err());
}

#[test]
fn inspect_manifest_reports_effects_requests_grants_and_limits_as_json() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.txt");
    std::fs::write(&input, "hello").unwrap();
    let source = format!("return await read_file({:?})", input.to_string_lossy());
    let src = write_temp_source(&source);

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ach"))
        .arg("--no-config")
        .arg("--error-format")
        .arg("json")
        .arg("--allow-read")
        .arg(directory.path())
        .arg("--max-tasks")
        .arg("12")
        .arg("--max-pending-native-requests")
        .arg("5")
        .arg("--max-retained-task-results")
        .arg("6")
        .arg("--max-channels")
        .arg("3")
        .arg("--blocking-workers")
        .arg("2")
        .arg("--blocking-queue-capacity")
        .arg("7")
        .arg("inspect")
        .arg(src.path())
        .arg("--manifest")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(manifest["effects"], "task,io.file");
    assert_eq!(manifest["requested_host_capabilities"], "file.read");
    assert!(manifest["granted_host_capabilities"]
        .as_str()
        .unwrap()
        .contains("file.read"));
    assert_eq!(manifest["limits"]["tasks"], 12);
    assert_eq!(manifest["limits"]["pending_native_requests"], 5);
    assert_eq!(manifest["limits"]["retained_task_results"], 6);
    assert_eq!(manifest["limits"]["channels"], 3);
    assert_eq!(manifest["limits"]["blocking_workers"], 2);
    assert_eq!(manifest["limits"]["blocking_queue_capacity"], 7);
    assert_eq!(manifest["proving"]["key_source"], "deny-insecure-setup");
}

#[test]
fn inspect_manifest_makes_insecure_development_setup_visible() {
    let src = write_temp_source("return 1");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ach"))
        .args([
            "--no-config",
            "--error-format",
            "json",
            "--insecure-dev-setup",
        ])
        .arg("inspect")
        .arg(src.path())
        .arg("--manifest")
        .output()
        .unwrap();
    assert!(output.status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(manifest["proving"]["key_source"], "insecure-local");
    assert_eq!(
        manifest["proving"]["trusted_key_dir"],
        serde_json::Value::Null
    );
}

#[test]
fn inspect_manifest_reports_the_selected_trusted_key_store() {
    let src = write_temp_source("return 1");
    let trusted_dir = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ach"))
        .args(["--no-config", "--error-format", "json", "--trusted-key-dir"])
        .arg(trusted_dir.path())
        .arg("inspect")
        .arg(src.path())
        .arg("--manifest")
        .output()
        .unwrap();
    assert!(output.status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(manifest["proving"]["key_source"], "trusted-store");
    assert_eq!(
        manifest["proving"]["trusted_key_dir"],
        trusted_dir.path().display().to_string()
    );
}

// ======================================================================
// error_format tests
// ======================================================================

#[test]
fn json_error_format_produces_valid_json() {
    let src = write_temp_source("let = ???");
    let result = cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        None,
        PrimeId::Bn254,
        ErrorFormat::Json,
    );
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&err).expect("should be valid JSON");
    assert!(parsed.get("message").is_some(), "JSON should have message");
    assert!(parsed.get("level").is_some(), "JSON should have level");
    assert!(parsed.get("spans").is_some(), "JSON should have spans");
}

#[test]
fn short_error_format_is_grep_friendly() {
    let src = write_temp_source("let = ???");
    let result = cli::commands::compile::compile_file(
        src.path().to_str().unwrap(),
        None,
        PrimeId::Bn254,
        ErrorFormat::Short,
    );
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    // Should contain severity and colon-separated location
    assert!(
        err.contains("error:"),
        "short format should contain 'error:', got: {err}"
    );
}

#[test]
fn json_warning_format_produces_valid_json() {
    // This source triggers an unused variable warning
    let src = write_temp_source("fn test() { let x = 5; 1 }");
    let content = std::fs::read_to_string(src.path()).unwrap();
    let mut compiler = akronc::Compiler::new();
    let _ = compiler.compile(&content);
    let warnings = compiler.take_warnings();
    assert!(!warnings.is_empty(), "should have warnings");

    // Render each warning as JSON
    for w in &warnings {
        let rendered = cli::commands::render_compile_error(
            &akronc::CompilerError::DiagnosticError(Box::new(w.clone())),
            &content,
            ErrorFormat::Json,
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&rendered).expect("warning JSON should be valid");
        assert_eq!(parsed["level"], "warning");
    }
}
