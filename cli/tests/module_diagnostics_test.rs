use std::path::{Path, PathBuf};

use akronc::CompileOptions;
use cli::commands::ErrorFormat;
use memory::field::PrimeId;

struct ModuleProject {
    _directory: tempfile::TempDir,
    main: PathBuf,
    module: PathBuf,
}

fn module_project(module_source: &str) -> ModuleProject {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("main.ach");
    let module = directory.path().join("broken.ach");
    std::fs::write(
        &main,
        "import \"./broken.ach\" as broken\nprint(broken::answer(1))\n",
    )
    .unwrap();
    std::fs::write(&module, module_source).unwrap();
    ModuleProject {
        _directory: directory,
        main,
        module,
    }
}

fn compile_error(project: &ModuleProject, format: ErrorFormat) -> String {
    let result = cli::commands::compile::compile_file(
        project.main.to_str().unwrap(),
        None,
        PrimeId::Bn254,
        format,
    );
    format!("{}", result.expect_err("module must fail to compile"))
}

#[test]
fn module_parse_error_preserves_file_and_span_in_json() {
    let project = module_project(
        "// The parse error is deliberately on line 2.\n\
         export fn answer(value, ) { value }\n",
    );

    let rendered = compile_error(&project, ErrorFormat::Json);
    let diagnostic: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    let primary = &diagnostic["spans"][0];

    assert_eq!(
        primary["file_name"],
        project.module.canonicalize().unwrap().display().to_string()
    );
    assert_eq!(primary["line_start"], 2);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("expected identifier"));
}

#[test]
fn module_parse_error_renders_the_module_source() {
    let project = module_project(
        "// The parse error is deliberately on line 2.\n\
         export fn answer(value, ) { value }\n",
    );

    let rendered = compile_error(&project, ErrorFormat::Human);

    assert!(rendered.contains(project.module.canonicalize().unwrap().to_str().unwrap()));
    assert!(rendered.contains("export fn answer(value, ) { value }"));
    assert!(!rendered.contains("print(broken::answer(1))"));
}

#[test]
fn warning_from_imported_module_preserves_its_file() {
    let project = module_project("export fn answer(unused) { 42 }\n");
    let source = std::fs::read_to_string(&project.main).unwrap();
    let mut compiler = cli::commands::new_compiler();

    compiler
        .compile_program(&source, &CompileOptions::for_source(&project.main))
        .unwrap();

    let warning = compiler
        .take_warnings()
        .into_iter()
        .find(|warning| {
            warning
                .message
                .contains("unused function parameter: `unused`")
        })
        .expect("imported module must emit W001");
    assert_eq!(
        warning.primary_span.file.as_deref(),
        Some(project.module.canonicalize().unwrap().as_path())
    );
    assert_eq!(warning.primary_span.line_start, 1);
}

#[test]
fn module_path_is_absolute_in_short_diagnostics() {
    let project = module_project("export fn answer(value, ) { value }\n");

    let rendered = compile_error(&project, ErrorFormat::Short);

    assert!(rendered.starts_with(&project.module.canonicalize().unwrap().display().to_string()));
    assert!(Path::new(rendered.split(':').next().unwrap()).is_absolute());
}

#[test]
fn imported_prove_array_parameters_count_as_reads() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("main.ach");
    let module = directory.path().join("proofs.ach");
    std::fs::write(
        &main,
        "import \"./proofs.ach\" as proofs\n\
         let generated = proofs::prove_membership(\n\
             0p1,\n\
             0p2,\n\
             [0p3, 0p4],\n\
             [0p0, 0p1]\n\
         )\n",
    )
    .unwrap();
    std::fs::write(
        &module,
        "export fn prove_membership(\n\
             root,\n\
             leaf,\n\
             path: Field[2],\n\
             indices: Field[2]\n\
         ) {\n\
             prove(root: Public) {\n\
                 merkle_verify(root, leaf, path, indices)\n\
             }\n\
         }\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&main).unwrap();
    let mut compiler = cli::commands::new_compiler();

    compiler
        .compile_program(&source, &CompileOptions::for_source(&main))
        .unwrap();

    let warnings = compiler.take_warnings();
    assert!(
        !warnings.iter().any(|warning| {
            warning
                .message
                .contains("unused function parameter: `path`")
                || warning
                    .message
                    .contains("unused function parameter: `indices`")
        }),
        "imported prove captures must count as parameter reads: {warnings:?}"
    );
}
