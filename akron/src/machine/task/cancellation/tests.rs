use super::super::state::{TaskContext, TaskWait, TaskWaitMode};
use super::*;
use crate::opcode::instruction::{encode_abc, encode_abx};
use crate::OpCode;
use memory::{Closure, Function, Value};

fn spawn_raw_task(vm: &mut VM, name: &str, chunk: Vec<u32>) -> u32 {
    spawn_raw_task_with_args(vm, name, 0, chunk, Vec::new())
}

fn spawn_raw_task_with_args(
    vm: &mut VM,
    name: &str,
    arity: u8,
    chunk: Vec<u32>,
    args: Vec<Value>,
) -> u32 {
    let line_info = vec![1; chunk.len()];
    let function = vm
        .heap
        .alloc_function(Function {
            name: name.to_string(),
            arity,
            max_slots: 1,
            chunk,
            constants: Vec::new(),
            upvalue_info: Vec::new(),
            line_info,
        })
        .unwrap();
    let closure = vm
        .heap
        .alloc_closure(Closure {
            function,
            upvalues: Vec::new(),
        })
        .unwrap();
    vm.spawn_task(Value::closure(closure), args)
        .unwrap()
        .as_task_handle()
        .unwrap()
}

fn returning_nil_chunk() -> Vec<u32> {
    vec![
        encode_abx(OpCode::LoadNil.as_u8(), 0, 0),
        encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
    ]
}

#[test]
fn cleanup_failure_is_attached_without_replacing_the_primary_failure() {
    let mut vm = VM::new();
    vm.enter_task_scope().unwrap();
    let primary = spawn_raw_task(&mut vm, "primary", returning_nil_chunk());
    let cancelled = spawn_raw_task(&mut vm, "cancelled", returning_nil_chunk());
    let waiter = spawn_raw_task(&mut vm, "waiter", returning_nil_chunk());
    vm.task_scheduler.tasks.get_mut(waiter).unwrap().state = TaskState::Waiting(
        TaskContext {
            stack: vec![Value::nil()],
            frames: Vec::new(),
            last_result: Value::nil(),
            instruction_budget: u64::MAX,
        },
        TaskWait {
            targets: vec![cancelled],
            destination: 0,
            mode: TaskWaitMode::Propagate,
        },
    );

    vm.store_task_outcome(primary, Err(RuntimeError::task_failed("primary failure")))
        .unwrap();

    let diagnostic = vm.last_task_failure().unwrap();
    assert_eq!(diagnostic.task.id, primary);
    assert!(diagnostic.message.contains("primary failure"));
    assert_eq!(diagnostic.cleanup_failures.len(), 1);
    assert!(diagnostic.cleanup_failures[0].contains(&format!("task {waiter}")));
    assert!(diagnostic.cleanup_failures[0].contains("task was cancelled"));
}

#[test]
fn cpu_bound_cancellation_latency_is_bounded_by_explicit_checks() {
    let mut late_vm = VM::new();
    late_vm.instruction_budget = 64;
    late_vm.enter_task_scope().unwrap();
    let mut late_chunk = vec![encode_abx(OpCode::LoadNil.as_u8(), 0, 0); 128];
    late_chunk.push(encode_abc(OpCode::CancelCheck.as_u8(), 0, 0, 0));
    late_chunk.push(encode_abc(OpCode::Return.as_u8(), 0, 1, 0));
    let late = spawn_raw_task(&mut late_vm, "late_check", late_chunk);
    late_vm
        .task_scheduler
        .tasks
        .get(late)
        .unwrap()
        .cancel_requested
        .store(true, Ordering::Release);

    let late_error = late_vm.drive_task(late).unwrap_err();
    assert!(
        late_error.to_string().contains("instruction budget"),
        "{late_error}"
    );

    let mut early_vm = VM::new();
    early_vm.instruction_budget = 64;
    early_vm.enter_task_scope().unwrap();
    let early = spawn_raw_task(
        &mut early_vm,
        "early_check",
        vec![
            encode_abc(OpCode::CancelCheck.as_u8(), 0, 0, 0),
            encode_abx(OpCode::LoadNil.as_u8(), 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
    );
    early_vm
        .task_scheduler
        .tasks
        .get(early)
        .unwrap()
        .cancel_requested
        .store(true, Ordering::Release);

    let early_error = early_vm.drive_task(early).unwrap_err();
    assert!(
        early_error.to_string().contains("cancelled"),
        "{early_error}"
    );
}

#[test]
fn scope_exit_closes_a_transferred_resource_exactly_once() {
    use memory::ValueResourceKind;

    let mut vm = VM::new();
    vm.enter_task_scope().unwrap();
    let handle = vm.reserve_resource(ValueResourceKind::Connection).unwrap();
    let resource = vm
        .resources
        .activate_network(handle, ValueResourceKind::Connection)
        .unwrap();
    let child = spawn_raw_task_with_args(
        &mut vm,
        "resource_owner",
        1,
        returning_nil_chunk(),
        vec![resource],
    );

    vm.drive_task(child).unwrap();
    assert_eq!(vm.resources.close_count(handle), 0);
    vm.exit_task_scope().unwrap();
    assert_eq!(vm.resources.close_count(handle), 1);

    vm.close_resource_handle(handle);
    assert_eq!(vm.resources.close_count(handle), 1);
}
