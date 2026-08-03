use achronyme_parser::ast::{Block, CallArg, ElseBranch, Expr, ForIterable, Span};
use achronyme_parser::Diagnostic;
use akron::specs::{ResourceEffect, ResourceKind};

use super::super::summaries::{created_kind, resource_argument_mode};
use super::super::{
    aggregate_resource_error, callee_name, direct_ident, merge_scopes, OwnershipResult,
};
use super::Analyzer;

impl<'a> Analyzer<'a> {
    pub(super) fn analyze_expr(
        &mut self,
        expression: &Expr,
    ) -> OwnershipResult<Option<ResourceKind>> {
        match expression {
            Expr::Ident { name, span, .. } => self.use_binding(name, span),
            Expr::Call { callee, args, .. } => self.analyze_call(callee, args),
            Expr::Spawn { call, span, .. } => self.analyze_spawn(call, span),
            Expr::Await { task, .. } => self.analyze_expr(task),
            Expr::BinOp { lhs, rhs, .. } => {
                self.analyze_expr(lhs)?;
                self.analyze_expr(rhs)?;
                Ok(None)
            }
            Expr::UnaryOp { operand, .. } => {
                self.analyze_expr(operand)?;
                Ok(None)
            }
            Expr::Index { object, index, .. } => {
                self.analyze_expr(object)?;
                self.analyze_expr(index)?;
                Ok(None)
            }
            Expr::DotAccess { object, .. } => {
                self.analyze_expr(object)?;
                Ok(None)
            }
            Expr::If {
                condition,
                then_block,
                else_branch,
                ..
            } => self.analyze_if(condition, then_block, else_branch.as_ref()),
            Expr::For { iterable, body, .. } => {
                match iterable {
                    ForIterable::ExprRange { end, .. } | ForIterable::Expr(end) => {
                        self.analyze_expr(end)?;
                    }
                    ForIterable::Range { .. } => {}
                }
                let before = self.scopes.clone();
                self.analyze_block(body)?;
                let after = self.scopes.clone();
                self.scopes = merge_scopes(&before, &after, &before);
                Ok(None)
            }
            Expr::While {
                condition, body, ..
            } => {
                self.analyze_expr(condition)?;
                let before = self.scopes.clone();
                self.analyze_block(body)?;
                let after = self.scopes.clone();
                self.scopes = merge_scopes(&before, &after, &before);
                Ok(None)
            }
            Expr::Forever { body, .. } | Expr::Concurrent { body, .. } => self.analyze_block(body),
            Expr::Block { block, .. } | Expr::Prove { body: block, .. } => {
                self.analyze_block(block)
            }
            Expr::FnExpr { params, body, .. } => {
                let saved = self.scopes.clone();
                self.push_scope();
                for param in params {
                    self.declare(&param.name, None);
                }
                self.analyze_statements(&body.stmts)?;
                self.scopes = saved;
                Ok(None)
            }
            Expr::Array { elements, span, .. } => {
                for element in elements {
                    if self.analyze_expr(element)?.is_some() {
                        return Err(aggregate_resource_error(span));
                    }
                }
                Ok(None)
            }
            Expr::Map { pairs, span, .. } => {
                for (_, value) in pairs {
                    if self.analyze_expr(value)?.is_some() {
                        return Err(aggregate_resource_error(span));
                    }
                }
                Ok(None)
            }
            Expr::Number { .. }
            | Expr::FieldLit { .. }
            | Expr::BigIntLit { .. }
            | Expr::Bool { .. }
            | Expr::StringLit { .. }
            | Expr::Nil { .. }
            | Expr::StaticAccess { .. }
            | Expr::Error { .. } => Ok(None),
        }
    }

    fn analyze_if(
        &mut self,
        condition: &Expr,
        then_block: &Block,
        else_branch: Option<&ElseBranch>,
    ) -> OwnershipResult<Option<ResourceKind>> {
        self.analyze_expr(condition)?;
        let before = self.scopes.clone();
        let left = self.analyze_block(then_block)?;
        let left_scopes = self.scopes.clone();
        self.scopes = before.clone();
        let right = match else_branch {
            Some(ElseBranch::Block(block)) => self.analyze_block(block)?,
            Some(ElseBranch::If(expression)) => self.analyze_expr(expression)?,
            None => None,
        };
        let right_scopes = self.scopes.clone();
        self.scopes = merge_scopes(&before, &left_scopes, &right_scopes);
        Ok((left == right).then_some(left).flatten())
    }

    fn analyze_call(
        &mut self,
        callee: &Expr,
        args: &[CallArg],
    ) -> OwnershipResult<Option<ResourceKind>> {
        let name = callee_name(callee);
        if name.is_none() {
            self.analyze_expr(callee)?;
        }
        if let Some(effect) = name.and_then(|name| self.natives.get(name)).copied() {
            return self.analyze_native_call(effect, args);
        }

        let mut argument_kinds = Vec::with_capacity(args.len());
        for argument in args {
            argument_kinds.push(self.analyze_expr(&argument.value)?);
        }
        if let Some(summary) = name.and_then(|name| self.functions.get(name)) {
            for ((argument, actual_kind), mode) in
                args.iter().zip(argument_kinds).zip(&summary.modes)
            {
                let Some(mode) = mode else { continue };
                self.require_kind(actual_kind, mode.kind, argument.value.span())?;
                if mode.consumes {
                    if let Some((binding, span)) = direct_ident(&argument.value) {
                        self.mark_unavailable(
                            binding,
                            span,
                            format!("consumed by call to `{}`", name.unwrap_or("function")),
                            true,
                        )?;
                    }
                }
            }
            return Ok(summary.return_kind);
        }
        Ok(None)
    }

    fn analyze_native_call(
        &mut self,
        effect: ResourceEffect,
        args: &[CallArg],
    ) -> OwnershipResult<Option<ResourceKind>> {
        let resource_argument = resource_argument_mode(effect);
        for (index, argument) in args.iter().enumerate() {
            let actual = self.analyze_expr(&argument.value)?;
            if index == 0 {
                if let Some((expected, consumes)) = resource_argument {
                    self.require_kind(actual, expected, argument.value.span())?;
                    if consumes {
                        if let Some((binding, span)) = direct_ident(&argument.value) {
                            self.mark_unavailable(
                                binding,
                                span,
                                "closed by this call".to_string(),
                                true,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(created_kind(effect))
    }

    fn analyze_spawn(
        &mut self,
        call: &Expr,
        spawn_span: &Span,
    ) -> OwnershipResult<Option<ResourceKind>> {
        let Expr::Call { callee, args, .. } = call else {
            self.analyze_expr(call)?;
            return Ok(None);
        };
        let name = callee_name(callee);
        if name.is_none() {
            self.analyze_expr(callee)?;
        }
        let native_effect = name.and_then(|name| self.natives.get(name)).copied();
        let borrowed_kind = match native_effect {
            Some(ResourceEffect::Borrows(kind)) => Some(kind),
            Some(ResourceEffect::CreatesAndBorrows { borrowed, .. }) => Some(borrowed),
            _ => None,
        };

        for (index, argument) in args.iter().enumerate() {
            let kind = self.analyze_expr(&argument.value)?;
            if index == 0
                && borrowed_kind.is_some_and(|borrowed| {
                    borrowed != ResourceKind::Channel && kind == Some(borrowed)
                })
            {
                return Err(Box::new(
                    Diagnostic::error(
                        "a borrowed resource cannot cross a `spawn` task boundary",
                        spawn_span.into(),
                    )
                    .with_code("E024")
                    .with_label(
                        argument.value.span().into(),
                        "this operation borrows the resource",
                    )
                    .with_note(
                        "transfer the resource to a user function, or await the borrowing operation in the current task",
                    ),
                ));
            }
            if let Some(kind) = kind {
                if kind != ResourceKind::Channel {
                    if let Some((binding, span)) = direct_ident(&argument.value) {
                        self.mark_unavailable(
                            binding,
                            span,
                            "moved into child task".to_string(),
                            false,
                        )?;
                    }
                }
            }
        }
        Ok(None)
    }
}
