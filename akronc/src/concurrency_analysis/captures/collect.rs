use std::collections::{HashMap, HashSet};

use achronyme_parser::ast::{Block, Expr, ForIterable, Span, Stmt, TypedParam};

use super::walk::{child_nodes, walk_expr_blocks};
use super::CaptureSummary;

pub(super) fn collect_mutable_bindings(statements: &[Stmt], output: &mut HashMap<String, Span>) {
    for statement in statements {
        match statement {
            Stmt::MutDecl {
                name, value, span, ..
            } => {
                output.entry(name.clone()).or_insert_with(|| span.clone());
                collect_mutable_expr(value, output);
            }
            Stmt::LetDecl { value, .. } | Stmt::Print { value, .. } | Stmt::Expr(value) => {
                collect_mutable_expr(value, output)
            }
            Stmt::Assignment { target, value, .. } => {
                collect_mutable_expr(target, output);
                collect_mutable_expr(value, output);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    collect_mutable_expr(value, output);
                }
            }
            Stmt::FnDecl { body, .. } | Stmt::CircuitDecl { body, .. } => {
                collect_mutable_bindings(&body.stmts, output)
            }
            Stmt::Export { inner, .. } => {
                collect_mutable_bindings(std::slice::from_ref(inner.as_ref()), output)
            }
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

fn collect_mutable_expr(expression: &Expr, output: &mut HashMap<String, Span>) {
    walk_expr_blocks(expression, &mut |block| {
        collect_mutable_bindings(&block.stmts, output)
    });
}

pub(super) fn collect_function_summaries(
    statements: &[Stmt],
    output: &mut HashMap<String, CaptureSummary>,
) {
    for statement in statements {
        match statement {
            Stmt::FnDecl {
                name, params, body, ..
            } => {
                output.insert(name.clone(), summarize_function(params, body));
                collect_function_summaries(&body.stmts, output);
            }
            Stmt::Export { inner, .. } => {
                collect_function_summaries(std::slice::from_ref(inner.as_ref()), output)
            }
            Stmt::LetDecl { value, .. }
            | Stmt::MutDecl { value, .. }
            | Stmt::Print { value, .. }
            | Stmt::Expr(value) => collect_functions_expr(value, output),
            Stmt::Assignment { target, value, .. } => {
                collect_functions_expr(target, output);
                collect_functions_expr(value, output);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    collect_functions_expr(value, output);
                }
            }
            Stmt::CircuitDecl { body, .. } => collect_function_summaries(&body.stmts, output),
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

fn collect_functions_expr(expression: &Expr, output: &mut HashMap<String, CaptureSummary>) {
    if let Expr::FnExpr {
        name: Some(name),
        params,
        body,
        ..
    } = expression
    {
        output.insert(name.clone(), summarize_function(params, body));
    }
    walk_expr_blocks(expression, &mut |block| {
        collect_function_summaries(&block.stmts, output)
    });
}

pub(super) fn summarize_function(params: &[TypedParam], body: &Block) -> CaptureSummary {
    let mut locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    collect_local_names(&body.stmts, &mut locals);
    let mut direct = HashSet::new();
    let mut calls = HashSet::new();
    collect_uses(&body.stmts, &locals, &mut direct, &mut calls);
    CaptureSummary {
        transitive: direct.clone(),
        direct,
        calls,
    }
}

fn collect_local_names(statements: &[Stmt], locals: &mut HashSet<String>) {
    for statement in statements {
        match statement {
            Stmt::LetDecl { name, value, .. } | Stmt::MutDecl { name, value, .. } => {
                locals.insert(name.clone());
                collect_local_names_expr(value, locals);
            }
            Stmt::FnDecl { name, .. } => {
                locals.insert(name.clone());
            }
            Stmt::Assignment { target, value, .. } => {
                collect_local_names_expr(target, locals);
                collect_local_names_expr(value, locals);
            }
            Stmt::Print { value, .. } | Stmt::Expr(value) => {
                collect_local_names_expr(value, locals)
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    collect_local_names_expr(value, locals);
                }
            }
            Stmt::Export { inner, .. } => {
                collect_local_names(std::slice::from_ref(inner.as_ref()), locals)
            }
            Stmt::CircuitDecl { name, .. } => {
                locals.insert(name.clone());
            }
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

fn collect_local_names_expr(expression: &Expr, locals: &mut HashSet<String>) {
    match expression {
        Expr::For {
            var,
            iterable,
            body,
            ..
        } => {
            locals.insert(var.clone());
            if let ForIterable::ExprRange { end, .. } | ForIterable::Expr(end) = iterable {
                collect_local_names_expr(end, locals);
            }
            collect_local_names(&body.stmts, locals);
        }
        Expr::FnExpr { name, .. } => {
            if let Some(name) = name {
                locals.insert(name.clone());
            }
        }
        _ => {
            let (children, blocks) = child_nodes(expression);
            for child in children {
                collect_local_names_expr(child, locals);
            }
            for block in blocks {
                collect_local_names(&block.stmts, locals);
            }
        }
    }
}

fn collect_uses(
    statements: &[Stmt],
    locals: &HashSet<String>,
    direct: &mut HashSet<String>,
    calls: &mut HashSet<String>,
) {
    for statement in statements {
        match statement {
            Stmt::FnDecl { .. } | Stmt::CircuitDecl { .. } => {}
            Stmt::LetDecl { value, .. }
            | Stmt::MutDecl { value, .. }
            | Stmt::Print { value, .. }
            | Stmt::Expr(value) => collect_expr_uses(value, locals, direct, calls, false),
            Stmt::Assignment { target, value, .. } => {
                collect_expr_uses(target, locals, direct, calls, false);
                collect_expr_uses(value, locals, direct, calls, false);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    collect_expr_uses(value, locals, direct, calls, false);
                }
            }
            Stmt::Export { inner, .. } => {
                collect_uses(std::slice::from_ref(inner.as_ref()), locals, direct, calls)
            }
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

fn collect_expr_uses(
    expression: &Expr,
    locals: &HashSet<String>,
    direct: &mut HashSet<String>,
    calls: &mut HashSet<String>,
    callee_position: bool,
) {
    match expression {
        Expr::Ident { name, .. } => {
            if callee_position {
                calls.insert(name.clone());
            } else if !locals.contains(name) {
                direct.insert(name.clone());
            }
        }
        Expr::Call { callee, args, .. } => {
            collect_expr_uses(callee, locals, direct, calls, true);
            for argument in args {
                collect_expr_uses(&argument.value, locals, direct, calls, false);
            }
        }
        Expr::FnExpr { .. } => {}
        _ => {
            let (children, blocks) = child_nodes(expression);
            for child in children {
                collect_expr_uses(child, locals, direct, calls, false);
            }
            for block in blocks {
                collect_uses(&block.stmts, locals, direct, calls);
            }
        }
    }
}

pub(super) fn close_transitive_captures(summaries: &mut HashMap<String, CaptureSummary>) {
    for _ in 0..32 {
        let snapshot = summaries.clone();
        let mut changed = false;
        for summary in summaries.values_mut() {
            let mut next = summary.direct.clone();
            for callee in &summary.calls {
                if let Some(callee) = snapshot.get(callee) {
                    next.extend(callee.transitive.iter().cloned());
                }
            }
            if next != summary.transitive {
                summary.transitive = next;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}
