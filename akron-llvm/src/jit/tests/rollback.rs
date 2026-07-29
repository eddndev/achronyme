use akron::VM;

use super::specializations::list_specialization_program;
use super::{arithmetic_program, recursive_fibonacci_program};
use crate::{ExecutionOutcome, JitEngine, LlvmTierOptions};
use akron::OpCode;

#[test]
fn jit_tier2_capabilities_roll_back_independently_to_tier1_paths() {
    let arithmetic = arithmetic_program(OpCode::Add, 40, 2);
    let fast_poll = execute_with_disabled(
        &arithmetic,
        LlvmTierOptions {
            fast_poll: false,
            ..LlvmTierOptions::default()
        },
    );
    assert_eq!(fast_poll.value.as_int(), Some(42));
    assert_eq!(fast_poll.stats.fast_poll_hits, 0);
    assert!(fast_poll.stats.slow_poll_entries > 0);

    let recursive = recursive_fibonacci_program(12);
    let known_calls = execute_with_disabled(
        &recursive,
        LlvmTierOptions {
            known_calls: false,
            ..LlvmTierOptions::default()
        },
    );
    assert_eq!(known_calls.stats.known_call_fast_hits, 0);
    assert!(known_calls.stats.compiled_function_calls > 0);

    let list = list_specialization_program();
    let lists = execute_with_disabled(
        &list,
        LlvmTierOptions {
            list_specializations: false,
            ..LlvmTierOptions::default()
        },
    );
    assert_eq!(lists.value.as_int(), Some(7));
    assert_eq!(lists.stats.specialization_hits, 0);
    assert_eq!(lists.stats.specialization_misses, 0);
    assert_eq!(lists.stats.runtime_calls, 4);
}

fn execute_with_disabled(
    program: &akron::CompiledProgram,
    options: LlvmTierOptions,
) -> crate::ExecutionResult {
    let mut oracle = VM::new();
    oracle.load_program(program.clone()).unwrap();
    oracle.interpret().unwrap();

    let engine = JitEngine::compile_with_options(program, options).unwrap();
    let mut compiled = VM::new();
    compiled.load_program(program.clone()).unwrap();
    let result = engine.execute(&mut compiled).unwrap();

    assert_eq!(result.value, oracle.last_result);
    assert_eq!(compiled.instruction_budget, oracle.instruction_budget);
    assert_eq!(result.stats.interpreter_fallbacks, 0);
    result
}

#[test]
fn unsupported_opcode_uses_the_visible_interpreter_rollback() {
    let program = arithmetic_program(OpCode::Pow, 2, 3);
    let engine = JitEngine::compile(&program).unwrap();
    let mut vm = VM::new();
    vm.load_program(program).unwrap();

    let result = engine.execute(&mut vm).unwrap();

    assert_eq!(
        result.outcome,
        ExecutionOutcome::InterpreterBailout { instruction: 2 }
    );
    assert_eq!(result.value.as_int(), Some(8));
    assert_eq!(result.stats.interpreter_fallbacks, 1);
    assert_eq!(result.stats.first_fallback_instruction, Some(2));
    assert_eq!(vm.stack[2].as_int(), Some(8));
    assert!(vm.frames.is_empty());
    assert_eq!(vm.instruction_budget, u64::MAX.wrapping_sub(4));
}
