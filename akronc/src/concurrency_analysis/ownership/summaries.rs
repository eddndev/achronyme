use std::collections::HashMap;

use achronyme_parser::ast::{Block, ElseBranch, Expr, ForIterable, Stmt};
use akron::specs::{ResourceEffect, ResourceKind};

use super::{callee_name, FunctionDef, FunctionSummary, NativeResources, ParamMode};

pub(super) fn infer_function_summaries(
    functions: &HashMap<String, FunctionDef>,
    natives: &NativeResources,
) -> HashMap<String, FunctionSummary> {
    let mut summaries = functions
        .iter()
        .map(|(name, function)| {
            (
                name.clone(),
                FunctionSummary {
                    params: function.params.clone(),
                    modes: vec![None; function.params.len()],
                    return_kind: None,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    for _ in 0..32 {
        let snapshot = summaries.clone();
        let mut changed = false;
        for (name, function) in functions {
            let mut next = snapshot.get(name).cloned().unwrap_or_default();
            infer_statement_constraints(
                &function.body.stmts,
                &function.params,
                natives,
                &snapshot,
                &mut next.modes,
            );
            let return_kind =
                block_result_kind(&function.body, &function.params, &next, natives, &snapshot);
            if return_kind.is_some() {
                next.return_kind = return_kind;
            }
            if summaries.get(name) != Some(&next) {
                summaries.insert(name.clone(), next);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    summaries
}

fn infer_statement_constraints(
    statements: &[Stmt],
    params: &[String],
    natives: &NativeResources,
    functions: &HashMap<String, FunctionSummary>,
    modes: &mut [Option<ParamMode>],
) {
    for statement in statements {
        match statement {
            Stmt::LetDecl { value, .. }
            | Stmt::MutDecl { value, .. }
            | Stmt::Print { value, .. }
            | Stmt::Expr(value) => infer_expr_constraints(value, params, natives, functions, modes),
            Stmt::Assignment { target, value, .. } => {
                infer_expr_constraints(target, params, natives, functions, modes);
                infer_expr_constraints(value, params, natives, functions, modes);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    infer_expr_constraints(value, params, natives, functions, modes);
                }
            }
            Stmt::Export { inner, .. } => infer_statement_constraints(
                std::slice::from_ref(inner.as_ref()),
                params,
                natives,
                functions,
                modes,
            ),
            Stmt::CircuitDecl { body, .. } => {
                infer_statement_constraints(&body.stmts, params, natives, functions, modes)
            }
            Stmt::FnDecl { .. }
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

fn infer_expr_constraints(
    expression: &Expr,
    params: &[String],
    natives: &NativeResources,
    functions: &HashMap<String, FunctionSummary>,
    modes: &mut [Option<ParamMode>],
) {
    match expression {
        Expr::Call { callee, args, .. } => {
            if let Some(name) = callee_name(callee) {
                if let Some(effect) = natives.get(name) {
                    if let Some((kind, consumes)) = resource_argument_mode(*effect) {
                        constrain_direct_param(
                            args.first().map(|arg| &arg.value),
                            params,
                            modes,
                            kind,
                            consumes,
                        );
                    }
                } else if let Some(summary) = functions.get(name) {
                    for (argument, mode) in args.iter().zip(&summary.modes) {
                        if let Some(mode) = mode {
                            constrain_direct_param(
                                Some(&argument.value),
                                params,
                                modes,
                                mode.kind,
                                mode.consumes,
                            );
                        }
                    }
                }
            }
            infer_expr_constraints(callee, params, natives, functions, modes);
            for argument in args {
                infer_expr_constraints(&argument.value, params, natives, functions, modes);
            }
        }
        Expr::Spawn { call, .. } => {
            infer_expr_constraints(call, params, natives, functions, modes);
            if let Expr::Call { callee, args, .. } = call.as_ref() {
                if let Some(name) = callee_name(callee) {
                    if let Some(effect) = natives.get(name) {
                        if let Some((kind, _)) = resource_argument_mode(*effect) {
                            constrain_direct_param(
                                args.first().map(|arg| &arg.value),
                                params,
                                modes,
                                kind,
                                true,
                            );
                        }
                    } else if let Some(summary) = functions.get(name) {
                        for (argument, mode) in args.iter().zip(&summary.modes) {
                            if let Some(mode) = mode {
                                constrain_direct_param(
                                    Some(&argument.value),
                                    params,
                                    modes,
                                    mode.kind,
                                    true,
                                );
                            }
                        }
                    }
                }
            }
        }
        Expr::Await { task, .. } | Expr::UnaryOp { operand: task, .. } => {
            infer_expr_constraints(task, params, natives, functions, modes)
        }
        Expr::BinOp { lhs, rhs, .. } => {
            infer_expr_constraints(lhs, params, natives, functions, modes);
            infer_expr_constraints(rhs, params, natives, functions, modes);
        }
        Expr::Index { object, index, .. } => {
            infer_expr_constraints(object, params, natives, functions, modes);
            infer_expr_constraints(index, params, natives, functions, modes);
        }
        Expr::DotAccess { object, .. } => {
            infer_expr_constraints(object, params, natives, functions, modes)
        }
        Expr::If {
            condition,
            then_block,
            else_branch,
            ..
        } => {
            infer_expr_constraints(condition, params, natives, functions, modes);
            infer_statement_constraints(&then_block.stmts, params, natives, functions, modes);
            if let Some(branch) = else_branch {
                match branch {
                    ElseBranch::Block(block) => {
                        infer_statement_constraints(&block.stmts, params, natives, functions, modes)
                    }
                    ElseBranch::If(expression) => {
                        infer_expr_constraints(expression, params, natives, functions, modes)
                    }
                }
            }
        }
        Expr::For { iterable, body, .. } => {
            if let ForIterable::ExprRange { end, .. } | ForIterable::Expr(end) = iterable {
                infer_expr_constraints(end, params, natives, functions, modes);
            }
            infer_statement_constraints(&body.stmts, params, natives, functions, modes);
        }
        Expr::While {
            condition, body, ..
        } => {
            infer_expr_constraints(condition, params, natives, functions, modes);
            infer_statement_constraints(&body.stmts, params, natives, functions, modes);
        }
        Expr::Forever { body, .. }
        | Expr::Concurrent { body, .. }
        | Expr::Block { block: body, .. }
        | Expr::FnExpr { body, .. }
        | Expr::Prove { body, .. } => {
            infer_statement_constraints(&body.stmts, params, natives, functions, modes)
        }
        Expr::Array { elements, .. } => {
            for element in elements {
                infer_expr_constraints(element, params, natives, functions, modes);
            }
        }
        Expr::Map { pairs, .. } => {
            for (_, value) in pairs {
                infer_expr_constraints(value, params, natives, functions, modes);
            }
        }
        Expr::Number { .. }
        | Expr::FieldLit { .. }
        | Expr::BigIntLit { .. }
        | Expr::Bool { .. }
        | Expr::StringLit { .. }
        | Expr::Nil { .. }
        | Expr::Ident { .. }
        | Expr::StaticAccess { .. }
        | Expr::Error { .. } => {}
    }
}

fn constrain_direct_param(
    expression: Option<&Expr>,
    params: &[String],
    modes: &mut [Option<ParamMode>],
    kind: ResourceKind,
    consumes: bool,
) {
    let Some(Expr::Ident { name, .. }) = expression else {
        return;
    };
    let Some(index) = params.iter().position(|param| param == name) else {
        return;
    };
    let next = ParamMode { kind, consumes };
    modes[index] = match modes[index] {
        None => Some(next),
        Some(previous) if previous.kind == kind => Some(ParamMode {
            kind,
            consumes: previous.consumes || consumes,
        }),
        Some(previous) => Some(previous),
    };
}

pub(super) fn resource_argument_mode(effect: ResourceEffect) -> Option<(ResourceKind, bool)> {
    match effect {
        ResourceEffect::Consumes(kind) => Some((kind, true)),
        ResourceEffect::Borrows(kind) => Some((kind, false)),
        ResourceEffect::CreatesAndBorrows { borrowed, .. } => Some((borrowed, false)),
        ResourceEffect::None | ResourceEffect::Creates(_) => None,
    }
}

fn block_result_kind(
    block: &Block,
    params: &[String],
    own: &FunctionSummary,
    natives: &NativeResources,
    functions: &HashMap<String, FunctionSummary>,
) -> Option<ResourceKind> {
    for statement in &block.stmts {
        if let Stmt::Return {
            value: Some(value), ..
        } = statement
        {
            if let Some(kind) = expression_result_kind(value, params, own, natives, functions) {
                return Some(kind);
            }
        }
    }
    match block.stmts.last() {
        Some(Stmt::Expr(expression)) => {
            expression_result_kind(expression, params, own, natives, functions)
        }
        _ => None,
    }
}

fn expression_result_kind(
    expression: &Expr,
    params: &[String],
    own: &FunctionSummary,
    natives: &NativeResources,
    functions: &HashMap<String, FunctionSummary>,
) -> Option<ResourceKind> {
    match expression {
        Expr::Ident { name, .. } => params
            .iter()
            .position(|param| param == name)
            .and_then(|index| own.modes.get(index).and_then(|mode| *mode))
            .map(|mode| mode.kind),
        Expr::Await { task, .. } => expression_result_kind(task, params, own, natives, functions),
        Expr::Call { callee, .. } => callee_name(callee).and_then(|name| {
            natives
                .get(name)
                .and_then(|effect| created_kind(*effect))
                .or_else(|| functions.get(name).and_then(|summary| summary.return_kind))
        }),
        Expr::Block { block, .. } => block_result_kind(block, params, own, natives, functions),
        Expr::If {
            then_block,
            else_branch: Some(else_branch),
            ..
        } => {
            let left = block_result_kind(then_block, params, own, natives, functions);
            let right = match else_branch {
                ElseBranch::Block(block) => {
                    block_result_kind(block, params, own, natives, functions)
                }
                ElseBranch::If(expression) => {
                    expression_result_kind(expression, params, own, natives, functions)
                }
            };
            (left == right).then_some(left).flatten()
        }
        _ => None,
    }
}

pub(super) fn created_kind(effect: ResourceEffect) -> Option<ResourceKind> {
    match effect {
        ResourceEffect::Creates(kind) => Some(kind),
        ResourceEffect::CreatesAndBorrows { created, .. } => Some(created),
        ResourceEffect::None | ResourceEffect::Consumes(_) | ResourceEffect::Borrows(_) => None,
    }
}
