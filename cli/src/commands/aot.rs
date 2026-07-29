#[cfg(feature = "llvm")]
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;

use super::ErrorFormat;

#[allow(clippy::too_many_arguments)]
pub fn aot_file(
    path: &str,
    output: Option<&str>,
    runtime: Option<&str>,
    clang: &str,
    prime_id: memory::field::PrimeId,
    error_format: ErrorFormat,
    circom_lib_dirs: &[PathBuf],
) -> Result<()> {
    #[cfg(feature = "llvm")]
    {
        build_aot(
            path,
            output,
            runtime,
            clang,
            prime_id,
            error_format,
            circom_lib_dirs,
        )
    }
    #[cfg(not(feature = "llvm"))]
    {
        let _ = (
            path,
            output,
            runtime,
            clang,
            prime_id,
            error_format,
            circom_lib_dirs,
        );
        Err(anyhow::anyhow!(
            "this interpreter-only ach build excludes LLVM AOT; rebuild without `--no-default-features`"
        ))
    }
}

#[cfg(feature = "llvm")]
#[allow(clippy::too_many_arguments)]
fn build_aot(
    path: &str,
    output: Option<&str>,
    runtime: Option<&str>,
    clang: &str,
    prime_id: memory::field::PrimeId,
    error_format: ErrorFormat,
    circom_lib_dirs: &[PathBuf],
) -> Result<()> {
    use anyhow::Context;

    let program = if path.ends_with(".achb") {
        let bytes = std::fs::read(path).context("Failed to open binary file")?;
        akron::CompiledProgram::read_executable(&mut bytes.as_slice())
            .map_err(|error| anyhow::anyhow!("Loader error: {error}"))?
    } else {
        let source = std::fs::read_to_string(path).context("Failed to read source file")?;
        let mut compiler = super::new_compiler();
        let options = akronc::CompileOptions::for_source(path)
            .with_prime(prime_id)
            .with_circom_lib_dirs(circom_lib_dirs.to_vec());
        let program = compiler
            .compile_program(&source, &options)
            .map_err(|error| {
                anyhow::anyhow!(super::render_compile_error(&error, &source, error_format))
            })?;
        super::print_warnings(&mut compiler, &source, error_format);
        program
    };

    let runtime = find_runtime_archive(runtime)?;
    let output = output
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(path).with_extension(""));
    let options = akron_llvm::AotOptions::new(clang, runtime);
    let artifact = akron_llvm::AotCompiler::new(options)
        .build_executable(&program, &output)
        .map_err(|error| anyhow::anyhow!("AOT build failed: {error}"))?;
    println!(
        "Built native executable {} with clang {}.{}.{} ({}/{} bytecode instructions directly lowered)",
        artifact.executable.display(),
        artifact.clang_version.major,
        artifact.clang_version.minor,
        artifact.clang_version.patch,
        artifact.native_instruction_count,
        artifact.instruction_count
    );
    Ok(())
}

#[cfg(feature = "llvm")]
fn find_runtime_archive(explicit: Option<&str>) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = explicit {
        candidates.push(PathBuf::from(path));
    } else if let Some(path) = std::env::var_os("AKRON_AOT_RUNTIME_ARCHIVE") {
        candidates.push(PathBuf::from(path));
    } else {
        if let Ok(executable) = std::env::current_exe() {
            if let Some(directory) = executable.parent() {
                candidates.push(directory.join("libakron_aot_runtime.a"));
                candidates.push(directory.join("../lib/libakron_aot_runtime.a"));
            }
        }
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli crate has workspace parent");
        candidates.push(workspace.join("target/release/libakron_aot_runtime.a"));
        candidates.push(workspace.join("target/debug/libakron_aot_runtime.a"));
    }

    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Akron AOT runtime archive not found; pass `--runtime PATH` or set AKRON_AOT_RUNTIME_ARCHIVE"
            )
        })
}
