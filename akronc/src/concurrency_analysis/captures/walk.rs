use achronyme_parser::ast::{Block, ElseBranch, Expr, ForIterable};

pub(super) fn walk_expr_blocks(expression: &Expr, block: &mut impl FnMut(&Block)) {
    let (children, blocks) = child_nodes(expression);
    for child in children {
        walk_expr_blocks(child, block);
    }
    for child_block in blocks {
        block(child_block);
    }
}

pub(super) fn child_nodes(expression: &Expr) -> (Vec<&Expr>, Vec<&Block>) {
    let mut expressions = Vec::new();
    let mut blocks = Vec::new();
    match expression {
        Expr::BinOp { lhs, rhs, .. } => {
            expressions.push(lhs.as_ref());
            expressions.push(rhs.as_ref());
        }
        Expr::UnaryOp { operand, .. }
        | Expr::Spawn { call: operand, .. }
        | Expr::Await { task: operand, .. }
        | Expr::DotAccess {
            object: operand, ..
        } => expressions.push(operand.as_ref()),
        Expr::Call { callee, args, .. } => {
            expressions.push(callee.as_ref());
            for argument in args {
                expressions.push(&argument.value);
            }
        }
        Expr::Index { object, index, .. } => {
            expressions.push(object.as_ref());
            expressions.push(index.as_ref());
        }
        Expr::If {
            condition,
            then_block,
            else_branch,
            ..
        } => {
            expressions.push(condition.as_ref());
            blocks.push(then_block);
            if let Some(branch) = else_branch {
                match branch {
                    ElseBranch::Block(block) => blocks.push(block),
                    ElseBranch::If(expression) => expressions.push(expression.as_ref()),
                }
            }
        }
        Expr::For { iterable, body, .. } => {
            if let ForIterable::ExprRange { end, .. } | ForIterable::Expr(end) = iterable {
                expressions.push(end.as_ref());
            }
            blocks.push(body);
        }
        Expr::While {
            condition, body, ..
        } => {
            expressions.push(condition.as_ref());
            blocks.push(body);
        }
        Expr::Forever { body, .. }
        | Expr::Concurrent { body, .. }
        | Expr::FnExpr { body, .. }
        | Expr::Prove { body, .. } => blocks.push(body),
        Expr::Block { block, .. } => blocks.push(block),
        Expr::Array { elements, .. } => {
            for element in elements {
                expressions.push(element);
            }
        }
        Expr::Map { pairs, .. } => {
            for (_, value) in pairs {
                expressions.push(value);
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
    (expressions, blocks)
}
