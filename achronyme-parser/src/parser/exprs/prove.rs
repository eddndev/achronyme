use crate::ast::{Expr, TypedParam, Visibility};
use crate::error::ParseError;
use crate::token::TokenKind;

use super::super::core::Parser;

impl Parser {
    pub(super) fn parse_prove(&mut self) -> Result<Expr, ParseError> {
        let sp = self.span();
        self.advance(); // eat `prove`

        // Optional name: `prove eligibility(hash: Public) { ... }`
        let name = if self.at(&TokenKind::Ident)
            && (matches!(self.lookahead(1), TokenKind::LParen | TokenKind::LBrace))
        {
            Some(self.expect_ident()?)
        } else {
            None
        };

        let params = self.parse_prove_params()?;
        let body = self.parse_block_inner()?;

        let id = self.alloc_expr_id();
        Ok(Expr::Prove {
            id,
            name,
            body,
            params,
            span: self.span_to_prev(&sp),
        })
    }

    /// Parse optional prove parameter list.
    ///
    /// New syntax: `(hash: Public, root: Public Field)` -- typed params with visibility.
    /// Old syntax: `(public: [x, y])` -- deprecated, converted to typed params.
    /// No parens: empty params (all old-style declarations or all-witness).
    pub(in crate::parser) fn parse_prove_params(&mut self) -> Result<Vec<TypedParam>, ParseError> {
        if !self.at(&TokenKind::LParen) {
            return Ok(Vec::new());
        }
        self.advance(); // eat `(`

        // Syntax: `(hash: Public, root: Public Field)`
        let mut params = Vec::new();
        while !self.at(&TokenKind::RParen) {
            let param_name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let type_ann = self.parse_type()?;

            // Only Public visibility is allowed in prove params
            // (witnesses are auto-captured from scope)
            match type_ann.visibility {
                Some(Visibility::Public) => {}
                Some(Visibility::Witness) => {
                    return Err(ParseError::new(
                        format!(
                            "prove parameter `{param_name}` cannot be `Witness` — \
                             witnesses are auto-captured from outer scope"
                        ),
                        0,
                        0,
                    ));
                }
                None => {
                    return Err(ParseError::new(
                        format!("prove parameter `{param_name}` requires `Public` annotation"),
                        0,
                        0,
                    ));
                }
            }

            params.push(TypedParam {
                name: param_name,
                type_ann: Some(type_ann),
            });

            if self.at(&TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(&TokenKind::RParen)?;
        Ok(params)
    }
}
