use std::path::Path;
use std::process::{Command, Output};

fn export(output_dir: &Path, low_memory: bool, extra: &[&str]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let source = root.join("test/circom/multiplier.circom");
    let inputs = root.join("test/proving/multiplier.inputs.toml");
    let r1cs = output_dir.join("circuit.r1cs");
    let wtns = output_dir.join("witness.wtns");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ach"));
    command.args([
        "--no-config",
        "circom",
        source.to_str().unwrap(),
        "--input-file",
        inputs.to_str().unwrap(),
        "--r1cs",
        r1cs.to_str().unwrap(),
        "--wtns",
        wtns.to_str().unwrap(),
    ]);
    if low_memory {
        command.arg("--low-memory");
    }
    command.args(extra).output().unwrap()
}

#[test]
fn low_memory_export_matches_the_standard_r1cs_and_witness() {
    let root = tempfile::tempdir().unwrap();
    let standard = root.path().join("standard");
    let bounded = root.path().join("bounded");
    std::fs::create_dir(&standard).unwrap();
    std::fs::create_dir(&bounded).unwrap();

    for output in [export(&standard, false, &[]), export(&bounded, true, &[])] {
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        std::fs::read(standard.join("circuit.r1cs")).unwrap(),
        std::fs::read(bounded.join("circuit.r1cs")).unwrap()
    );
    assert_eq!(
        std::fs::read(standard.join("witness.wtns")).unwrap(),
        std::fs::read(bounded.join("witness.wtns")).unwrap()
    );
}

#[test]
fn low_memory_export_rejects_modes_that_need_full_metadata() {
    let root = tempfile::tempdir().unwrap();
    let output = export(root.path(), true, &["--no-optimize=true"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("--low-memory requires an optimized r1cs export"));
}
