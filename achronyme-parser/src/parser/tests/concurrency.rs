use super::*;

#[test]
fn parses_concurrent_spawn_and_await() {
    let program = parse_ok(
        r#"
        concurrent {
            let task = spawn fetch("/health");
            await task
        }
        "#,
    );

    let Stmt::Expr(Expr::Concurrent { body, span, .. }) = &program.stmts[0] else {
        panic!("expected concurrent expression, got {:?}", program.stmts[0]);
    };
    assert_eq!(span.line_start, 2);
    assert_eq!(span.line_end, 5);

    let Stmt::LetDecl { value, .. } = &body.stmts[0] else {
        panic!("expected task declaration, got {:?}", body.stmts[0]);
    };
    let Expr::Spawn { call, .. } = value else {
        panic!("expected spawn expression, got {value:?}");
    };
    assert!(matches!(call.as_ref(), Expr::Call { .. }));

    let Stmt::Expr(Expr::Await { task, .. }) = &body.stmts[1] else {
        panic!("expected await expression, got {:?}", body.stmts[1]);
    };
    assert!(matches!(task.as_ref(), Expr::Ident { name, .. } if name == "task"));
}

#[test]
fn parses_recoverable_await_outcome_without_hiding_suspension() {
    let (program, diagnostics) =
        parse_program("concurrent { let task = spawn work(); await task as outcome }");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let Stmt::Expr(Expr::Concurrent { body, .. }) = &program.stmts[0] else {
        panic!("expected concurrent expression");
    };
    let Stmt::Expr(Expr::Await { mode, .. }) = &body.stmts[1] else {
        panic!("expected await expression");
    };
    assert_eq!(*mode, AwaitMode::Outcome);
}

#[test]
fn recoverable_await_rejects_unknown_mode() {
    let (_, diagnostics) =
        parse_program("concurrent { let task = spawn work(); await task as maybe }");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("expected `outcome` or `race`")));
}

#[test]
fn parses_explicit_bounded_task_race() {
    let (program, diagnostics) = parse_program(
        "concurrent { let a = spawn left(); let b = spawn right(); await [a, b] as race }",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let Stmt::Expr(Expr::Concurrent { body, .. }) = &program.stmts[0] else {
        panic!("expected concurrent expression");
    };
    let Stmt::Expr(Expr::Await { task, mode, .. }) = &body.stmts[2] else {
        panic!("expected await race expression");
    };
    assert_eq!(*mode, AwaitMode::Race);
    assert!(matches!(task.as_ref(), Expr::Array { elements, .. } if elements.len() == 2));
}

#[test]
fn await_keeps_prefix_precedence() {
    let program = parse_ok("concurrent { await task + 1 }");
    let Stmt::Expr(Expr::Concurrent { body, .. }) = &program.stmts[0] else {
        panic!("expected concurrent expression");
    };
    let Stmt::Expr(Expr::BinOp { lhs, .. }) = &body.stmts[0] else {
        panic!("expected binary expression, got {:?}", body.stmts[0]);
    };
    assert!(matches!(lhs.as_ref(), Expr::Await { .. }));
}

#[test]
fn spawn_accepts_postfix_call_expression() {
    let program = parse_ok("concurrent { spawn client.fetch(1) }");
    let Stmt::Expr(Expr::Concurrent { body, .. }) = &program.stmts[0] else {
        panic!("expected concurrent expression");
    };
    let Stmt::Expr(Expr::Spawn { call, .. }) = &body.stmts[0] else {
        panic!("expected spawn expression, got {:?}", body.stmts[0]);
    };
    assert!(matches!(call.as_ref(), Expr::Call { .. }));
}

#[test]
fn nested_control_flow_remains_inside_concurrent_scope() {
    let source = r#"
        concurrent {
            // Scheduling stays explicit even inside nested control flow.
            forever {
                if ready { spawn serve(next()) } else { await signal }
                break
            }
        }
    "#;
    assert!(!has_errors(source));
}

#[test]
fn spawn_outside_concurrent_is_rejected() {
    let (_, errors) = parse_program("spawn work()");
    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("`spawn` is only allowed inside a `concurrent` block"));
}

#[test]
fn spawn_requires_a_call_expression() {
    let (_, errors) = parse_program("concurrent { spawn worker }");
    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("`spawn` operand must be a call expression"));
}

#[test]
fn concurrent_requires_a_block() {
    let (_, errors) = parse_program("concurrent work()");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("expected `{`"));
}

#[test]
fn await_is_explicit_but_not_scope_restricted() {
    let program = parse_ok("fn wait(task) { return await task }");
    let Stmt::FnDecl { body, .. } = &program.stmts[0] else {
        panic!("expected function declaration");
    };
    assert!(matches!(
        &body.stmts[0],
        Stmt::Return {
            value: Some(Expr::Await { .. }),
            ..
        }
    ));
}

#[test]
fn empty_and_nested_concurrent_scopes_parse() {
    let program = parse_ok("concurrent { concurrent {} }");
    let Stmt::Expr(Expr::Concurrent { body, .. }) = &program.stmts[0] else {
        panic!("expected outer concurrent scope");
    };
    assert!(matches!(
        &body.stmts[0],
        Stmt::Expr(Expr::Concurrent { body, .. }) if body.stmts.is_empty()
    ));
}

#[test]
fn reserved_concurrency_identifier_has_migration_hint() {
    let (_, errors) = parse_program("let await = 1");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("reserved keyword `await`"));
    assert!(errors[0].message.contains("rename"));
}
