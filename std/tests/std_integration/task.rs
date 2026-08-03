#[test]
fn std_modules_count() {
    let modules = achronyme_std::std_modules();
    // conv + string_ext + task + compatibility I/O + owned files + TCP
    #[cfg(feature = "io")]
    assert_eq!(modules.len(), 6);
    #[cfg(not(feature = "io"))]
    assert_eq!(modules.len(), 3);
}

#[test]
fn bounded_channel_coordinates_sibling_tasks_without_os_threads() {
    use akronc::CompileOptions;

    let source = r#"
        fn producer(messages) {
            await channel_send(messages, "hello")
            await channel_send(messages, " world")
        }
        let messages = channel(1)
        let result = concurrent {
            spawn producer(messages)
            let first = await channel_receive(messages)
            let second = await channel_receive(messages)
            first + second
        }
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let result = compiler.global_symbols["result"].index as usize;
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();

    let value = vm.globals[result].value;
    assert_eq!(
        vm.heap.get_string(value.as_handle().unwrap()).unwrap(),
        "hello world"
    );
    assert_eq!(vm.live_task_count(), 0);
}

#[test]
fn channel_deadlock_fails_explicitly_instead_of_blocking_the_vm() {
    use akronc::CompileOptions;

    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(
            "let messages = channel(1)\nawait channel_receive(messages)",
            &CompileOptions::default(),
        )
        .unwrap();
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.load_program(program).unwrap();
    let error = vm.interpret().unwrap_err();
    assert!(error.to_string().contains("scheduler stalled"), "{error}");
}

#[test]
fn awaited_yield_gives_fifo_siblings_a_turn() {
    use akronc::CompileOptions;

    let source = r#"
        mut order = 0
        fn first() {
            order = 1
            await yield_now()
            assert(order == 2)
        }
        fn second() { order = 2 }
        concurrent {
            spawn first()
            spawn second()
        }
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let order = compiler.global_symbols["order"].index as usize;
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.globals[order].value.as_int(), Some(2));
}

#[test]
fn permit_pool_enforces_a_hard_active_child_limit() {
    use akronc::CompileOptions;

    let source = r#"
        mut active = 0
        mut peak = 0
        fn worker(permits) {
            await permit_acquire(permits)
            active = active + 1
            if active > peak { peak = active }
            assert(active <= 2)
            await yield_now()
            active = active - 1
            await permit_release(permits)
        }
        let permits = permit_pool(2)
        concurrent {
            spawn worker(permits)
            spawn worker(permits)
            spawn worker(permits)
            spawn worker(permits)
            spawn worker(permits)
        }
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let peak = compiler.global_symbols["peak"].index as usize;
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.globals[peak].value.as_int(), Some(2));
}

#[test]
fn bounded_server_helper_enforces_its_handler_limit() {
    use akron::specs::{ResourceEffect, ResourceKind};
    use akronc::CompileOptions;

    let source = r#"
        mut active = 0
        mut peak = 0
        fn worker(server) {
            await permit_acquire(server)
            active = active + 1
            if active > peak { peak = active }
            await yield_now()
            active = active - 1
            await permit_release(server)
        }
        let server = bounded_server(2)
        concurrent {
            spawn worker(server)
            spawn worker(server)
            spawn worker(server)
            spawn worker(server)
            spawn worker(server)
        }
    "#;
    let table = achronyme_std::std_native_table();
    let helper = table
        .iter()
        .find(|meta| meta.name == "bounded_server")
        .expect("bounded_server metadata");
    assert_eq!(
        helper.resource,
        ResourceEffect::Creates(ResourceKind::Channel)
    );
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let peak = compiler.global_symbols["peak"].index as usize;
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.globals[peak].value.as_int(), Some(2));
}

#[test]
fn task_race_returns_the_first_outcome_and_reaps_the_timer_loser() {
    use akronc::CompileOptions;

    let source = r#"
        fn slow() { await sleep(50); 1 }
        fn fast() { await yield_now(); 2 }
        let result = concurrent {
            let slow_task = spawn slow()
            let fast_task = spawn fast()
            await [slow_task, fast_task] as race
        }
        assert(result["index"] == 1)
        assert(result["ok"] == true)
        assert(result["value"] == 2)
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.grant(akron::specs::CapabilitySet::CLOCK);
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.live_task_count(), 0);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn task_race_observes_the_earliest_timer_instead_of_registration_order() {
    use akronc::CompileOptions;

    let source = r#"
        fn slow() { await sleep(50); 1 }
        fn fast() { await sleep(1); 2 }
        let result = concurrent {
            let slow_task = spawn slow()
            let fast_task = spawn fast()
            await [slow_task, fast_task] as race
        }
        assert(result["index"] == 1)
        assert(result["ok"] == true)
        assert(result["value"] == 2)
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.grant(akron::specs::CapabilitySet::CLOCK);
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.live_task_count(), 0);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn nested_task_race_resumes_its_parent_after_a_suspended_child_wins() {
    use akronc::CompileOptions;

    let source = r#"
        fn left() { await sleep(30); 4 }
        fn right() { await yield_now(); 9 }
        fn supervise() {
            concurrent {
                let left_task = spawn left()
                let right_task = spawn right()
                let winner = await [left_task, right_task] as race
                winner["value"]
            }
        }
        let result = concurrent {
            let supervisor = spawn supervise()
            await supervisor
        }
        assert(result == 9)
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.grant(akron::specs::CapabilitySet::CLOCK);
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.live_task_count(), 0);
}

#[test]
fn timeout_after_composes_with_race_and_cancels_unfinished_work() {
    use akronc::CompileOptions;

    let source = r#"
        fn work() { await sleep(100); 42 }
        let result = concurrent {
            let work_task = spawn work()
            await [work_task, timeout_after(1)] as race
        }
        assert(result["index"] == 1)
        assert(result["ok"] == true)
        assert(result["value"] == nil)
    "#;
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(source, &CompileOptions::default())
        .unwrap();
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.grant(akron::specs::CapabilitySet::CLOCK);
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();
    assert_eq!(vm.live_task_count(), 0);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[test]
fn std_native_table_matches_modules() {
    let table = achronyme_std::std_native_table();
    let modules = achronyme_std::std_modules();

    let mut expected_names: Vec<&str> = Vec::new();
    for module in &modules {
        for def in module.natives() {
            expected_names.push(def.name);
        }
    }

    assert_eq!(table.len(), expected_names.len());
    for (meta, expected) in table.iter().zip(expected_names.iter()) {
        assert_eq!(meta.name, *expected);
    }
}
#[test]
fn io_native_metadata_is_precise() {
    use akron::specs::{CapabilitySet, EffectSet, NativeBehavior};

    let table = achronyme_std::std_native_table();
    let read_line = table
        .iter()
        .find(|meta| meta.name == "read_line")
        .expect("read_line metadata");
    assert_eq!(read_line.effects, EffectSet::IO_CONSOLE);
    assert_eq!(read_line.capabilities, CapabilitySet::CONSOLE_READ);
    assert_eq!(read_line.behavior, NativeBehavior::Blocking);

    let read_file = table
        .iter()
        .find(|meta| meta.name == "read_file")
        .expect("read_file metadata");
    assert_eq!(read_file.effects, EffectSet::IO_FILE);
    assert_eq!(read_file.capabilities, CapabilitySet::FILE_READ);
    assert_eq!(read_file.behavior, NativeBehavior::Blocking);

    let write_file = table
        .iter()
        .find(|meta| meta.name == "write_file")
        .expect("write_file metadata");
    assert_eq!(write_file.effects, EffectSet::IO_FILE);
    assert_eq!(write_file.capabilities, CapabilitySet::FILE_WRITE);
    assert_eq!(write_file.behavior, NativeBehavior::Blocking);
}
