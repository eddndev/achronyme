use std::fmt::Write;

use crate::ast::{
    AwaitMode, BigIntRadix, BinOp, ElseBranch, Expr, FieldRadix, ForIterable, MapKey, UnaryOp,
};

use super::{FormatError, Formatter};

impl Formatter {
    pub(super) fn write_expr(
        &mut self,
        expression: &Expr,
        minimum_binding_power: u8,
    ) -> Result<(), FormatError> {
        let binding_power = expression_binding_power(expression);
        let parenthesized = binding_power < minimum_binding_power;
        if parenthesized {
            self.output.push('(');
        }

        match expression {
            Expr::Number { value, .. } => self.output.push_str(value),
            Expr::FieldLit { value, radix, .. } => {
                self.output.push_str(match radix {
                    FieldRadix::Decimal => "0p",
                    FieldRadix::Hex => "0px",
                    FieldRadix::Binary => "0pb",
                });
                self.output.push_str(value);
            }
            Expr::BigIntLit {
                value,
                width,
                radix,
                ..
            } => {
                let radix = match radix {
                    BigIntRadix::Hex => 'x',
                    BigIntRadix::Decimal => 'd',
                    BigIntRadix::Binary => 'b',
                };
                write!(self.output, "0i{width}{radix}{value}").unwrap();
            }
            Expr::Bool { value, .. } => self.output.push_str(if *value { "true" } else { "false" }),
            Expr::StringLit { value, .. } => write!(self.output, "{value:?}").unwrap(),
            Expr::Nil { .. } => self.output.push_str("nil"),
            Expr::Ident { name, .. } => self.output.push_str(name),
            Expr::BinOp { op, lhs, rhs, .. } => {
                let (left_power, right_power) = binary_binding_power(op);
                let left_power = if is_comparison(op) {
                    right_power
                } else {
                    left_power
                };
                self.write_expr(lhs, left_power)?;
                write!(self.output, " {} ", binary_text(op)).unwrap();
                self.write_expr(rhs, right_power)?;
            }
            Expr::UnaryOp { op, operand, .. } => {
                self.output.push_str(match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                });
                self.write_expr(operand, 11)?;
            }
            Expr::Call { callee, args, .. } => {
                self.write_expr(callee, 13)?;
                self.output.push('(');
                for (index, argument) in args.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    if let Some(name) = &argument.name {
                        write!(self.output, "{name}: ").unwrap();
                    }
                    self.write_expr(&argument.value, 0)?;
                }
                self.output.push(')');
            }
            Expr::Index { object, index, .. } => {
                self.write_expr(object, 13)?;
                self.output.push('[');
                self.write_expr(index, 0)?;
                self.output.push(']');
            }
            Expr::DotAccess { object, field, .. } => {
                self.write_expr(object, 13)?;
                write!(self.output, ".{field}").unwrap();
            }
            Expr::If {
                condition,
                then_block,
                else_branch,
                ..
            } => {
                self.output.push_str("if ");
                self.write_expr(condition, 0)?;
                self.output.push(' ');
                self.write_block(then_block)?;
                if let Some(branch) = else_branch {
                    self.output.push_str(" else ");
                    match branch {
                        ElseBranch::Block(block) => self.write_block(block)?,
                        ElseBranch::If(expression) => self.write_expr(expression, 0)?,
                    }
                }
            }
            Expr::For {
                var,
                iterable,
                body,
                ..
            } => {
                write!(self.output, "for {var} in ").unwrap();
                match iterable {
                    ForIterable::Range { start, end } => {
                        write!(self.output, "{start}..{end}").unwrap();
                    }
                    ForIterable::ExprRange { start, end } => {
                        write!(self.output, "{start}..").unwrap();
                        self.write_expr(end, 0)?;
                    }
                    ForIterable::Expr(expression) => self.write_expr(expression, 0)?,
                }
                self.output.push(' ');
                self.write_block(body)?;
            }
            Expr::While {
                condition, body, ..
            } => {
                self.output.push_str("while ");
                self.write_expr(condition, 0)?;
                self.output.push(' ');
                self.write_block(body)?;
            }
            Expr::Forever { body, .. } => {
                self.output.push_str("forever ");
                self.write_block(body)?;
            }
            Expr::Concurrent { body, .. } => {
                self.output.push_str("concurrent ");
                self.write_block(body)?;
            }
            Expr::Spawn { call, .. } => {
                self.output.push_str("spawn ");
                self.write_expr(call, 11)?;
            }
            Expr::Await { task, mode, .. } => {
                self.output.push_str("await ");
                self.write_expr(task, 11)?;
                match mode {
                    AwaitMode::Propagate => {}
                    AwaitMode::Outcome => self.output.push_str(" as outcome"),
                    AwaitMode::Race => self.output.push_str(" as race"),
                }
            }
            Expr::Block { block, .. } => self.write_block(block)?,
            Expr::FnExpr {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                self.output.push_str("fn");
                if let Some(name) = name {
                    write!(self.output, " {name}").unwrap();
                }
                self.output.push('(');
                self.write_params(params);
                self.output.push(')');
                if let Some(annotation) = return_type {
                    write!(self.output, " -> {annotation}").unwrap();
                }
                self.output.push(' ');
                self.write_block(body)?;
            }
            Expr::Prove {
                name, body, params, ..
            } => {
                self.output.push_str("prove");
                if let Some(name) = name {
                    write!(self.output, " {name}").unwrap();
                }
                if !params.is_empty() {
                    self.output.push('(');
                    self.write_params(params);
                    self.output.push(')');
                }
                self.output.push(' ');
                self.write_block(body)?;
            }
            Expr::Array { elements, .. } => {
                self.output.push('[');
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    self.write_expr(element, 0)?;
                }
                self.output.push(']');
            }
            Expr::Map { pairs, .. } => {
                self.output.push('{');
                for (index, (key, value)) in pairs.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    match key {
                        MapKey::Ident(key) => self.output.push_str(key),
                        MapKey::StringLit(key) => write!(self.output, "{key:?}").unwrap(),
                    }
                    self.output.push_str(": ");
                    self.write_expr(value, 0)?;
                }
                self.output.push('}');
            }
            Expr::StaticAccess {
                type_name, member, ..
            } => write!(self.output, "{type_name}::{member}").unwrap(),
            Expr::Error { .. } => return Err(FormatError::recovered_node("expression")),
        }

        if parenthesized {
            self.output.push(')');
        }
        Ok(())
    }
}

fn expression_binding_power(expression: &Expr) -> u8 {
    match expression {
        Expr::BinOp { op, .. } => binary_binding_power(op).0,
        Expr::UnaryOp { .. } | Expr::Spawn { .. } | Expr::Await { .. } => 11,
        Expr::Call { .. } | Expr::Index { .. } | Expr::DotAccess { .. } => 13,
        _ => 14,
    }
}

fn binary_binding_power(operator: &BinOp) -> (u8, u8) {
    match operator {
        BinOp::Or => (1, 2),
        BinOp::And => (3, 4),
        BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => (5, 6),
        BinOp::Add | BinOp::Sub => (7, 8),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (9, 10),
        BinOp::Pow => (12, 11),
    }
}

fn is_comparison(operator: &BinOp) -> bool {
    matches!(
        operator,
        BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
    )
}

fn binary_text(operator: &BinOp) -> &'static str {
    match operator {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Pow => "^",
        BinOp::Eq => "==",
        BinOp::Neq => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
    }
}
