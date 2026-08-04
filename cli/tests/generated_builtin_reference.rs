#[test]
fn checked_in_builtin_reference_matches_the_canonical_registry() {
    let native_table = achronyme_std::std_native_table();
    let registry = resolve::BuiltinRegistry::with_extra_natives(&native_table);
    let generated = registry.markdown_reference();
    let checked_in = include_str!("../../docs/reference/builtins.md");

    assert_eq!(checked_in, generated);
}
