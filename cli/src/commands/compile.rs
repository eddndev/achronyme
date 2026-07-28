use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use super::ErrorFormat;

pub fn compile_file(
    path: &str,
    output: Option<&str>,
    prime_id: memory::field::PrimeId,
    error_format: ErrorFormat,
) -> Result<()> {
    compile_file_with_lib_dirs(path, output, prime_id, error_format, &[])
}

pub fn compile_file_with_lib_dirs(
    path: &str,
    output: Option<&str>,
    prime_id: memory::field::PrimeId,
    error_format: ErrorFormat,
    circom_lib_dirs: &[PathBuf],
) -> Result<()> {
    let content = fs::read_to_string(path).context("Failed to read file")?;
    let mut compiler = super::new_compiler();
    let options = akronc::CompileOptions::for_source(path)
        .with_prime(prime_id)
        .with_circom_lib_dirs(circom_lib_dirs.to_vec());
    let program = compiler
        .compile_program(&content, &options)
        .map_err(|error| {
            let rendered = super::render_compile_error(&error, &content, error_format);
            anyhow::anyhow!("{rendered}")
        })?;

    super::print_warnings(&mut compiler, &content, error_format);

    println!("Compiled {} instructions.", program.main.chunk.len());

    if let Some(out_path) = output {
        let mut file = fs::File::create(out_path).context("Failed to create output file")?;
        program
            .write_executable(&mut file)
            .map_err(|error| anyhow::anyhow!("Failed to write executable: {error}"))?;
        println!("Saved binary to {}", out_path);
    }
    Ok(())
}
