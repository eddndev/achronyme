use achronyme_parser::ast::{Block, Expr, Stmt};
use achronyme_parser::Diagnostic;
use akron::specs::ResourceKind;

use super::super::{direct_ident, OwnershipResult, OwnershipState, ResourceBinding};
use super::Analyzer;

impl<'a> Analyzer<'a> {
    pub(crate) fn analyze_statements(&mut self, statements: &[Stmt]) -> OwnershipResult<()> {
        for statement in statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    pub(super) fn analyze_block(&mut self, block: &Block) -> OwnershipResult<Option<ResourceKind>> {
        self.push_scope();
        let mut result = None;
        for statement in &block.stmts {
            result = match statement {
                Stmt::Expr(expression) => self.analyze_expr(expression)?,
                _ => {
                    self.analyze_statement(statement)?;
                    None
                }
            };
        }
        self.pop_scope();
        Ok(result)
    }

    fn analyze_statement(&mut self, statement: &Stmt) -> OwnershipResult<()> {
        match statement {
            Stmt::LetDecl { name, value, .. } | Stmt::MutDecl { name, value, .. } => {
                let kind = self.analyze_expr(value)?;
                if let (Some(kind), Some((source, span))) = (kind, direct_ident(value)) {
                    if kind != ResourceKind::Channel {
                        self.mark_unavailable(source, span, format!("moved into `{name}`"), false)?;
                    }
                }
                self.declare(
                    name,
                    kind.map(|kind| ResourceBinding {
                        kind,
                        state: OwnershipState::Active,
                    }),
                );
            }
            Stmt::Assignment {
                target,
                value,
                span,
            } => {
                let kind = self.analyze_expr(value)?;
                if let (Some(kind), Some((source, source_span))) = (kind, direct_ident(value)) {
                    if kind != ResourceKind::Channel {
                        self.mark_unavailable(
                            source,
                            source_span,
                            "moved by assignment".to_string(),
                            false,
                        )?;
                    }
                }
                if let Expr::Ident { name, .. } = target {
                    if self.binding(name).is_some_and(|binding| {
                        binding.resource.as_ref().is_some_and(|resource| {
                            matches!(resource.state, OwnershipState::Active)
                        })
                    }) {
                        return Err(Box::new(
                            Diagnostic::error(
                                format!(
                                    "owned resource `{name}` cannot be overwritten before it is closed or moved"
                                ),
                                span.into(),
                            )
                            .with_code("E025"),
                        ));
                    }
                    if let Some(binding) = self.binding_mut(name) {
                        binding.resource = kind.map(|kind| ResourceBinding {
                            kind,
                            state: OwnershipState::Active,
                        });
                    }
                } else {
                    self.analyze_expr(target)?;
                }
            }
            Stmt::Print { value, .. } => {
                self.analyze_expr(value)?;
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.analyze_expr(value)?;
                }
            }
            Stmt::Export { inner, .. } => self.analyze_statement(inner)?,
            Stmt::Expr(expression) => {
                self.analyze_expr(expression)?;
            }
            Stmt::CircuitDecl { body, .. } => {
                self.analyze_block(body)?;
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
        Ok(())
    }
}
