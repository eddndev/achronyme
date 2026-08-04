fn main() {
    let native_table = achronyme_std::std_native_table();
    let registry = resolve::BuiltinRegistry::with_extra_natives(&native_table);
    print!("{}", registry.markdown_reference());
}
