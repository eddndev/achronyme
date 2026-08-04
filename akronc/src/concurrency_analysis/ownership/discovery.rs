use std::collections::HashMap;

use achronyme_parser::ast::{ElseBranch, Expr, ForIterable, Stmt, TypedParam};

use super::FunctionDef;

pub(super) fn collect_function_defs(
    statements: &[Stmt],
    output: &mut HashMap<String, FunctionDef>,
) {
    for statement in statements {
        match statement {
            Stmt::FnDecl {
                name, params, body, ..
            } => {
                output.insert(
                    name.clone(),
                    FunctionDef {
                        params: param_names(params),
                        body: body.clone(),
                    },
                );
                collect_function_defs(&body.stmts, output);
            }
            Stmt::Export { inner, .. } => {
                collect_function_defs(std::slice::from_ref(inner.as_ref()), output)
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
            Stmt::CircuitDecl { body, .. } => collect_function_defs(&body.stmts, output),
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

fn collect_functions_expr(expression: &Expr, output: &mut HashMap<String, FunctionDef>) {
    match expression {
        Expr::FnExpr {
            name: Some(name),
            params,
            body,
            ..
        } => {
            output.insert(
                name.clone(),
                FunctionDef {
                    params: param_names(params),
                    body: body.clone(),
                },
            );
            collect_function_defs(&body.stmts, output);
        }
        Expr::If {
            condition,
            then_block,
            else_branch,
            ..
        } => {
            collect_functions_expr(condition, output);
            collect_function_defs(&then_block.stmts, output);
            if let Some(branch) = else_branch {
                match branch {
                    ElseBranch::Block(block) => collect_function_defs(&block.stmts, output),
                    ElseBranch::If(expression) => collect_functions_expr(expression, output),
                }
            }
        }
        Expr::For { iterable, body, .. } => {
            if let ForIterable::ExprRange { end, .. } | ForIterable::Expr(end) = iterable {
                collect_functions_expr(end, output);
            }
            collect_function_defs(&body.stmts, output);
        }
        Expr::While {
            condition, body, ..
        } => {
            collect_functions_expr(condition, output);
            collect_function_defs(&body.stmts, output);
        }
        Expr::Forever { body, .. } | Expr::Concurrent { body, .. } | Expr::Prove { body, .. } => {
            collect_function_defs(&body.stmts, output)
        }
        Expr::Block { block, .. } => collect_function_defs(&block.stmts, output),
        Expr::BinOp { lhs, rhs, .. } => {
            collect_functions_expr(lhs, output);
            collect_functions_expr(rhs, output);
        }
        Expr::UnaryOp { operand, .. }
        | Expr::Spawn { call: operand, .. }
        | Expr::Await { task: operand, .. } => collect_functions_expr(operand, output),
        Expr::Call { callee, args, .. } => {
            collect_functions_expr(callee, output);
            for argument in args {
                collect_functions_expr(&argument.value, output);
            }
        }
        Expr::Index { object, index, .. } => {
            collect_functions_expr(object, output);
            collect_functions_expr(index, output);
        }
        Expr::DotAccess { object, .. } => collect_functions_expr(object, output),
        Expr::Array { elements, .. } => {
            for element in elements {
                collect_functions_expr(element, output);
            }
        }
        Expr::Map { pairs, .. } => {
            for (_, value) in pairs {
                collect_functions_expr(value, output);
            }
        }
        Expr::FnExpr { body, .. } => collect_function_defs(&body.stmts, output),
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

fn param_names(params: &[TypedParam]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}
