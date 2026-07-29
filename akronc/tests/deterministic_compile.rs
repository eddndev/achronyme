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

#[test]
fn repeated_compilation_produces_identical_executable_bytes() {
    let compile = || {
        let program = Compiler::new()
            .compile_program(SOURCE, &CompileOptions::default())
            .unwrap();
        let mut executable = Vec::new();
        program.write_executable(&mut executable).unwrap();
        executable
    };

    let expected = compile();
    for iteration in 1..64 {
        assert_eq!(
            compile(),
            expected,
            "compilation {iteration} changed ACHB bytes"
        );
    }
}
