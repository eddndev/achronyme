use std::io::Write;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use cli::commands::ErrorFormat;
use memory::field::PrimeId;

const EXAMPLES: &[&str] = &[
    "structured_tasks.ach",
    "bounded_channel.ach",
    "timer_race.ach",
    "owned_file.ach",
    "tcp_echo.ach",
    "task_outcome.ach",
    "channel_pipeline.ach",
];

fn example_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/concurrency")
        .join(name)
}

fn run_example(name: &str, stdin: &str, extra_args: &[String]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ach"));
    command.arg("--no-config").args(extra_args);
    command
        .arg("run")
        .arg(example_path(name))
        .arg("--engine")
        .arg("interpreter");
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn documented example");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .expect("write documented example stdin");
    child
        .wait_with_output()
        .expect("wait for documented example")
}

fn assert_success(output: &Output, marker: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(marker),
        "missing `{marker}` in {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn every_documented_concurrency_example_compiles() {
    let directory = tempfile::tempdir().unwrap();
    for name in EXAMPLES {
        let output = directory.path().join(format!("{name}.achb"));
        cli::commands::compile::compile_file(
            example_path(name).to_str().unwrap(),
            Some(output.to_str().unwrap()),
            PrimeId::Bn254,
            ErrorFormat::Human,
        )
        .unwrap_or_else(|error| panic!("{name} did not compile: {error}"));
    }
}

#[test]
fn safe_in_memory_examples_execute() {
    for (name, marker) in [
        ("structured_tasks.ach", "structured-tasks-ok"),
        ("bounded_channel.ach", "bounded-channel-ok"),
        ("timer_race.ach", "timer-race-ok"),
        ("task_outcome.ach", "task-outcome-ok"),
        ("channel_pipeline.ach", "channel-pipeline-ok"),
    ] {
        assert_success(&run_example(name, "", &[]), marker);
    }
}

#[test]
fn owned_file_example_executes_inside_an_exact_granted_root() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("owned-example.txt");
    let root = directory.path().display().to_string();
    let output = run_example(
        "owned_file.ach",
        &format!("{}\n", destination.display()),
        &[
            "--allow-read".into(),
            root.clone(),
            "--allow-write".into(),
            root,
        ],
    );

    assert_success(&output, "owned-file-ok");
    assert_eq!(std::fs::read_to_string(destination).unwrap(), "achronyme");
}

#[test]
fn tcp_example_executes_with_exact_loopback_grants() {
    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let address: SocketAddr = probe.local_addr().unwrap();
    drop(probe);
    let value = address.to_string();
    let output = run_example(
        "tcp_echo.ach",
        &format!("{value}\n"),
        &[
            "--allow-connect".into(),
            value.clone(),
            "--allow-listen".into(),
            value,
        ],
    );

    assert_success(&output, "tcp-echo-ok");
}
