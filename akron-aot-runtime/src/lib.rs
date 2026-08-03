use std::slice;
use std::time::Instant;

use akron::compiled::{
    runtime_api, CompiledEntry, ExecutionStats, RuntimeCapabilities, RuntimeContext,
};
use akron::{CompiledProgram, HostPolicy, ValueOps, VM};
use memory::Value;

mod config;

const MAX_PROGRAM_IMAGE: usize = 512 * 1024 * 1024;

#[no_mangle]
/// Execute an embedded ACHB image through a generated native entry point.
///
/// # Safety
///
/// `program_bytes` must identify `program_len` readable bytes for the duration
/// of this call. `entry` must be a valid function with the `CompiledEntry` ABI
/// generated for that exact program image. The ABI requirement values must be
/// the ones used to lower `entry`. Expected loader and runtime failures return
/// an exit status. An unexpected internal panic is fatal and is never unwound
/// across this C ABI boundary.
pub unsafe extern "C" fn akron_aot_runtime_main(
    program_bytes: *const u8,
    program_len: usize,
    entry: CompiledEntry,
    required_abi_version: u32,
    required_abi_size: u32,
    required_capabilities: u64,
) -> i32 {
    if program_bytes.is_null() || program_len == 0 || program_len > MAX_PROGRAM_IMAGE {
        eprintln!("Error: invalid embedded Akron program image");
        return 2;
    }

    let bytes = unsafe { slice::from_raw_parts(program_bytes, program_len) };
    match run(
        bytes,
        entry,
        required_abi_version,
        required_abi_size,
        required_capabilities,
    ) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("Error: {message}");
            1
        }
    }
}

fn run(
    bytes: &[u8],
    entry: CompiledEntry,
    required_abi_version: u32,
    required_abi_size: u32,
    required_capabilities: u64,
) -> Result<(), String> {
    let mut reader = bytes;
    let program = CompiledProgram::read_executable(&mut reader)
        .map_err(|error| format!("embedded program loader error: {error}"))?;
    runtime_api()
        .validate(
            required_abi_version,
            required_abi_size,
            RuntimeCapabilities::from_bits(required_capabilities),
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
    vm.host_policy = HostPolicy::standard_cli();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module)
            .map_err(|error| error.to_string())?;
    }
    config::apply_environment(&mut vm)?;
    vm.host_policy
        .require_program(program.requested_host_capabilities())
        .map_err(|error| format!("Host capability preflight failed: {error}"))?;
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
    if bailout_ip.is_some() {
        if !vm.frames.is_empty() {
            return Err(format!(
                "interpreter bailout left {} frames active",
                vm.frames.len()
            ));
        }
    } else {
        vm.finish_compiled_top_level(result)
            .map_err(|error| format_runtime_error(vm, &error))?;
    }
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

fn format_runtime_error(vm: &VM, error: &akron::RuntimeError) -> String {
    let mut message = match &vm.last_error_location {
        Some((function, line)) => format!("[line {line}] in {function}: {error}"),
        None => format!("Runtime error: {error}"),
    };
    if let Some(task) = vm.last_task_failure() {
        message.push_str("\nTask failure: ");
        message.push_str(&task.to_string());
    }
    message
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::c_void;

    use akron::compiled::{ExecutionStats, RuntimeApi, RuntimeStatus, STATUS_OK};
    use akron::opcode::instruction::{encode_abc, encode_abx};
    use akron::specs::CapabilitySet;
    use akron::{CompiledProgram, OpCode, VM};
    use memory::field::PrimeId;
    use memory::{Function, UpvalueLocation, Value};

    use super::{configured_vm, execute_loaded, format_stats};

    fn captured_main_local_program() -> CompiledProgram {
        let add_offset = Function {
            name: "add_offset".to_string(),
            arity: 1,
            max_slots: 2,
            chunk: vec![
                encode_abx(OpCode::GetUpvalue.as_u8(), 1, 0),
                encode_abc(OpCode::Add.as_u8(), 0, 0, 1),
                encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
            ],
            constants: Vec::new(),
            upvalue_info: vec![1, 0],
            line_info: vec![2; 3],
        };
        let main = Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 2,
            chunk: vec![
                encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
                encode_abx(OpCode::Closure.as_u8(), 1, 0),
                encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
            ],
            constants: vec![Value::int(5)],
            upvalue_info: Vec::new(),
            line_info: vec![1; 3],
        };
        CompiledProgram::new(
            PrimeId::Bn254,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![add_offset],
            main,
            HashMap::new(),
        )
    }

    unsafe extern "C" fn compiled_closure_entry(
        api: *const RuntimeApi,
        context: *mut c_void,
        frame_index: u32,
        base: u32,
        output: *mut u64,
    ) -> RuntimeStatus {
        let api = unsafe { &*api };
        for instruction in 0..2 {
            let status = unsafe { (api.execute_instruction)(context, frame_index, instruction) };
            if status != STATUS_OK {
                return status;
            }
        }
        unsafe { (api.load_register)(context, base, 1, output) }
    }

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

    #[test]
    fn standalone_aot_host_selects_explicit_cli_compatibility_authority() {
        let vm = configured_vm(captured_main_local_program()).unwrap();
        let granted = vm.host_policy.granted();
        assert!(granted.contains(CapabilitySet::CONSOLE_READ));
        assert!(granted.contains(CapabilitySet::CONSOLE_WRITE));
        assert!(granted.contains(CapabilitySet::CLOCK));
        assert!(granted.contains(CapabilitySet::RANDOM));
    }

    #[test]
    fn aot_host_closes_captured_main_locals() {
        let mut vm = VM::new();
        vm.load_program(captured_main_local_program()).unwrap();

        let (bailout, _) = execute_loaded(&mut vm, compiled_closure_entry).unwrap();

        assert_eq!(bailout, None);
        assert!(vm.frames.is_empty());
        assert!(vm.open_upvalues.is_none());
        let closure_value = vm.last_result;
        let closure = vm
            .heap
            .get_closure(closure_value.as_handle().unwrap())
            .unwrap();
        assert!(matches!(
            vm.heap.get_upvalue(closure.upvalues[0]).unwrap().location,
            UpvalueLocation::Closed(value) if value.as_int() == Some(5)
        ));
        assert_eq!(
            vm.call_value(closure_value, &[Value::int(7)]).unwrap(),
            Value::int(12)
        );
    }
}
