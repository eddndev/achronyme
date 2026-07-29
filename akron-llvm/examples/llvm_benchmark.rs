#[cfg(not(all(feature = "llvm", feature = "aot")))]
compile_error!("run this benchmark with --features llvm,aot");

use std::collections::HashMap;
use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use akron_llvm::{lower_program, AotCompiler, AotOptions, JitEngine};
use memory::field::PrimeId;
use memory::{Function, Value};

fn loop_program(iterations: i64) -> CompiledProgram {
    CompiledProgram::new(
        PrimeId::Bn254,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Function {
            name: "main".to_string(),
            arity: 0,
            max_slots: 4,
            chunk: vec![
                encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
                encode_abx(OpCode::LoadConst.as_u8(), 1, 1),
                encode_abx(OpCode::LoadConst.as_u8(), 2, 2),
                encode_abc(OpCode::Add.as_u8(), 0, 0, 1),
                encode_abc(OpCode::Lt.as_u8(), 3, 0, 2),
                encode_abx(OpCode::JumpIfFalse.as_u8(), 3, 7),
                encode_abx(OpCode::Jump.as_u8(), 0, 3),
                encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
            ],
            constants: vec![Value::int(0), Value::int(1), Value::int(iterations)],
            upvalue_info: Vec::new(),
            line_info: vec![1; 8],
        },
        HashMap::new(),
    )
}

fn loaded_vm(program: &CompiledProgram) -> VM {
    let mut vm = VM::new();
    vm.load_program(program.clone()).unwrap();
    vm
}

fn interpreter_once(program: &CompiledProgram, expected: i64) {
    let mut vm = loaded_vm(program);
    vm.interpret().unwrap();
    assert_eq!(black_box(vm.last_result.as_int()), Some(expected));
}

fn jit_once(engine: &JitEngine, program: &CompiledProgram, expected: i64) {
    let mut vm = loaded_vm(program);
    let result = engine.execute(&mut vm).unwrap();
    assert_eq!(black_box(result.value.as_int()), Some(expected));
}

fn measure(mut operation: impl FnMut(), repetitions: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..repetitions {
        operation();
    }
    start.elapsed()
}

fn per_iteration(duration: Duration, repetitions: usize) -> u128 {
    duration.as_nanos() / repetitions as u128
}

fn runtime_archive() -> PathBuf {
    if let Some(path) = std::env::var_os("AKRON_AOT_RUNTIME_ARCHIVE") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/release/libakron_aot_runtime.a")
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let loop_iterations: i64 = arguments
        .next()
        .as_deref()
        .unwrap_or("100000")
        .parse()
        .expect("loop iterations must be an integer");
    let repetitions: usize = arguments
        .next()
        .as_deref()
        .unwrap_or("20")
        .parse()
        .expect("repetitions must be an integer");
    assert!(loop_iterations > 0 && repetitions > 0);
    let program = loop_program(loop_iterations);

    let start = Instant::now();
    let lowered = lower_program(black_box(&program)).unwrap();
    let lower_duration = start.elapsed();

    let start = Instant::now();
    let jit = JitEngine::compile(black_box(&program)).unwrap();
    let jit_compile_duration = start.elapsed();

    let jit_first = measure(|| jit_once(&jit, &program, loop_iterations), 1);
    let jit_warm = measure(|| jit_once(&jit, &program, loop_iterations), repetitions);
    let interpreter = measure(|| interpreter_once(&program, loop_iterations), repetitions);

    let runtime = runtime_archive();
    assert!(
        runtime.is_file(),
        "build release runtime: cargo build -p akron-aot-runtime --release"
    );
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("loop-native");
    let clang = std::env::var_os("AKRON_CLANG").unwrap_or_else(|| "clang".into());
    let start = Instant::now();
    let artifact = AotCompiler::new(AotOptions::new(clang, runtime))
        .build_executable(&program, &executable)
        .unwrap();
    let aot_compile_duration = start.elapsed();

    let run_aot = || {
        let output = Command::new(&executable).output().unwrap();
        assert!(output.status.success());
        black_box(output.stdout);
    };
    let aot_first = measure(run_aot, 1);
    let aot_warm = measure(run_aot, repetitions);

    println!("metric,value,unit");
    println!("loop_iterations,{loop_iterations},instructions_per_kernel_loop");
    println!("repetitions,{repetitions},runs");
    println!(
        "directly_lowered,{},{}/bytecode_instructions",
        artifact.native_instruction_count, artifact.instruction_count
    );
    println!("llvm_ir_bytes,{},bytes", lowered.ir.len());
    println!("lower_to_ir,{},ns", lower_duration.as_nanos());
    println!("jit_compile_orc,{},ns", jit_compile_duration.as_nanos());
    println!("jit_first_vm_setup_and_execute,{},ns", jit_first.as_nanos());
    println!(
        "jit_warm_vm_setup_and_execute,{},ns_per_run",
        per_iteration(jit_warm, repetitions)
    );
    println!(
        "interpreter_vm_setup_and_execute,{},ns_per_run",
        per_iteration(interpreter, repetitions)
    );
    println!(
        "aot_compile_and_link,{},ns",
        aot_compile_duration.as_nanos()
    );
    println!("aot_first_process,{},ns", aot_first.as_nanos());
    println!(
        "aot_warm_process,{},ns_per_run",
        per_iteration(aot_warm, repetitions)
    );
}
