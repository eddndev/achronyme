use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::commands::ErrorFormat;
use memory::field::PrimeId;

#[path = "engine_oracle/concurrency.rs"]
mod concurrency;

#[derive(Clone, Copy, Default)]
struct OracleGrants<'a> {
    file_root: Option<&'a Path>,
    connect: Option<std::net::SocketAddr>,
    listen: Option<std::net::SocketAddr>,
}

fn workspace_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(relative)
}

fn run_program(path: &Path, stdin: &str, engine: Option<&str>, grants: OracleGrants<'_>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ach"));
    command.arg("--no-config");
    if let Some(root) = grants.file_root {
        command
            .arg("--allow-read")
            .arg(root)
            .arg("--allow-write")
            .arg(root);
    }
    if let Some(address) = grants.connect {
        command.arg("--allow-connect").arg(address.to_string());
    }
    if let Some(address) = grants.listen {
        command.arg("--allow-listen").arg(address.to_string());
    }
    command.arg("run").arg(path);
    if let Some(engine) = engine {
        command.arg("--engine").arg(engine);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ach");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait for ach")
}

#[cfg(feature = "llvm")]
fn run_native(path: &Path, stdin: &str, grants: OracleGrants<'_>) -> Output {
    let mut command = Command::new(path);
    if let Some(root) = grants.file_root {
        let grant = std::env::join_paths([root]).expect("encode AOT file grant");
        command
            .env("AKRON_ALLOW_READ", &grant)
            .env("AKRON_ALLOW_WRITE", &grant);
    }
    if let Some(address) = grants.connect {
        command.env("AKRON_ALLOW_CONNECT", address.to_string());
    }
    if let Some(address) = grants.listen {
        command.env("AKRON_ALLOW_LISTEN", address.to_string());
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn AOT executable");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .expect("write AOT stdin");
    child.wait_with_output().expect("wait for AOT executable")
}

fn assert_source_bytecode_parity(source: &Path, stdin: &str) -> (Output, Output) {
    assert_source_bytecode_parity_inner(source, stdin, OracleGrants::default())
}

fn assert_source_bytecode_parity_with_file_grants(
    source: &Path,
    stdin: &str,
    root: &Path,
) -> (Output, Output) {
    assert_source_bytecode_parity_inner(
        source,
        stdin,
        OracleGrants {
            file_root: Some(root),
            ..OracleGrants::default()
        },
    )
}

fn assert_source_bytecode_parity_with_grants(
    source: &Path,
    stdin: &str,
    grants: OracleGrants<'_>,
) -> (Output, Output) {
    assert_source_bytecode_parity_inner(source, stdin, grants)
}

fn assert_source_bytecode_parity_inner(
    source: &Path,
    stdin: &str,
    grants: OracleGrants<'_>,
) -> (Output, Output) {
    let directory = tempfile::tempdir().unwrap();
    let binary = directory.path().join("program.achb");
    cli::commands::compile::compile_file(
        source.to_str().unwrap(),
        Some(binary.to_str().unwrap()),
        PrimeId::Bn254,
        ErrorFormat::Human,
    )
    .expect("compile oracle source");

    let source_output = run_program(source, stdin, Some("interpreter"), grants);
    let bytecode_output = run_program(&binary, stdin, Some("interpreter"), grants);
    #[cfg(feature = "llvm")]
    let mut alternatives = vec![("bytecode interpreter", &bytecode_output)];
    #[cfg(not(feature = "llvm"))]
    let alternatives = vec![("bytecode interpreter", &bytecode_output)];
    #[cfg(feature = "llvm")]
    let (jit_source_output, jit_bytecode_output) = (
        run_program(source, stdin, Some("jit"), grants),
        run_program(&binary, stdin, Some("jit"), grants),
    );
    #[cfg(feature = "llvm")]
    alternatives.extend([
        ("source JIT", &jit_source_output),
        ("bytecode JIT", &jit_bytecode_output),
    ]);
    #[cfg(feature = "llvm")]
    let aot_output = {
        let executable = directory.path().join("program-native");
        let runtime = workspace_path("target/debug/libakron_aot_runtime.a");
        assert!(
            runtime.is_file(),
            "build the runtime first with cargo build -p akron-aot-runtime"
        );
        let build = Command::new(env!("CARGO_BIN_EXE_ach"))
            .arg("--no-config")
            .arg("aot")
            .arg(&binary)
            .arg("--output")
            .arg(&executable)
            .arg("--runtime")
            .arg(&runtime)
            .output()
            .expect("build AOT executable");
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        run_native(&executable, stdin, grants)
    };
    #[cfg(feature = "llvm")]
    alternatives.push(("AOT executable", &aot_output));
    for (name, output) in alternatives {
        assert_eq!(
            source_output.status.code(),
            output.status.code(),
            "{name} exit status differs"
        );
        assert_eq!(source_output.stdout, output.stdout, "{name} stdout differs");
        assert_eq!(source_output.stderr, output.stderr, "{name} stderr differs");
    }
    (source_output, bytecode_output)
}

#[test]
fn oracle_modules_and_native_print() {
    let (output, _) =
        assert_source_bytecode_parity(&workspace_path("test/modules/main_vm.ach"), "");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("3\n"));
}

#[test]
fn oracle_closures_upvalues_and_higher_order_calls() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("closures.ach");
    std::fs::write(
        &source,
        "fn make_adder(offset) { return fn(value) { value + offset } }\n\
         let add5 = make_adder(5)\n\
         let values = [1, 2, 3]\n\
         let mapped = values.map(add5)\n\
         assert(mapped[0] == 6)\n\
         assert(mapped[2] == 8)\n\
         print(mapped)\n",
    )
    .unwrap();

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("[6, 7, 8]"));
}

#[test]
fn oracle_collections_and_control_flow() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("collections.ach");
    std::fs::write(
        &source,
        "let values = [1, 2, 3, 4]\n\
         mut total = 0\n\
         for value in values { total = total + value }\n\
         let record = {sum: total, ok: total == 10}\n\
         if record.ok { print(record.sum) } else { assert(false) }\n",
    )
    .unwrap();

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("10\n"));
}

#[test]
fn oracle_real_file_io() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("io.ach");
    let destination = directory.path().join("output.txt");
    std::fs::write(
        &source,
        "let path = read_line()\n\
         write_file(path, \"akron-llvm-io\")\n\
         let content = read_file(path)\n\
         assert(content == \"akron-llvm-io\")\n\
         print(content)\n",
    )
    .unwrap();
    let stdin = format!("{}\n", destination.display());

    let (output, _) =
        assert_source_bytecode_parity_with_file_grants(&source, &stdin, directory.path());
    assert!(output.status.success());
    assert_eq!(
        std::fs::read_to_string(destination).unwrap(),
        "akron-llvm-io"
    );
}

#[test]
fn oracle_file_capability_denial_is_identical_across_engines() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("denied-io.ach");
    std::fs::write(&source, "read_file(\"denied.txt\")\n").unwrap();

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Host capability preflight failed"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn oracle_runtime_error_and_line_information() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("error.ach");
    std::fs::write(&source, "let a = 10\nlet b = 0\nlet c = a / b\n").unwrap();

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[line 3] in main"), "{stderr}");
    assert!(stderr.contains("division by zero"), "{stderr}");
}
