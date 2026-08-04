use std::sync::atomic::AtomicBool;

use crate::compiled::{runtime_api, RuntimeContext, STATUS_RUNTIME_ERROR};
use crate::{RuntimeError, VM};

use super::*;

fn record() -> TaskRecord {
    TaskRecord {
        parent: None,
        owner_scope: 1,
        spawn_function: "main".to_string(),
        spawn_line: 1,
        last_function: Some("main".to_string()),
        last_line: Some(1),
        callee: Value::nil(),
        args: Vec::new(),
        state: TaskState::Completed(Value::nil()),
        cancel_requested: Arc::new(AtomicBool::new(false)),
        blocking_job: None,
        borrowed_resources: Vec::new(),
        close_on_terminal: Vec::new(),
        capture_failure: false,
        observed: true,
    }
}

#[test]
fn compiled_runtime_context_rejects_an_active_task_switch() {
    let mut vm = VM::new();
    vm.task_scheduler.current_tasks.push(7);
    let vm_pointer = std::ptr::from_mut(&mut vm);
    let mut context = RuntimeContext::new(&mut vm);

    unsafe {
        (*vm_pointer).task_scheduler.current_tasks.pop();
        (*vm_pointer).task_scheduler.current_tasks.push(9);
    }

    let mut output = 0;
    let status = unsafe { (runtime_api().load_register)(context.as_opaque(), 0, 0, &mut output) };
    assert_eq!(status, STATUS_RUNTIME_ERROR);
    let error = context.finish(status).unwrap_err();
    assert!(matches!(error, RuntimeError::TaskFailed(_)));
    assert!(
        error
            .to_string()
            .contains("compiled runtime changed active task from 7 to 9"),
        "{error}"
    );
}

#[test]
fn recycled_task_slots_reject_stale_generations() {
    let mut tasks = TaskTable::default();
    let first = tasks.reserve().unwrap();
    assert!(tasks.insert(first, record()));
    tasks.release_consumed(first).unwrap();
    assert!(tasks.was_consumed(first));

    let second = tasks.reserve().unwrap();
    assert_ne!(first, second);
    assert_eq!(first as u16, second as u16, "slot should be reused");
    assert!(tasks.insert(second, record()));
    assert!(tasks.get(first).is_none());
    assert!(tasks.get(second).is_some());
}

#[test]
fn completed_tasks_reuse_bounded_storage_across_many_generations() {
    let mut tasks = TaskTable::default();
    for _ in 0..50_000 {
        let handle = tasks.reserve().unwrap();
        assert!(tasks.insert(handle, record()));
        tasks.release(handle).unwrap();
    }
    assert_eq!(tasks.len(), 0);
    assert_eq!(tasks.slots.len(), 1);
}

#[test]
fn scheduler_has_an_explicit_running_root_task() {
    let scheduler = TaskScheduler::default();

    assert!(matches!(scheduler.root_task.state, TaskState::Running));
    assert_eq!(scheduler.root_task.parent, None);
}
