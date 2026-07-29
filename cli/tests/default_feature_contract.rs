#[test]
fn llvm_is_part_of_cli_default_features() {
    let manifest: toml::Value = toml::from_str(include_str!("../Cargo.toml")).unwrap();
    let defaults = manifest["features"]["default"].as_array().unwrap();

    assert!(
        defaults
            .iter()
            .any(|feature| feature.as_str() == Some("llvm")),
        "the normal CLI build must include the LLVM backend"
    );
}
