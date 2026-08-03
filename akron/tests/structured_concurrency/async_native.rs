fn native_delay(_vm: &mut VM, _args: &[memory::Value]) -> Result<memory::Value, RuntimeError> {
    Ok(memory::Value::int(1))
}

fn lock_async_pool_test() -> MutexGuard<'static, ()> {
    ASYNC_POOL_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn start_delay(_vm: &mut VM, _args: &[memory::Value]) -> Result<NativeAsyncRequest, RuntimeError> {
    Ok(NativeAsyncRequest::blocking(Box::new(|| {
        let active = ACTIVE_JOBS.fetch_add(1, Ordering::SeqCst) + 1;
        MAX_ACTIVE_JOBS.fetch_max(active, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(40));
        ACTIVE_JOBS.fetch_sub(1, Ordering::SeqCst);
        Ok(NativeAsyncOutput::Int(1))
    })))
}

fn gated_request(
    release: &'static AtomicBool,
    active: &'static AtomicUsize,
) -> Result<NativeAsyncRequest, RuntimeError> {
    Ok(NativeAsyncRequest::blocking(Box::new(move || {
        active.fetch_add(1, Ordering::AcqRel);
        while !release.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(1));
        }
        active.fetch_sub(1, Ordering::AcqRel);
        Ok(NativeAsyncOutput::Nil)
    })))
}

fn start_backpressure_gated(
    _vm: &mut VM,
    _args: &[memory::Value],
) -> Result<NativeAsyncRequest, RuntimeError> {
    gated_request(&BACKPRESSURE_RELEASE_JOBS, &BACKPRESSURE_ACTIVE_JOBS)
}

fn start_cancellation_gated(
    _vm: &mut VM,
    args: &[memory::Value],
) -> Result<NativeAsyncRequest, RuntimeError> {
    let mode = args
        .first()
        .and_then(memory::Value::as_int)
        .ok_or_else(|| RuntimeError::type_mismatch("cancellation_gate expects an integer mode"))?;
    match mode {
        0 => Ok(NativeAsyncRequest::blocking(Box::new(|| {
            CANCELLATION_JOB_STARTED.store(true, Ordering::Release);
            while !CANCELLATION_RELEASE_JOBS.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
            CANCELLATION_JOB_COMPLETED.store(true, Ordering::Release);
            Ok(NativeAsyncOutput::Nil)
        }))),
        1 => Ok(NativeAsyncRequest::blocking(Box::new(|| {
            while !CANCELLATION_JOB_STARTED.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(NativeAsyncOutput::Nil)
        }))),
        _ => Err(RuntimeError::type_mismatch(
            "cancellation_gate mode must be 0 or 1",
        )),
    }
}

fn start_pending_limit_gated(
    _vm: &mut VM,
    _args: &[memory::Value],
) -> Result<NativeAsyncRequest, RuntimeError> {
    gated_request(&PENDING_LIMIT_RELEASE_JOBS, &PENDING_LIMIT_ACTIVE_JOBS)
}

struct TestAsyncModule {
    name: &'static str,
    start: akron::NativeAsyncStart,
}

impl NativeModule for TestAsyncModule {
    fn name(&self) -> &'static str {
        "test_async"
    }

    fn natives(&self) -> Vec<NativeDef> {
        vec![NativeDef {
            name: self.name,
            func: native_delay,
            arity: 0,
            effects: EffectSet::TASK,
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Suspending,
            cancellation: CancellationPolicy::BeforeStart,
            resource: ResourceEffect::None,
            async_start: Some(self.start),
        }]
    }
}

fn async_native_meta(name: &'static str) -> NativeMeta {
    async_native_meta_with_arity(name, 0)
}

fn async_native_meta_with_arity(name: &'static str, arity: isize) -> NativeMeta {
    NativeMeta {
        name,
        arity,
        effects: EffectSet::TASK,
        capabilities: CapabilitySet::empty(),
        behavior: NativeBehavior::Suspending,
        cancellation: CancellationPolicy::BeforeStart,
        resource: ResourceEffect::None,
    }
}

struct CancellationAsyncModule;

impl NativeModule for CancellationAsyncModule {
    fn name(&self) -> &'static str {
        "test_cancellation_async"
    }

    fn natives(&self) -> Vec<NativeDef> {
        vec![NativeDef {
            name: "cancellation_gate",
            func: native_delay,
            arity: 1,
            effects: EffectSet::TASK,
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Suspending,
            cancellation: CancellationPolicy::BeforeStart,
            resource: ResourceEffect::None,
            async_start: Some(start_cancellation_gated),
        }]
    }
}

fn loaded_cancellation_vm(source: &str) -> VM {
    let mut compiler =
        Compiler::with_extra_natives(&[async_native_meta_with_arity("cancellation_gate", 1)]);
    let program = compiler
        .compile_program(source, &akronc::CompileOptions::default())
        .unwrap();
    let mut vm = VM::new();
    vm.register_module(&CancellationAsyncModule).unwrap();
    vm.load_program(program).unwrap();
    vm
}

fn loaded_async_vm(source: &str, name: &'static str, start: akron::NativeAsyncStart) -> VM {
    let mut compiler = Compiler::with_extra_natives(&[async_native_meta(name)]);
    let program = compiler
        .compile_program(source, &akronc::CompileOptions::default())
        .unwrap();
    let mut vm = VM::new();
    vm.register_module(&TestAsyncModule { name, start })
        .unwrap();
    vm.load_program(program).unwrap();
    vm
}

#[test]
fn blocking_pool_runs_spawned_host_jobs_concurrently() {
    let _pool_guard = lock_async_pool_test();
    ACTIVE_JOBS.store(0, Ordering::SeqCst);
    MAX_ACTIVE_JOBS.store(0, Ordering::SeqCst);
    let mut vm = loaded_async_vm(
        "let result = concurrent { let first = spawn delay(); let second = spawn delay(); await first + await second }",
        "delay",
        start_delay,
    );

    vm.interpret().unwrap();
    assert!(
        MAX_ACTIVE_JOBS.load(Ordering::SeqCst) >= 2,
        "both blocking jobs should overlap on the bounded pool"
    );
}

#[test]
fn awaiting_host_io_yields_the_vm_lane_to_a_ready_language_task() {
    let _pool_guard = lock_async_pool_test();
    ACTIVE_JOBS.store(0, Ordering::SeqCst);
    MAX_ACTIVE_JOBS.store(0, Ordering::SeqCst);
    let mut vm = loaded_async_vm(
        "mut order = 0\n\
         fn wait_then_check(value) { await delay(); assert(order == 1); value }\n\
         fn mark_ready() { order = 1 }\n\
         let result = concurrent { let waiting = spawn wait_then_check(\"preserved\"); spawn mark_ready(); await waiting }",
        "delay",
        start_delay,
    );
    vm.stress_mode = true;

    vm.interpret().unwrap();
    assert_eq!(global_int(&vm, 17), 1);
    let value = vm.globals[20].value;
    assert_eq!(
        vm.heap.get_string(value.as_handle().unwrap()).unwrap(),
        "preserved"
    );
}

#[test]
fn blocking_pool_applies_backpressure_when_workers_and_queue_are_full() {
    let _pool_guard = lock_async_pool_test();
    BACKPRESSURE_RELEASE_JOBS.store(false, Ordering::Release);
    BACKPRESSURE_ACTIVE_JOBS.store(0, Ordering::Release);
    let spawns = (0..80)
        .map(|_| "spawn gated()")
        .collect::<Vec<_>>()
        .join("; ");
    let source = format!("concurrent {{ {spawns} }}");
    let mut vm = loaded_async_vm(&source, "gated", start_backpressure_gated);
    let releaser = std::thread::spawn(|| {
        let deadline = Instant::now() + Duration::from_secs(2);
        while BACKPRESSURE_ACTIVE_JOBS.load(Ordering::Acquire) < 4 {
            assert!(Instant::now() < deadline, "blocking workers did not fill");
            std::thread::sleep(Duration::from_millis(1));
        }
        std::thread::sleep(Duration::from_millis(20));
        BACKPRESSURE_RELEASE_JOBS.store(true, Ordering::Release);
    });

    let result = vm.interpret();
    releaser.join().unwrap();
    let error = result.unwrap_err();
    assert!(
        matches!(error, RuntimeError::ResourceLimitExceeded(_)),
        "unexpected error: {error}"
    );
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn configured_pending_request_limit_is_a_hard_bound() {
    let _pool_guard = lock_async_pool_test();
    PENDING_LIMIT_RELEASE_JOBS.store(false, Ordering::Release);
    PENDING_LIMIT_ACTIVE_JOBS.store(0, Ordering::Release);
    let mut vm = loaded_async_vm(
        "concurrent { let first = spawn gated(); let second = spawn gated(); nil }",
        "gated",
        start_pending_limit_gated,
    );
    let mut limits = vm.runtime_limits;
    limits.max_pending_native_requests = 1;
    vm.set_runtime_limits(limits).unwrap();
    let releaser = std::thread::spawn(|| {
        let deadline = Instant::now() + Duration::from_secs(2);
        while PENDING_LIMIT_ACTIVE_JOBS.load(Ordering::Acquire) < 1 {
            assert!(Instant::now() < deadline, "pending-limit job did not start");
            std::thread::sleep(Duration::from_millis(1));
        }
        std::thread::sleep(Duration::from_millis(20));
        PENDING_LIMIT_RELEASE_JOBS.store(true, Ordering::Release);
    });

    let result = vm.interpret();
    releaser.join().unwrap();
    let error = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pending native requests exceed 1"),
        "{error}"
    );
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn sibling_failure_waits_for_running_host_io_before_scope_cleanup() {
    let _pool_guard = lock_async_pool_test();
    CANCELLATION_RELEASE_JOBS.store(false, Ordering::Release);
    CANCELLATION_JOB_STARTED.store(false, Ordering::Release);
    CANCELLATION_JOB_COMPLETED.store(false, Ordering::Release);
    let mut vm = loaded_cancellation_vm(
        "fn waiting() { await cancellation_gate(0); 1 }\n\
         fn fail_after_start() { await cancellation_gate(1); assert(false) }\n\
         concurrent { spawn fail_after_start(); spawn waiting() }",
    );
    let releaser = std::thread::spawn(|| {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !CANCELLATION_JOB_STARTED.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "host job did not start");
            std::thread::sleep(Duration::from_millis(1));
        }
        std::thread::sleep(Duration::from_millis(40));
        CANCELLATION_RELEASE_JOBS.store(true, Ordering::Release);
    });

    let result = vm.interpret();
    let completed_before_return = CANCELLATION_JOB_COMPLETED.load(Ordering::Acquire);
    releaser.join().unwrap();
    let error = result.unwrap_err();
    assert!(error.to_string().contains("task failed"), "{error}");
    assert!(
        completed_before_return,
        "scope cleanup returned while cancelled host work was still running"
    );
    assert_eq!(vm.active_task_scope_count(), 0);
}
