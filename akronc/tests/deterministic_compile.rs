use akronc::{CompileOptions, Compiler};

const SOURCE: &str = r#"
mut values = []
mut i = 0
while i < 20000 {
    values.push(i * i)
    i = i + 1
}

mut index = 0
mut total = 0
while index < 20000 {
    total = total + values[index]
    index = index + 1
}

assert(total == 2666466670000)
total
"#;

const CONCURRENT_SOURCE: &str = r#"
fn invoke(f) { f() }

let value = concurrent {
    let task = spawn invoke(time)
    await task
}
value
"#;

fn compile(source: &str) -> (akron::CompiledProgram, Vec<u8>) {
    let program = Compiler::new()
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let mut executable = Vec::new();
    program.write_executable(&mut executable).unwrap();
    (program, executable)
}

#[test]
fn repeated_compilation_produces_identical_executable_bytes() {
    let expected = compile(SOURCE).1;
    for iteration in 1..64 {
        assert_eq!(
            compile(SOURCE).1,
            expected,
            "compilation {iteration} changed ACHB bytes"
        );
    }
}

#[test]
fn concurrent_compilation_and_metadata_are_deterministic() {
    let (program, expected) = compile(CONCURRENT_SOURCE);
    assert!(program
        .requested_effects()
        .contains(akron::specs::EffectSet::TASK | akron::specs::EffectSet::UNKNOWN_HOST));
    assert!(program
        .requested_host_capabilities()
        .contains(akron::specs::CapabilitySet::CLOCK | akron::specs::CapabilitySet::UNKNOWN_HOST));

    for iteration in 1..64 {
        assert_eq!(
            compile(CONCURRENT_SOURCE).1,
            expected,
            "concurrent compilation {iteration} changed ACHB bytes"
        );
    }
}
