use std::net::{SocketAddr, TcpListener};
use std::path::Path;

use super::{
    assert_source_bytecode_parity, assert_source_bytecode_parity_with_grants, OracleGrants,
};

fn write_source(directory: &tempfile::TempDir, name: &str, source: &str) -> std::path::PathBuf {
    let path = directory.path().join(name);
    std::fs::write(&path, source).unwrap();
    path
}

fn assert_success(source: &Path, stdin: &str, grants: OracleGrants<'_>, marker: &str) {
    let (output, _) = assert_source_bytecode_parity_with_grants(source, stdin, grants);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(marker),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn oracle_structured_success_and_yield_match_every_engine() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(
        &directory,
        "structured-success.ach",
        r#"
            fn first() {
                await yield_now()
                20
            }
            fn second() { 22 }
            let result = concurrent {
                let first_task = spawn first()
                let second_task = spawn second()
                let left = await first_task
                let right = await second_task
                left + right
            }
            assert(result == 42)
            print("structured-success")
        "#,
    );

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("structured-success"));
}

#[test]
fn oracle_failure_outcome_and_loser_cancellation_match_every_engine() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(
        &directory,
        "failure-cancellation.ach",
        r#"
            fn slow() { await sleep(100); 99 }
            fn fail() { assert(false) }
            let outcome = concurrent {
                let slow_task = spawn slow()
                let failing_task = spawn fail()
                await [slow_task, failing_task] as race
            }
            assert(outcome["index"] == 1)
            assert(outcome["ok"] == false)
            print("failure-cancellation")
        "#,
    );

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("failure-cancellation"));
}

#[test]
fn oracle_bounded_channel_backpressure_matches_every_engine() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(
        &directory,
        "channel.ach",
        r#"
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
            assert(result == "hello world")
            print("channel-backpressure")
        "#,
    );

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("channel-backpressure"));
}

#[test]
fn oracle_timer_race_matches_every_engine() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(
        &directory,
        "timer.ach",
        r#"
            fn slow() { await sleep(50); 1 }
            fn fast() { await sleep(1); 2 }
            let outcome = concurrent {
                let slow_task = spawn slow()
                let fast_task = spawn fast()
                await [slow_task, fast_task] as race
            }
            assert(outcome["index"] == 1)
            assert(outcome["value"] == 2)
            print("timer-race")
        "#,
    );

    let (output, _) = assert_source_bytecode_parity(&source, "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("timer-race"));
}

#[test]
fn oracle_owned_file_io_matches_every_engine() {
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(
        &directory,
        "owned-file.ach",
        r#"
            let path = read_line()
            let writer = await create_file(path)
            let written = await file_write(writer, "owned-oracle")
            assert(written == 12)
            await file_close(writer)
            let reader = await open_file(path)
            let bytes = await file_read(reader, 64)
            assert(bytes != nil)
            await file_close(reader)
            assert(read_file(path) == "owned-oracle")
            print("owned-file")
        "#,
    );
    let destination = directory.path().join("oracle.txt");
    let stdin = format!("{}\n", destination.display());

    assert_success(
        &source,
        &stdin,
        OracleGrants {
            file_root: Some(directory.path()),
            ..OracleGrants::default()
        },
        "owned-file",
    );
    assert_eq!(
        std::fs::read_to_string(destination).unwrap(),
        "owned-oracle"
    );
}

#[test]
fn oracle_owned_tcp_echo_matches_every_engine() {
    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let address: SocketAddr = probe.local_addr().unwrap();
    drop(probe);
    let directory = tempfile::tempdir().unwrap();
    let source = write_source(
        &directory,
        "tcp-echo.ach",
        &format!(
            r#"
                fn server(listener) {{
                    let connection = await tcp_accept(listener)
                    let input = await tcp_read(connection, 5)
                    await tcp_write(connection, input)
                    await tcp_close(connection)
                    await tcp_listener_close(listener)
                }}
                fn client(address) {{
                    let connection = await tcp_connect(address)
                    await tcp_write(connection, "hello")
                    let echoed = await tcp_read(connection, 5)
                    await tcp_close(connection)
                    echoed
                }}
                let listener = await tcp_listen("{address}")
                let echoed = concurrent {{
                    spawn server(listener)
                    let client_task = spawn client("{address}")
                    await client_task
                }}
                assert(echoed != nil)
                print("tcp-echo")
            "#
        ),
    );

    assert_success(
        &source,
        "",
        OracleGrants {
            connect: Some(address),
            listen: Some(address),
            ..OracleGrants::default()
        },
        "tcp-echo",
    );
}
