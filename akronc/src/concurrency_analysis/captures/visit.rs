use std::collections::{HashMap, HashSet};

use achronyme_parser::ast::{Expr, Span, Stmt};
use achronyme_parser::Diagnostic;

use super::collect::summarize_function;
use super::walk::child_nodes;
use super::CaptureSummary;

pub(super) fn visit_statements_for_concurrent(
    statements: &[Stmt],
    mutable_bindings: &HashMap<String, Span>,
    summaries: &HashMap<String, CaptureSummary>,
    output: &mut Vec<Diagnostic>,
) {
    for statement in statements {
        match statement {
            Stmt::LetDecl { value, .. }
            | Stmt::MutDecl { value, .. }
            | Stmt::Print { value, .. }
            | Stmt::Expr(value) => {
                visit_expr_for_concurrent(value, mutable_bindings, summaries, output)
            }
            Stmt::Assignment { target, value, .. } => {
                visit_expr_for_concurrent(target, mutable_bindings, summaries, output);
                visit_expr_for_concurrent(value, mutable_bindings, summaries, output);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    visit_expr_for_concurrent(value, mutable_bindings, summaries, output);
                }
            }
            Stmt::FnDecl { body, .. } | Stmt::CircuitDecl { body, .. } => {
                visit_statements_for_concurrent(&body.stmts, mutable_bindings, summaries, output)
            }
            Stmt::Export { inner, .. } => visit_statements_for_concurrent(
                std::slice::from_ref(inner.as_ref()),
                mutable_bindings,
                summaries,
                output,
            ),
            Stmt::PublicDecl { .. }
            | Stmt::WitnessDecl { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Import { .. }
            | Stmt::SelectiveImport { .. }
            | Stmt::ExportList { .. }
            | Stmt::ImportCircuit { .. }
            | Stmt::Error { .. } => {}
        }
    }
}

fn visit_expr_for_concurrent(
    expression: &Expr,
    mutable_bindings: &HashMap<String, Span>,
    summaries: &HashMap<String, CaptureSummary>,
    output: &mut Vec<Diagnostic>,
) {
    if let Expr::Concurrent { body, span, .. } = expression {
        let mut child_captures = Vec::new();
        collect_scope_spawns(&body.stmts, summaries, &mut child_captures);
        let mut counts = HashMap::<String, usize>::new();
        for captures in child_captures {
            for name in captures {
                if mutable_bindings.contains_key(&name) {
                    *counts.entry(name).or_default() += 1;
                }
            }
        }
        for (name, count) in counts {
            if count < 2 {
                continue;
            }
            let declaration = mutable_bindings
                .get(&name)
                .expect("capture name filtered by mutable bindings");
            output.push(
                Diagnostic::warning(
                    format!(
                        "outer mutable binding `{name}` is captured by {count} child tasks"
                    ),
                    span.into(),
                )
                .with_code("W011")
                .with_label(declaration.into(), "mutable binding declared here")
                .with_note(
                    "task interleaving occurs only at `await`, but the ordering remains observable and may be rejected by a future parallel runtime",
                ),
            );
        }
    }

    let (children, blocks) = child_nodes(expression);
    for child in children {
        visit_expr_for_concurrent(child, mutable_bindings, summaries, output);
    }
    for block in blocks {
        visit_statements_for_concurrent(&block.stmts, mutable_bindings, summaries, output);
    }
}

fn collect_scope_spawns(
    statements: &[Stmt],
    summaries: &HashMap<String, CaptureSummary>,
    output: &mut Vec<HashSet<String>>,
) {
    for statement in statements {
        match statement {
            Stmt::LetDecl { value, .. }
            | Stmt::MutDecl { value, .. }
            | Stmt::Print { value, .. }
            | Stmt::Expr(value) => collect_spawns_expr(value, summaries, output),
            Stmt::Assignment { target, value, .. } => {
                collect_spawns_expr(target, summaries, output);
                collect_spawns_expr(value, summaries, output);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    collect_spawns_expr(value, summaries, output);
                }
            }
            Stmt::Export { inner, .. } => {
                collect_scope_spawns(std::slice::from_ref(inner.as_ref()), summaries, output)
            }
            Stmt::FnDecl { .. }
            | Stmt::CircuitDecl { .. }
            | Stmt::PublicDecl { .. }
            | Stmt::WitnessDecl { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Import { .. }
            | Stmt::SelectiveImport { .. }
            | Stmt::ExportList { .. }
            | Stmt::ImportCircuit { .. }
            | Stmt::Error { .. } => {}
        }
    }
}

fn collect_spawns_expr(
    expression: &Expr,
    summaries: &HashMap<String, CaptureSummary>,
    output: &mut Vec<HashSet<String>>,
) {
    match expression {
        Expr::Spawn { call, .. } => {
            output.push(spawn_captures(call, summaries));
            collect_spawns_expr(call, summaries, output);
        }
        Expr::Concurrent { .. } => {}
        _ => {
            let (children, blocks) = child_nodes(expression);
            for child in children {
                collect_spawns_expr(child, summaries, output);
            }
            for block in blocks {
                collect_scope_spawns(&block.stmts, summaries, output);
            }
        }
    }
}

fn spawn_captures(call: &Expr, summaries: &HashMap<String, CaptureSummary>) -> HashSet<String> {
    let Expr::Call { callee, .. } = call else {
        return HashSet::new();
    };
    match callee.as_ref() {
        Expr::Ident { name, .. } => summaries
            .get(name)
            .map(|summary| summary.transitive.clone())
            .unwrap_or_default(),
        Expr::FnExpr { params, body, .. } => summarize_function(params, body).transitive,
        _ => HashSet::new(),
    }
}
