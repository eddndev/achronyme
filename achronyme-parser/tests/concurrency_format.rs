use achronyme_parser::{format_program, parse_program};

fn parsed(source: &str) -> achronyme_parser::ast::Program {
    let (program, diagnostics) = parse_program(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    program
}

#[test]
fn accepted_rfc_examples_parse() {
    for source in [
        r#"
fn handle(connection) {
    let request = await connection.read()
    await connection.write(make_response(request))
}

fn serve(listener) {
    concurrent {
        forever {
            let connection = await listener.accept()
            spawn handle(connection)
        }
    }
}
"#,
        r#"
concurrent {
    let account = spawn fetch_account(account_id)
    let policy = spawn fetch_policy(account_id)
    combine(await account, await policy)
}
"#,
    ] {
        parsed(source);
    }
}

#[test]
fn concurrency_formatter_is_golden_round_trip_and_idempotent() {
    let source = r#"
fn coordinate(worker, signal) {
  concurrent {
    // Comments are accepted even though the AST does not retain trivia.
    let first=spawn worker(1)
    let second = spawn worker(2)
    if signal { return await [first,second] as race } else {
      forever { await signal as outcome; break }
    }
  }
}
"#;
    let expected = r#"fn coordinate(worker, signal) {
    concurrent {
        let first = spawn worker(1)
        let second = spawn worker(2)
        if signal {
            return await [first, second] as race
        } else {
            forever {
                await signal as outcome
                break
            }
        }
    }
}
"#;

    let formatted = format_program(&parsed(source)).unwrap();
    assert_eq!(formatted, expected);
    let reparsed = parsed(&formatted);
    assert_eq!(format_program(&reparsed).unwrap(), formatted);
}

#[test]
fn formatter_preserves_await_prefix_precedence() {
    let source = "concurrent { return await task + 1 }";
    let formatted = format_program(&parsed(source)).unwrap();
    assert!(formatted.contains("return await task + 1"), "{formatted}");
    parsed(&formatted);
}
