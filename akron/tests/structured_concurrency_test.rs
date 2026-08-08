use akron::module::{NativeDef, NativeModule};
use akron::specs::{
    CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, NativeMeta, ResourceEffect,
};
use akron::{NativeAsyncOutput, NativeAsyncRequest};
use akron::{RuntimeError, TaskDiagnosticState, VM};
use akronc::Compiler;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

mod support;

use support::{global_int, loaded_vm, run};

#[test]
fn await_returns_child_value_and_consumes_handle() {
    let vm = run("fn add(left, right) { left + right }\n\
         let result = concurrent { let task = spawn add(20, 22); await task }")
    .unwrap();

    assert_eq!(global_int(&vm, 17), 42);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn awaiting_a_call_creates_an_implicit_single_child_scope() {
    let vm = run("fn answer() { 42 }\nlet result = await answer()").unwrap();

    assert_eq!(global_int(&vm, 17), 42);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn scope_exit_joins_unawaited_children_in_spawn_order() {
    let vm = run("mut count = 0\n\
         fn increment() { count = count + 1 }\n\
         concurrent { spawn increment(); spawn increment() }")
    .unwrap();

    assert_eq!(global_int(&vm, 16), 2);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn awaiting_a_later_child_still_runs_ready_tasks_fifo() {
    let vm = run("mut order = 0\n\
         fn first() { order = 1; 10 }\n\
         fn second() { assert(order == 1); 20 }\n\
         let result = concurrent { let first_task = spawn first(); let second_task = spawn second(); await second_task }")
    .unwrap();

    assert_eq!(global_int(&vm, 16), 1);
    assert_eq!(global_int(&vm, 19), 20);
}

#[test]
fn first_child_failure_cancels_remaining_children_and_cleans_scope() {
    let mut vm = loaded_vm(
        "mut count = 0\n\
         fn fail() { assert(false) }\n\
         fn increment() { count = count + 1 }\n\
         concurrent { spawn fail(); spawn increment() }",
        false,
    )
    .unwrap();

    let error = vm.interpret().unwrap_err();
    assert!(error.to_string().contains("task failed"), "{error}");
    assert_eq!(global_int(&vm, 16), 0);
    assert_eq!(vm.active_task_scope_count(), 0);

    let failure = vm
        .last_task_failure()
        .expect("failed task remains available after deterministic cleanup");
    assert_eq!(failure.task.parent, None);
    assert_eq!(failure.task.state, TaskDiagnosticState::Failed);
    assert_eq!(failure.task.spawn_function, "main");
    assert!(failure.task.spawn_line > 0);
    assert_eq!(failure.task.last_function.as_deref(), Some("fail"));
    assert!(failure.message.contains("assertion failed"));
    assert!(failure.cleanup_failures.is_empty());
}

#[test]
fn await_as_outcome_recovers_child_failure_inside_a_server_scope() {
    let vm = run("fn fail() { assert(false) }\n\
         fn answer() { 42 }\n\
         let result = concurrent {\n\
             let failed = spawn fail();\n\
             let outcome = await failed as outcome;\n\
             assert(outcome[\"ok\"] == false);\n\
             assert(typeof(outcome[\"error\"]) == \"String\");\n\
             let healthy = spawn answer();\n\
             await healthy\n\
         }")
    .unwrap();

    assert_eq!(global_int(&vm, 18), 42);
    assert_eq!(vm.active_task_scope_count(), 0);
    assert!(vm.last_task_failure().is_none());
}

#[test]
fn nested_failure_diagnostic_preserves_the_task_parent_chain() {
    let mut vm = loaded_vm(
        "fn fail() { assert(false) }\n\
         fn parent() { concurrent { let child = spawn fail(); await child } }\n\
         concurrent { let parent_task = spawn parent(); await parent_task }",
        false,
    )
    .unwrap();

    vm.interpret().unwrap_err();
    let failure = vm.last_task_failure().unwrap();
    assert!(failure.task.parent.is_some());
    assert_eq!(failure.task.last_function.as_deref(), Some("fail"));
    assert!(failure.to_string().contains("spawned at"));
}

#[test]
fn child_task_can_recover_a_nested_failure_and_resume_after_suspension() {
    let vm = run("fn fail() { assert(false) }\n\
         fn supervise_one() {\n\
             concurrent {\n\
                 let child = spawn fail();\n\
                 let outcome = await child as outcome;\n\
                 if outcome[\"ok\"] { 0 } else { 7 }\n\
             }\n\
         }\n\
         let result = concurrent {\n\
             let supervisor = spawn supervise_one();\n\
             await supervisor\n\
         }")
    .unwrap();

    assert_eq!(global_int(&vm, vm.globals.len() - 1), 7);
}

#[test]
fn long_lived_scope_reaps_consumed_and_unobserved_children() {
    let vm = run("mut count = 0\n\
         fn increment() { count = count + 1 }\n\
         concurrent {\n\
             mut index = 0;\n\
             while index < 55000 {\n\
                 spawn increment();\n\
                 let observed = spawn increment();\n\
                 await observed;\n\
                 index = index + 1\n\
             };\n\
             index\n\
         }")
    .unwrap();

    assert_eq!(global_int(&vm, 16), 110_000);
    assert_eq!(vm.live_task_count(), 0);
}

#[test]
fn configured_live_task_limit_is_a_hard_bound() {
    let mut vm = loaded_vm(
        "fn wait() { 1 }\n\
         concurrent { spawn wait(); spawn wait() }",
        false,
    )
    .unwrap();
    let mut limits = vm.runtime_limits;
    limits.max_tasks = 1;
    vm.set_runtime_limits(limits).unwrap();

    let error = vm.interpret().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("live child task count exceeds 1"),
        "{error}"
    );
}

#[test]
fn configured_task_limit_counts_explicit_and_implicit_children() {
    let _pool_guard = lock_async_pool_test();
    let mut vm = loaded_async_vm(
        "fn wait_once() { await delay() }\n\
         concurrent {\n\
             let first = spawn wait_once();\n\
             let second = spawn wait_once();\n\
             await first;\n\
             await second\n\
         }",
        "delay",
        start_delay,
    );
    let mut limits = vm.runtime_limits;
    limits.max_tasks = 2;
    vm.set_runtime_limits(limits).unwrap();

    let error = vm.interpret().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("live child task count exceeds 2"),
        "{error}"
    );
}

#[test]
fn configured_task_scope_limit_counts_simultaneously_live_scopes() {
    let _pool_guard = lock_async_pool_test();
    let mut vm = loaded_async_vm(
        "fn wait_once() { await delay() }\n\
         concurrent {\n\
             let first = spawn wait_once();\n\
             let second = spawn wait_once();\n\
             await first;\n\
             await second\n\
         }",
        "delay",
        start_delay,
    );
    let mut limits = vm.runtime_limits;
    limits.max_task_scopes = 2;
    vm.set_runtime_limits(limits).unwrap();

    let error = vm.interpret().unwrap_err();
    assert!(
        error.to_string().contains("live task scopes exceed 2"),
        "{error}"
    );
}

#[test]
fn configured_retained_result_limit_is_a_hard_bound() {
    let mut vm = loaded_vm(
        "fn answer() { 42 }\n\
         concurrent { let first = spawn answer(); let second = spawn answer(); nil }",
        false,
    )
    .unwrap();
    let mut limits = vm.runtime_limits;
    limits.max_retained_task_results = 1;
    vm.set_runtime_limits(limits).unwrap();

    let error = vm.interpret().unwrap_err();
    assert!(
        error.to_string().contains("retained task results exceed 1"),
        "{error}"
    );
}

#[test]
fn task_instruction_budgets_are_isolated_across_context_switches() {
    let mut sequential = loaded_vm(
        "fn work() {\n\
             mut total = 0;\n\
             mut index = 0;\n\
             while index < 8 { total = total + index; index = index + 1 };\n\
             total\n\
         }\n\
         let first = work();\n\
         let second = work();\n\
         let result = first + second",
        false,
    )
    .unwrap();
    sequential.instruction_budget = 200;
    assert!(matches!(
        sequential.interpret(),
        Err(RuntimeError::InstructionBudgetExhausted)
    ));

    let mut vm = loaded_vm(
        "fn work() {\n\
             mut total = 0;\n\
             mut index = 0;\n\
             while index < 8 { total = total + index; index = index + 1 };\n\
             total\n\
         }\n\
         let result = concurrent {\n\
             let first = spawn work();\n\
             let second = spawn work();\n\
             await first + await second\n\
         }",
        false,
    )
    .unwrap();
    vm.instruction_budget = 200;

    vm.interpret().unwrap();
    assert_eq!(global_int(&vm, vm.globals.len() - 1), 56);
}

#[test]
fn task_handles_cannot_escape_their_lexical_scope() {
    let mut vm = loaded_vm(
        "fn answer() { 42 }\n\
         let escaped = concurrent { spawn answer() }\n\
         await escaped",
        false,
    )
    .unwrap();

    let error = vm.interpret().unwrap_err();
    assert!(
        matches!(
            error,
            RuntimeError::TaskOutOfScope | RuntimeError::InvalidTaskHandle
        ),
        "{error:?}"
    );
}

#[test]
fn task_can_only_be_awaited_once() {
    let mut vm = loaded_vm(
        "fn answer() { 42 }\n\
         concurrent { let task = spawn answer(); await task; await task }",
        false,
    )
    .unwrap();

    assert!(matches!(
        vm.interpret(),
        Err(RuntimeError::TaskAlreadyAwaited)
    ));
}

#[test]
fn suspended_task_values_are_gc_roots() {
    let mut vm = loaded_vm(
        "fn echo(value) { value }\n\
         let result = concurrent { let task = spawn echo(\"kept alive\"); await task }",
        true,
    )
    .unwrap();
    vm.interpret().unwrap();
    let value = vm.globals[17].value;
    assert!(value.is_string());
    assert_eq!(
        vm.heap.get_string(value.as_handle().unwrap()).unwrap(),
        "kept alive"
    );
}

static ACTIVE_JOBS: AtomicUsize = AtomicUsize::new(0);
static MAX_ACTIVE_JOBS: AtomicUsize = AtomicUsize::new(0);
static BACKPRESSURE_RELEASE_JOBS: AtomicBool = AtomicBool::new(false);
static BACKPRESSURE_ACTIVE_JOBS: AtomicUsize = AtomicUsize::new(0);
static PENDING_LIMIT_RELEASE_JOBS: AtomicBool = AtomicBool::new(false);
static PENDING_LIMIT_ACTIVE_JOBS: AtomicUsize = AtomicUsize::new(0);
static CANCELLATION_RELEASE_JOBS: AtomicBool = AtomicBool::new(false);
static CANCELLATION_JOB_STARTED: AtomicBool = AtomicBool::new(false);
static CANCELLATION_JOB_COMPLETED: AtomicBool = AtomicBool::new(false);
static ASYNC_POOL_TEST_LOCK: Mutex<()> = Mutex::new(());

include!("structured_concurrency/async_native.rs");
