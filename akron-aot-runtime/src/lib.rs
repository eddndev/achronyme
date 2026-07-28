use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;
use std::time::Instant;

use akron::compiled::{
    runtime_api, CompiledEntry, ExecutionStats, RuntimeCapabilities, RuntimeContext,
    RUNTIME_ABI_VERSION,
};
use akron::{CompiledProgram, ValueOps, VM};
use memory::Value;

const MAX_PROGRAM_IMAGE: usize = 512 * 1024 * 1024;

#[no_mangle]
/// Execute an embedded ACHB image through a generated native entry point.
///
/// # Safety
///
/// `program_bytes` must identify `program_len` readable bytes for the duration
/// of this call. `entry` must be a valid function with the `CompiledEntry` ABI
/// generated for that exact program image.
pub unsafe extern "C" fn akron_aot_runtime_main(
    program_bytes: *const u8,
    program_len: usize,
    entry: CompiledEntry,
) -> i32 {
    if program_bytes.is_null() || program_len == 0 || program_len > MAX_PROGRAM_IMAGE {
        eprintln!("Error: invalid embedded Akron program image");
        return 2;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let bytes = unsafe { slice::from_raw_parts(program_bytes, program_len) };
        run(bytes, entry)
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            eprintln!("Error: {message}");
            1
        }
        Err(_) => {
            eprintln!("Error: Akron AOT runtime panicked");
            2
        }
    }
}

fn run(bytes: &[u8], entry: CompiledEntry) -> Result<(), String> {
    let mut reader = bytes;
    let program = CompiledProgram::read_executable(&mut reader)
        .map_err(|error| format!("embedded program loader error: {error}"))?;
    runtime_api()
        .validate(
            RUNTIME_ABI_VERSION,
            std::mem::size_of::<akron::compiled::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER2,
        )
        .map_err(|error| error.to_string())?;
    let repetitions = repetitions()?;
    let trace = std::env::var_os("AKRON_ENGINE_TRACE").is_some();
    let mut final_run = None;

    for iteration in 1..=repetitions {
        let started = Instant::now();
        let mut vm = configured_vm(program.clone())?;
        let setup = started.elapsed();

        let started = Instant::now();
        let (bailout_ip, stats) = execute_loaded(&mut vm, entry)?;
        let execution = started.elapsed();
        if trace {
            eprintln!(
                "LLVM AOT sample: iteration={iteration} setup_ns={} execution_ns={}",
                setup.as_nanos(),
                execution.as_nanos()
            );
        }
        final_run = Some((vm, bailout_ip, stats));
    }

    let (vm, bailout_ip, stats) = final_run.ok_or_else(|| "no AOT run executed".to_string())?;
    if trace {
        print_trace(bailout_ip, stats);
    }
    println!("Exit Status: {}", vm.val_to_string(&vm.last_result));
    Ok(())
}

fn configured_vm(program: CompiledProgram) -> Result<VM, String> {
    let mut vm = VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module)
            .map_err(|error| error.to_string())?;
    }
    apply_environment(&mut vm)?;
    vm.load_program(program)
        .map_err(|error| format!("program loader error: {error}"))?;
    Ok(vm)
}

fn execute_loaded(
    vm: &mut VM,
    entry: CompiledEntry,
) -> Result<(Option<u32>, ExecutionStats), String> {
    if vm.frames.len() != 1 {
        return Err(format!(
            "compiled program requires one main frame, found {}",
            vm.frames.len()
        ));
    }

    let base =
        u32::try_from(vm.frames[0].base).map_err(|_| "main frame base exceeds u32".to_string())?;
    let mut result_bits = Value::nil().to_abi_bits();
    let mut context = RuntimeContext::new(vm);
    let status = unsafe {
        entry(
            runtime_api(),
            context.as_opaque(),
            0,
            base,
            &mut result_bits,
        )
    };
    let bailout_ip = context.bailout_ip();
    let stats = context.stats();
    if let Err(error) = context.finish(status) {
        return Err(format_runtime_error(vm, &error));
    }
    let result = Value::from_abi_bits(result_bits)
        .ok_or_else(|| "compiled entry returned noncanonical Value bits".to_string())?;
    vm.last_result = if bailout_ip.is_some() {
        vm.last_result
    } else {
        result
    };
    vm.frames.pop();
    Ok((bailout_ip, stats))
}

fn repetitions() -> Result<u32, String> {
    let Some(value) = std::env::var_os("AKRON_AOT_REPEAT") else {
        return Ok(1);
    };
    let repetitions = value
        .to_str()
        .ok_or_else(|| "AKRON_AOT_REPEAT must be a positive integer".to_string())?
        .parse::<u32>()
        .map_err(|_| "AKRON_AOT_REPEAT must be a positive integer".to_string())?;
    if !(1..=10_000).contains(&repetitions) {
        return Err("AKRON_AOT_REPEAT must be between 1 and 10000".to_string());
    }
    Ok(repetitions)
}

fn print_trace(bailout_ip: Option<u32>, stats: ExecutionStats) {
    match bailout_ip {
        Some(instruction) => {
            eprintln!("LLVM AOT bailout: resumed interpreter at bytecode instruction {instruction}")
        }
        None => eprintln!("LLVM AOT native: program completed without interpreter bailout"),
    }
    eprintln!("{}", format_stats(stats));
}

fn format_stats(stats: ExecutionStats) -> String {
    format!(
        "LLVM AOT stats: direct={} runtime_calls={} compiled_calls={} native_calls={} block_polls={} slow_paths={} fallbacks={} fast_poll_hits={} slow_poll_entries={} known_call_fast_hits={} known_call_fast_misses={} specialization_hits={} specialization_misses={} first_fallback={}",
        stats.direct_instructions,
        stats.runtime_calls,
        stats.compiled_function_calls,
        stats.native_function_calls,
        stats.block_polls,
        stats.slow_paths,
        stats.interpreter_fallbacks,
        stats.fast_poll_hits,
        stats.slow_poll_entries,
        stats.known_call_fast_hits,
        stats.known_call_fast_misses,
        stats.specialization_hits,
        stats.specialization_misses,
        stats
            .first_fallback_instruction
            .map_or_else(|| "none".to_string(), |instruction| instruction.to_string()),
    )
}

fn apply_environment(vm: &mut VM) -> Result<(), String> {
    if let Ok(value) = std::env::var("AKRON_INSTRUCTION_BUDGET") {
        vm.instruction_budget = value
            .parse()
            .map_err(|_| "AKRON_INSTRUCTION_BUDGET must be an unsigned integer".to_string())?;
    }
    if let Ok(value) = std::env::var("AKRON_MAX_HEAP") {
        let limit = parse_size(&value)
            .ok_or_else(|| "AKRON_MAX_HEAP must be bytes or use K, M, or G".to_string())?;
        vm.heap.set_max_heap_bytes(limit);
    }
    if let Ok(value) = std::env::var("AKRON_STRESS_GC") {
        vm.stress_mode = matches!(value.as_str(), "1" | "true" | "yes");
    }
    Ok(())
}

fn parse_size(value: &str) -> Option<usize> {
    let value = value.trim();
    let (number, multiplier) = match value.as_bytes().last()? {
        b'K' | b'k' => (&value[..value.len() - 1], 1024usize),
        b'M' | b'm' => (&value[..value.len() - 1], 1024 * 1024),
        b'G' | b'g' => (&value[..value.len() - 1], 1024 * 1024 * 1024),
        _ => (value, 1),
    };
    number.parse::<usize>().ok()?.checked_mul(multiplier)
}

fn format_runtime_error(vm: &VM, error: &akron::RuntimeError) -> String {
    match &vm.last_error_location {
        Some((function, line)) => format!("[line {line}] in {function}: {error}"),
        None => format!("Runtime error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use akron::compiled::ExecutionStats;

    use super::format_stats;

    #[test]
    fn trace_format_preserves_tier1_and_exposes_tier2_counters() {
        let stats = ExecutionStats {
            direct_instructions: 1,
            fast_poll_hits: 2,
            slow_poll_entries: 3,
            known_call_fast_hits: 4,
            known_call_fast_misses: 5,
            specialization_hits: 6,
            specialization_misses: 7,
            ..ExecutionStats::default()
        };

        let trace = format_stats(stats);
        assert!(trace.contains("direct=1"));
        assert!(trace.contains("fast_poll_hits=2"));
        assert!(trace.contains("slow_poll_entries=3"));
        assert!(trace.contains("known_call_fast_hits=4"));
        assert!(trace.contains("known_call_fast_misses=5"));
        assert!(trace.contains("specialization_hits=6"));
        assert!(trace.contains("specialization_misses=7"));
    }
}
