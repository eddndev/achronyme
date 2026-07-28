use std::rc::Rc;

use akron::{CompiledProgram, ValueOps, EXECUTABLE_FORMAT_VERSION, VM};
use anyhow::{Context, Result};
use std::fs;
use std::io::Cursor;

use super::ErrorFormat;
use crate::commands::engine::{ExecutionEngine, ExecutionError, PreparedEngine};
use crate::prove_handler::{DefaultProveHandler, ProveBackend, SharedProveHandler};

#[allow(clippy::too_many_arguments)]
pub fn run_file(
    path: &str,
    stress_gc: bool,
    ptau: Option<&str>,
    prove_backend: &str,
    prime_id: memory::field::PrimeId,
    max_heap: Option<&str>,
    gc_stats: bool,
    circuit_stats: bool,
    error_format: ErrorFormat,
    circom_lib_dirs: &[std::path::PathBuf],
) -> Result<()> {
    run_file_with_engine(
        path,
        stress_gc,
        ptau,
        prove_backend,
        prime_id,
        max_heap,
        None,
        gc_stats,
        circuit_stats,
        error_format,
        circom_lib_dirs,
        ExecutionEngine::Interpreter,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn run_file_with_engine(
    path: &str,
    stress_gc: bool,
    ptau: Option<&str>,
    prove_backend: &str,
    prime_id: memory::field::PrimeId,
    max_heap: Option<&str>,
    max_instructions: Option<u64>,
    gc_stats: bool,
    circuit_stats: bool,
    error_format: ErrorFormat,
    circom_lib_dirs: &[std::path::PathBuf],
    engine: ExecutionEngine,
) -> Result<()> {
    if ptau.is_some() {
        eprintln!(
            "Warning: --ptau is deprecated and ignored (native Groth16 backend does not use ptau files)"
        );
    }

    let backend = match prove_backend {
        "plonkish" => ProveBackend::Plonkish,
        _ => ProveBackend::R1cs,
    };

    let (mut vm, handler) = configured_vm(
        backend,
        prime_id,
        error_format,
        circuit_stats,
        stress_gc,
        max_heap,
        max_instructions,
    )?;

    let result = if path.ends_with(".achb") {
        execute_binary(path, &mut vm, engine)
    } else {
        let content = fs::read_to_string(path).context("Failed to source file")?;
        let mut compiler = super::new_compiler();
        let options = akronc::CompileOptions::for_source(path)
            .with_prime(prime_id)
            .with_circom_lib_dirs(circom_lib_dirs.to_vec());
        let program = compiler.compile_program(&content, &options).map_err(|e| {
            let rendered = super::render_compile_error(&e, &content, error_format);
            anyhow::anyhow!("{rendered}")
        })?;

        super::print_warnings(&mut compiler, &content, error_format);

        vm.circom_handler = Some(Box::new(
            crate::circom_handler::DefaultCircomWitnessHandler::new(
                compiler.circom_library_registry.take_libraries(),
            ),
        ));
        execute_program(&mut vm, program, engine)
    };

    print_gc_stats(gc_stats, &vm);
    handler.print_circuit_stats();
    result?;

    println!("Exit Status: {}", vm.val_to_string(&vm.last_result));
    Ok(())
}

fn configured_vm(
    backend: ProveBackend,
    prime_id: memory::field::PrimeId,
    error_format: ErrorFormat,
    circuit_stats: bool,
    stress_gc: bool,
    max_heap: Option<&str>,
    max_instructions: Option<u64>,
) -> Result<(VM, Rc<DefaultProveHandler>)> {
    let mut vm = VM::new();
    super::register_std_modules(&mut vm)?;
    vm.stress_mode = stress_gc;
    if let Some(limit_str) = max_heap {
        let limit = parse_size(limit_str).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid --max-heap value: `{limit_str}` (expected e.g. \"256M\", \"1G\", \"512K\")"
            )
        })?;
        vm.heap.set_max_heap_bytes(limit);
    }
    if let Some(limit) = max_instructions {
        vm.instruction_budget = limit;
    }
    let handler = Rc::new(DefaultProveHandler::new(
        backend,
        prime_id,
        error_format,
        circuit_stats,
    ));
    vm.verify_handler = Some(Box::new(SharedProveHandler(Rc::clone(&handler))));
    vm.prove_handler = Some(Box::new(SharedProveHandler(Rc::clone(&handler))));
    Ok((vm, handler))
}

fn execute_binary(path: &str, vm: &mut VM, engine: ExecutionEngine) -> Result<()> {
    let bytes = fs::read(path).context("Failed to open binary file")?;
    let is_current = bytes.get(3).copied() == Some(EXECUTABLE_FORMAT_VERSION);
    if is_current {
        let program = CompiledProgram::read_executable(&mut Cursor::new(bytes))
            .map_err(|error| anyhow::anyhow!("Loader error: {error}"))?;
        return execute_program(vm, program, engine);
    }

    match engine {
        ExecutionEngine::Jit => {
            return Err(anyhow::anyhow!(
                "LLVM JIT requires current ACHB format 0x{EXECUTABLE_FORMAT_VERSION:02x}"
            ));
        }
        ExecutionEngine::Auto => {
            eprintln!("LLVM JIT fallback: legacy ACHB images execute with the interpreter")
        }
        ExecutionEngine::Interpreter => {}
    }
    vm.load_executable(&mut Cursor::new(bytes))
        .map_err(|error| anyhow::anyhow!("Loader error: {error}"))?;
    let result = vm.interpret().map_err(ExecutionError::Runtime);
    map_execution_result(vm, result)
}

fn execute_program(vm: &mut VM, program: CompiledProgram, engine: ExecutionEngine) -> Result<()> {
    let prepared = PreparedEngine::prepare(engine, &program)?;
    vm.load_program(program)
        .map_err(|error| anyhow::anyhow!("Program loader error: {error}"))?;
    let result = prepared.execute(vm);
    map_execution_result(vm, result)
}

fn map_execution_result(vm: &VM, result: Result<(), ExecutionError>) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(ExecutionError::Runtime(error)) => {
            Err(anyhow::anyhow!(format_runtime_error(vm, &error)))
        }
        #[cfg(feature = "llvm")]
        Err(ExecutionError::Engine(error)) => Err(error),
    }
}

fn print_gc_stats(gc_stats: bool, vm: &VM) {
    if gc_stats {
        let s = &vm.heap.stats;
        eprintln!("-- GC Stats --");
        eprintln!("  Collections:    {}", s.collections);
        eprintln!("  Freed (total):  {} bytes", s.total_freed_bytes);
        eprintln!("  Peak heap:      {} bytes", s.peak_heap_bytes);
        eprintln!(
            "  GC time:        {:.3} ms",
            s.total_gc_time_ns as f64 / 1_000_000.0
        );
        eprintln!("  Heap now:       {} bytes", vm.heap.bytes_allocated);
    }
}

/// Remap field literal handles from compiler-space to VM heap-space.
pub fn remap_field_handles(constants: &mut [memory::Value], field_map: &[u32]) {
    for val in constants.iter_mut() {
        if val.is_field() {
            let old_handle = val.as_handle().expect("Field value must have handle");
            if let Some(&new_handle) = field_map.get(old_handle as usize) {
                *val = memory::Value::field(new_handle);
            }
        }
    }
}

/// Remap BigInt literal handles from compiler-space to VM heap-space.
pub fn remap_bigint_handles(constants: &mut [memory::Value], bigint_map: &[u32]) {
    for val in constants.iter_mut() {
        if val.is_bigint() {
            let old_handle = val.as_handle().expect("BigInt value must have handle");
            if let Some(&new_handle) = bigint_map.get(old_handle as usize) {
                *val = memory::Value::bigint(new_handle);
            }
        }
    }
}

/// Parse a human-readable size string (e.g., "256M", "1G", "512K") into bytes.
fn parse_size(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num_part, multiplier) = match s.as_bytes().last()? {
        b'K' | b'k' => (&s[..s.len() - 1], 1024usize),
        b'M' | b'm' => (&s[..s.len() - 1], 1024 * 1024),
        b'G' | b'g' => (&s[..s.len() - 1], 1024 * 1024 * 1024),
        _ => (s, 1),
    };
    let num: usize = num_part.parse().ok()?;
    num.checked_mul(multiplier)
}

/// Format a runtime error with source location if available.
fn format_runtime_error(vm: &VM, err: &akron::RuntimeError) -> String {
    match &vm.last_error_location {
        Some((func_name, line)) => format!("[line {line}] in {func_name}: {err}"),
        None => format!("Runtime error: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_size;

    #[test]
    fn test_parse_size_bytes() {
        assert_eq!(parse_size("1048576"), Some(1048576));
    }
    #[test]
    fn test_parse_size_kb() {
        assert_eq!(parse_size("512K"), Some(524288));
    }
    #[test]
    fn test_parse_size_mb() {
        assert_eq!(parse_size("256M"), Some(268435456));
    }
    #[test]
    fn test_parse_size_gb() {
        assert_eq!(parse_size("1G"), Some(1073741824));
    }
    #[test]
    fn test_parse_size_lowercase() {
        assert_eq!(parse_size("256m"), Some(268435456));
    }
    #[test]
    fn test_parse_size_zero() {
        assert_eq!(parse_size("0"), Some(0));
    }
    #[test]
    fn test_parse_size_empty() {
        assert_eq!(parse_size(""), None);
    }
    #[test]
    fn test_parse_size_invalid() {
        assert_eq!(parse_size("abc"), None);
    }
}
