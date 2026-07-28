#![cfg(feature = "llvm")]

use std::collections::HashMap;
use std::env;
use std::process::Command;

use akron::opcode::instruction::{encode_abc, encode_abx};
use akron::{CompiledProgram, OpCode, VM};
use akron_llvm::{JitCacheConfig, JitEngine, LlvmTierOptions};
use memory::field::PrimeId;
use memory::{Function, Value};
use tempfile::TempDir;

const HELPER_ENV: &str = "AKRON_JIT_CACHE_PROCESS_HELPER";
const DIRECTORY_ENV: &str = "AKRON_JIT_CACHE_PROCESS_DIRECTORY";

fn program() -> CompiledProgram {
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
            max_slots: 3,
            chunk: vec![
                encode_abx(OpCode::LoadConst.as_u8(), 0, 0),
                encode_abx(OpCode::LoadConst.as_u8(), 1, 1),
                encode_abc(OpCode::Add.as_u8(), 2, 0, 1),
                encode_abc(OpCode::Return.as_u8(), 2, 1, 0),
            ],
            constants: vec![Value::int(20), Value::int(22)],
            upvalue_info: Vec::new(),
            line_info: vec![1; 4],
        },
        HashMap::new(),
    )
}

#[test]
fn disabled_cache_restores_uncached_behavior() {
    let engine = JitEngine::compile_with_cache(&program(), &JitCacheConfig::disabled()).unwrap();
    assert_eq!(engine.cache_stats().lookups, 0);
    assert_eq!(engine.cache_stats().hits, 0);
    assert_eq!(engine.cache_stats().misses, 0);
}

#[test]
fn tier_options_partition_cached_objects() {
    let directory = TempDir::new().unwrap();
    let config = JitCacheConfig::new(directory.path()).with_max_bytes(8 * 1024 * 1024);
    let compiled_program = program();

    let tier2 = JitEngine::compile_with_cache(&compiled_program, &config).unwrap();
    assert_eq!(tier2.cache_stats().misses, 1);

    let tier1 = JitEngine::compile_with_cache_and_options(
        &compiled_program,
        &config,
        LlvmTierOptions::tier1(),
    )
    .unwrap();
    assert_eq!(tier1.cache_stats().hits, 0);
    assert_eq!(tier1.cache_stats().misses, 1);

    let tier1_hit = JitEngine::compile_with_cache_and_options(
        &compiled_program,
        &config,
        LlvmTierOptions::tier1(),
    )
    .unwrap();
    assert_eq!(tier1_hit.cache_stats().hits, 1);
}

#[test]
fn process_cache_helper() {
    if env::var_os(HELPER_ENV).is_none() {
        return;
    }
    let directory = env::var_os(DIRECTORY_ENV).unwrap();
    let config = JitCacheConfig::new(directory).with_max_bytes(8 * 1024 * 1024);
    let compiled_program = program();
    let engine = JitEngine::compile_with_cache(&compiled_program, &config).unwrap();
    let stats = engine.cache_stats();
    let timings = engine.compile_timings();

    let mut vm = VM::new();
    vm.load_program(compiled_program).unwrap();
    let result = engine.execute(&mut vm).unwrap();
    assert_eq!(result.value.as_int(), Some(42));
    println!(
        "CACHE_STATS,{},{},{},{},{},{},{},{},{}",
        stats.lookups,
        stats.hits,
        stats.misses,
        timings.lowering.as_nanos(),
        engine.instruction_count(),
        engine.native_instruction_count(),
        engine.direct_instruction_count(),
        engine.runtime_instruction_count(),
        engine.compiled_call_count()
    );
}

#[test]
fn fresh_process_reuses_a_persistent_object() {
    let directory = TempDir::new().unwrap();
    let run = || {
        Command::new(env::current_exe().unwrap())
            .args(["--exact", "process_cache_helper", "--nocapture"])
            .env(HELPER_ENV, "1")
            .env(DIRECTORY_ENV, directory.path())
            .output()
            .unwrap()
    };

    let first = run();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_output = String::from_utf8_lossy(&first.stdout);
    assert!(first_output.contains("CACHE_STATS,1,0,1"));

    let second = run();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_output = String::from_utf8_lossy(&second.stdout);
    assert!(second_output.contains("CACHE_STATS,1,1,0,0,4,4,4,0,0"));
}
