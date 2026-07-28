use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

use akron::compiled::{
    runtime_api, CompiledEntry, RuntimeCapabilities, RuntimeContext, RUNTIME_ABI_VERSION,
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
    let mut vm = VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module)
            .map_err(|error| error.to_string())?;
    }
    apply_environment(&mut vm)?;
    vm.load_program(program)
        .map_err(|error| format!("program loader error: {error}"))?;

    runtime_api()
        .validate(
            RUNTIME_ABI_VERSION,
            std::mem::size_of::<akron::compiled::RuntimeApi>() as u32,
            RuntimeCapabilities::LLVM_TIER1,
        )
        .map_err(|error| error.to_string())?;
    if vm.frames.len() != 1 {
        return Err(format!(
            "compiled program requires one main frame, found {}",
            vm.frames.len()
        ));
    }

    let base =
        u32::try_from(vm.frames[0].base).map_err(|_| "main frame base exceeds u32".to_string())?;
    let mut result_bits = Value::nil().to_abi_bits();
    let mut context = RuntimeContext::new(&mut vm);
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
    if let Err(error) = context.finish(status) {
        return Err(format_runtime_error(&vm, &error));
    }
    let result = Value::from_abi_bits(result_bits)
        .ok_or_else(|| "compiled entry returned noncanonical Value bits".to_string())?;
    vm.last_result = if bailout_ip.is_some() {
        vm.last_result
    } else {
        result
    };
    vm.frames.pop();

    if std::env::var_os("AKRON_ENGINE_TRACE").is_some() {
        match bailout_ip {
            Some(instruction) => eprintln!(
                "LLVM AOT bailout: resumed interpreter at bytecode instruction {instruction}"
            ),
            None => eprintln!("LLVM AOT native: program completed without interpreter bailout"),
        }
    }
    println!("Exit Status: {}", vm.val_to_string(&vm.last_result));
    Ok(())
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
