#[cfg(feature = "io")]
#[test]
fn tcp_echo_uses_one_reactor_and_transfers_listener_ownership() {
    use akronc::CompileOptions;

    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = probe.local_addr().unwrap();
    drop(probe);
    let source = format!(
        r#"
        fn server(listener) {{
            let connection = await tcp_accept(listener)
            let input = await tcp_read(connection, 5)
            await tcp_write(connection, input)
            await tcp_close(connection)
        }}
        fn client(address) {{
            let connection = await tcp_connect(address)
            await tcp_write(connection, "hello")
            let echoed = await tcp_read(connection, 5)
            await tcp_close(connection)
            echoed
        }}
        let listener = await tcp_listen("{address}")
        let result = concurrent {{
            spawn server(listener)
            let client_task = spawn client("{address}")
            await client_task
        }}
        "#
    );
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();
    let result = compiler.global_symbols["result"].index as usize;

    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_listen_addr(address);
    vm.host_policy.allow_connect_addr(address);
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();

    let value = vm.globals[result].value;
    assert!(value.is_bytes());
    assert_eq!(
        vm.heap.get_bytes(value.as_handle().unwrap()).unwrap(),
        b"hello"
    );
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[cfg(feature = "io")]
#[test]
fn recovered_connection_failure_keeps_the_accept_loop_alive() {
    use akronc::CompileOptions;

    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = probe.local_addr().unwrap();
    drop(probe);
    let source = format!(
        r#"
        fn fail_connection(connection) {{
            await tcp_read(connection, 1)
            assert(false)
        }}
        fn echo_connection(connection) {{
            let input = await tcp_read(connection, 5)
            await tcp_write(connection, input)
            await tcp_close(connection)
        }}
        fn server(listener) {{
            let first = await tcp_accept(listener)
            let failed = concurrent {{
                let handler = spawn fail_connection(first)
                await handler as outcome
            }}
            assert(failed["ok"] == false)

            let second = await tcp_accept(listener)
            concurrent {{
                let handler = spawn echo_connection(second)
                await handler
            }}
            await tcp_listener_close(listener)
        }}
        fn failing_client(address) {{
            let connection = await tcp_connect(address)
            await tcp_write(connection, "x")
            await tcp_close(connection)
        }}
        fn echo_client(address) {{
            let connection = await tcp_connect(address)
            await tcp_write(connection, "hello")
            let echoed = await tcp_read(connection, 5)
            await tcp_close(connection)
            echoed
        }}
        let listener = await tcp_listen("{address}")
        let result = concurrent {{
            let server_task = spawn server(listener)
            let failed_client = spawn failing_client("{address}")
            await failed_client
            let healthy_client = spawn echo_client("{address}")
            let echoed = await healthy_client
            await server_task
            echoed
        }}
        "#
    );
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();
    let result = compiler.global_symbols["result"].index as usize;

    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_listen_addr(address);
    vm.host_policy.allow_connect_addr(address);
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();

    let value = vm.globals[result].value;
    assert!(value.is_bytes());
    assert_eq!(
        vm.heap.get_bytes(value.as_handle().unwrap()).unwrap(),
        b"hello"
    );
    assert!(vm.last_task_failure().is_none());
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[cfg(feature = "io")]
#[test]
fn slow_clients_stay_within_server_task_resource_and_buffer_bounds() {
    use std::io::Write;
    use std::net::TcpStream;
    use std::time::{Duration, Instant};

    use akronc::CompileOptions;

    const CLIENT_COUNT: usize = 24;
    const HANDLER_LIMIT: usize = 2;

    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = probe.local_addr().unwrap();
    drop(probe);
    let source = format!(
        r#"
        mut active = 0
        mut peak = 0
        mut handled = 0

        fn handle(connection, server) {{
            active = active + 1
            if active > peak {{ peak = active }}
            await tcp_read(connection, 1)
            await tcp_close(connection)
            active = active - 1
            handled = handled + 1
            await permit_release(server)
        }}

        fn serve(listener, count) {{
            let server = bounded_server({HANDLER_LIMIT})
            concurrent {{
                mut accepted = 0
                while accepted < count {{
                    await permit_acquire(server)
                    let connection = await tcp_accept(listener)
                    spawn handle(connection, server)
                    accepted = accepted + 1
                }}
            }}
            await tcp_listener_close(listener)
        }}

        let listener = await tcp_listen("{address}")
        await serve(listener, {CLIENT_COUNT})
        assert(active == 0)
        assert(peak == {HANDLER_LIMIT})
        assert(handled == {CLIENT_COUNT})
        "#
    );
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();
    let peak = compiler.global_symbols["peak"].index as usize;
    let handled = compiler.global_symbols["handled"].index as usize;

    let clients = (0..CLIENT_COUNT)
        .map(|_| {
            std::thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(3);
                let mut stream = loop {
                    match TcpStream::connect(address) {
                        Ok(stream) => break stream,
                        Err(error) if Instant::now() < deadline => {
                            let _ = error;
                            std::thread::sleep(Duration::from_millis(2));
                        }
                        Err(error) => panic!("slow client could not connect: {error}"),
                    }
                };
                std::thread::sleep(Duration::from_millis(80));
                stream.write_all(b"x").unwrap();
            })
        })
        .collect::<Vec<_>>();

    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_listen_addr(address);
    let mut limits = vm.runtime_limits;
    limits.max_tasks = 8;
    limits.max_resources = 8;
    limits.max_task_scopes = 8;
    limits.max_pending_native_requests = 4;
    limits.max_channels = 1;
    limits.max_channel_operations = 2;
    vm.set_runtime_limits(limits).unwrap();
    vm.load_program(program).unwrap();
    let result = vm.interpret();

    for client in clients {
        client.join().unwrap();
    }
    result.unwrap();
    assert_eq!(vm.globals[peak].value.as_int(), Some(HANDLER_LIMIT as i64));
    assert_eq!(
        vm.globals[handled].value.as_int(),
        Some(CLIENT_COUNT as i64)
    );
    assert_eq!(vm.live_task_count(), 0);
    assert_eq!(vm.active_task_scope_count(), 0);
}

#[cfg(feature = "io")]
#[test]
fn tcp_connect_reports_deferred_socket_errors() {
    use akronc::CompileOptions;

    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = probe.local_addr().unwrap();
    drop(probe);

    let source = format!("await tcp_connect(\"{address}\")");
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();

    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_connect_addr(address);
    vm.load_program(program).unwrap();

    let error = vm.interpret().unwrap_err().to_string();
    assert!(error.contains("tcp connect failed"), "{error}");
    assert_eq!(vm.active_task_scope_count(), 0);
}
