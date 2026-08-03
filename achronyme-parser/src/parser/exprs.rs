use crate::ast::*;
use crate::error::ParseError;
use crate::token::TokenKind;

use super::core::Parser;
use super::tables::{infix_bp, is_comparison, tok_display, token_to_binop};

mod compound;
mod postfix;
mod prove;

impl Parser {
    // ========================================================================
    // Expressions — Pratt parser
    // ========================================================================

    pub(super) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        if self.expr_depth >= Self::MAX_EXPR_DEPTH {
            let sp = self.span();
            return Err(ParseError::new(
                format!("expression nesting exceeds {} levels", Self::MAX_EXPR_DEPTH),
                sp.line_start,
                sp.col_start,
            ));
        }
        self.expr_depth += 1;
        let result = self.parse_expr_bp_inner(min_bp);
        self.expr_depth -= 1;
        result
    }

    fn parse_expr_bp_inner(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        // Prefix
        let mut lhs = self.parse_prefix()?;

        // Infix / postfix loop
        loop {
            // Postfix: call, index, dot
            match self.peek_kind() {
                TokenKind::LParen => {
                    if 13 < min_bp {
                        break;
                    }
                    lhs = self.parse_call(lhs)?;
                    continue;
                }
                TokenKind::LBracket => {
                    if 13 < min_bp {
                        break;
                    }
                    lhs = self.parse_index(lhs)?;
                    continue;
                }
                TokenKind::Dot => {
                    if 13 < min_bp {
                        break;
                    }
                    lhs = self.parse_dot(lhs)?;
                    continue;
                }
                _ => {}
            }

            // Infix binary operators
            if let Some((l_bp, r_bp)) = infix_bp(self.peek_kind()) {
                if l_bp < min_bp {
                    break;
                }
                let op_tok = self.advance().clone();
                let was_cmp = is_comparison(&op_tok.kind);
                let op = token_to_binop(&op_tok.kind);
                let rhs = self.parse_expr_bp(r_bp)?;
                let sp = lhs.span().clone();
                let id = self.alloc_expr_id();
                lhs = Expr::BinOp {
                    id,
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    span: self.span_to_prev(&sp),
                };
                // Reject chained comparisons: `a < b < c` is a silent bug
                if was_cmp && is_comparison(self.peek_kind()) {
                    let next = self.peek();
                    return Err(ParseError::new(
                        "comparison operators cannot be chained; use `&&` to combine: `a < b && b < c`",
                        next.span.line_start,
                        next.span.col_start,
                    ));
                }
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            TokenKind::Minus | TokenKind::Not => {
                let sp = self.span();
                let op_tok = self.advance().clone();
                let op = match op_tok.kind {
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Not => UnaryOp::Not,
                    _ => unreachable!(),
                };
                let operand = self.parse_expr_bp(11)?; // prefix BP
                let id = self.alloc_expr_id();
                Ok(Expr::UnaryOp {
                    id,
                    op,
                    operand: Box::new(operand),
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::Await => {
                let sp = self.span();
                self.advance();
                let task = self.parse_expr_bp(11)?;
                let mode = if self.at(&TokenKind::As) {
                    self.advance();
                    let marker = self.advance().clone();
                    if marker.kind != TokenKind::Ident
                        || !matches!(marker.lexeme.as_str(), "outcome" | "race")
                    {
                        return Err(ParseError::new(
                            "expected `outcome` or `race` after `as` in await expression",
                            marker.span.line_start,
                            marker.span.col_start,
                        ));
                    }
                    if marker.lexeme == "race" {
                        AwaitMode::Race
                    } else {
                        AwaitMode::Outcome
                    }
                } else {
                    AwaitMode::Propagate
                };
                let id = self.alloc_expr_id();
                Ok(Expr::Await {
                    id,
                    task: Box::new(task),
                    mode,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::Spawn => {
                let sp = self.span();
                if self.concurrent_depth == 0 {
                    return Err(ParseError::new(
                        "`spawn` is only allowed inside a `concurrent` block",
                        sp.line_start,
                        sp.col_start,
                    ));
                }
                self.advance();
                let call = self.parse_expr_bp(11)?;
                if !matches!(call, Expr::Call { .. }) {
                    return Err(ParseError::new(
                        "`spawn` operand must be a call expression",
                        call.span().line_start,
                        call.span().col_start,
                    ));
                }
                let id = self.alloc_expr_id();
                Ok(Expr::Spawn {
                    id,
                    call: Box::new(call),
                    span: self.span_to_prev(&sp),
                })
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let sp = self.span();
        match *self.peek_kind() {
            TokenKind::Integer => {
                let tok = self.advance().clone();
                let id = self.alloc_expr_id();
                Ok(Expr::Number {
                    id,
                    value: tok.lexeme,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::FieldLit => {
                let tok = self.advance().clone();
                let (value, radix) = if let Some(hex) = tok.lexeme.strip_prefix('x') {
                    (hex.to_string(), FieldRadix::Hex)
                } else if let Some(bin) = tok.lexeme.strip_prefix('b') {
                    (bin.to_string(), FieldRadix::Binary)
                } else {
                    (tok.lexeme, FieldRadix::Decimal)
                };
                let id = self.alloc_expr_id();
                Ok(Expr::FieldLit {
                    id,
                    value,
                    radix,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::BigIntLit => {
                let tok = self.advance().clone();
                // Lexeme format: "256xFF00" or "512d42" or "256b1010"
                // Find the radix char position (first non-digit after start)
                let width_end = tok
                    .lexeme
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(tok.lexeme.len());
                let width: u16 = tok.lexeme[..width_end].parse().map_err(|_| {
                    ParseError::new("invalid BigInt width", sp.line_start, sp.col_start)
                })?;
                let rest = &tok.lexeme[width_end..];
                let (value, radix) = if let Some(hex) = rest.strip_prefix('x') {
                    (hex.to_string(), BigIntRadix::Hex)
                } else if let Some(dec) = rest.strip_prefix('d') {
                    (dec.to_string(), BigIntRadix::Decimal)
                } else if let Some(bin) = rest.strip_prefix('b') {
                    (bin.to_string(), BigIntRadix::Binary)
                } else {
                    return Err(ParseError::new(
                        "invalid BigInt literal radix",
                        sp.line_start,
                        sp.col_start,
                    ));
                };
                let id = self.alloc_expr_id();
                Ok(Expr::BigIntLit {
                    id,
                    value,
                    width,
                    radix,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::StringLit => {
                let tok = self.advance().clone();
                let id = self.alloc_expr_id();
                Ok(Expr::StringLit {
                    id,
                    value: tok.lexeme,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::True => {
                self.advance();
                let id = self.alloc_expr_id();
                Ok(Expr::Bool {
                    id,
                    value: true,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::False => {
                self.advance();
                let id = self.alloc_expr_id();
                Ok(Expr::Bool {
                    id,
                    value: false,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::Nil => {
                self.advance();
                let id = self.alloc_expr_id();
                Ok(Expr::Nil {
                    id,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::Ident => {
                let tok = self.advance().clone();
                // Check for `Type::member` static access
                if self.at(&TokenKind::ColonColon) {
                    self.advance(); // eat `::`
                    let member = self.expect_ident()?;
                    let id = self.alloc_expr_id();
                    return Ok(Expr::StaticAccess {
                        id,
                        type_name: tok.lexeme,
                        member,
                        span: self.span_to_prev(&sp),
                    });
                }
                let id = self.alloc_expr_id();
                Ok(Expr::Ident {
                    id,
                    name: tok.lexeme,
                    span: self.span_to_prev(&sp),
                })
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => self.parse_array(),
            TokenKind::LBrace => self.parse_brace_expr(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Forever => self.parse_forever(),
            TokenKind::Concurrent => self.parse_concurrent(),
            TokenKind::Fn => self.parse_fn_expr(),
            TokenKind::Prove => self.parse_prove(),
            _ => {
                let tok = self.peek();
                Err(ParseError::new(
                    format!("expected expression, found `{}`", tok_display(tok)),
                    tok.span.line_start,
                    tok.span.col_start,
                ))
            }
        }
    }
}
