use achronyme_parser::ast::Span;
use achronyme_parser::Diagnostic;
use akron::specs::ResourceKind;

use super::super::{OwnershipResult, OwnershipState};
use super::Analyzer;

impl<'a> Analyzer<'a> {
    pub(super) fn use_binding(
        &self,
        name: &str,
        use_span: &Span,
    ) -> OwnershipResult<Option<ResourceKind>> {
        let Some(resource) = self
            .binding(name)
            .and_then(|binding| binding.resource.as_ref())
        else {
            return Ok(None);
        };
        match &resource.state {
            OwnershipState::Active => Ok(Some(resource.kind)),
            OwnershipState::Unavailable {
                span,
                action,
                closed,
            } => {
                let verb = if *closed { "closed" } else { "moved" };
                Err(Box::new(
                    Diagnostic::error(
                        format!("owned resource `{name}` was {verb} and cannot be used again"),
                        use_span.into(),
                    )
                    .with_code("E023")
                    .with_label(span.into(), action.clone()),
                ))
            }
        }
    }

    pub(super) fn mark_unavailable(
        &mut self,
        name: &str,
        span: &Span,
        action: String,
        closed: bool,
    ) -> OwnershipResult<()> {
        self.use_binding(name, span)?;
        let Some(resource) = self
            .binding_mut(name)
            .and_then(|binding| binding.resource.as_mut())
        else {
            return Ok(());
        };
        resource.state = OwnershipState::Unavailable {
            span: span.clone(),
            action,
            closed,
        };
        Ok(())
    }

    pub(super) fn require_kind(
        &self,
        actual: Option<ResourceKind>,
        expected: ResourceKind,
        span: &Span,
    ) -> OwnershipResult<()> {
        if actual.is_some_and(|actual| actual != expected) {
            return Err(Box::new(
                Diagnostic::error(
                    format!("resource kind mismatch: expected {expected:?}"),
                    span.into(),
                )
                .with_code("E025"),
            ));
        }
        Ok(())
    }
}
