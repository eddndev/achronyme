#[test]
fn release_version_is_0_1_2_without_a_prerelease_suffix() {
    let version = env!("CARGO_PKG_VERSION");

    assert_eq!(version, "0.1.2");
    assert_eq!(version.split('.').count(), 3);
    assert!(version.split('.').all(|part| part.parse::<u64>().is_ok()));
}
