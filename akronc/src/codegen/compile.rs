//! The `Compiler::compile` entry point and the `is_terminator`
//! free helper it relies on for unreachable-code warnings.
//!
//! Parsing → (optional) resolver auto-build → per-statement dispatch →
//! final `Return` emission → bytecode optimizer. The routine is
//! short and mostly just a conductor, so the bulk of the
//! non-trivial work sits in the submodules listed at the top of
//! `codegen/mod.rs`.

use achronyme_parser::ast::Stmt;
use achronyme_parser::Diagnostic;
use akron::opcode::OpCode;

use super::Compiler;
use crate::error::CompilerError;
use crate::statements::{stmt_span, StatementCompiler};
use resolve::ResolveError;

impl Compiler {
    pub fn compile(&mut self, source: &str) -> Result<Vec<u32>, CompilerError> {
        let (program, parse_errors) = achronyme_parser::parse_program(source);
        // Only reject actual errors, not warnings (W008, W010, etc.)
        if let Some(err) = parse_errors
            .iter()
            .find(|d| d.severity == achronyme_parser::Severity::Error)
        {
            return Err(CompilerError::DiagnosticError(Box::new(err.clone())));
        }
        // Collect parser warnings into our warning list
        for diag in parse_errors {
            if diag.severity == achronyme_parser::Severity::Warning {
                self.warnings.push(diag);
            }
        }

        // If no resolver state was pre-installed (via
        // `install_resolver_state`), try to build one from the
        // parsed root. Any failure is silent — the unannotated
        // dispatch path must not regress because of a resolver
        // hiccup.
        if self.resolved_program.is_none() {
            self.try_auto_build_resolver_state(&program);
        }

        // Effect and prove-shape diagnostics are correctness checks, not
        // optional resolver hints. Stop before bytecode emission so an
        // effectful call can never slip through the legacy dispatch path.
        if let Some(error) = self
            .resolved_program
            .as_ref()
            .and_then(|resolved| resolved.diagnostics.first())
        {
            return Err(resolve_diagnostic(error));
        }

        let concurrency =
            crate::concurrency_analysis::analyze(&program, &self.extra_native_metadata);
        self.warnings.extend(concurrency.warnings);
        if let Some(error) = concurrency.error {
            return Err(CompilerError::DiagnosticError(error));
        }

        let mut terminated = false;
        let mut unreachable_warned = false;
        for stmt in &program.stmts {
            if terminated && !unreachable_warned {
                if let Some(span) = stmt_span(stmt) {
                    self.emit_warning(
                        Diagnostic::warning("unreachable code", span.into()).with_code("W003"),
                    );
                }
                unreachable_warned = true;
            }
            self.compile_stmt(stmt)?;
            if !terminated && is_terminator(stmt) {
                terminated = true;
            }
        }

        // W005: unused selective imports
        for (name, (_path, span)) in &self.imported_names {
            if !self.used_imported_names.contains(name) && !name.starts_with('_') {
                self.warnings.push(
                    Diagnostic::warning(
                        format!("imported name `{name}` is never used"),
                        span.into(),
                    )
                    .with_code("W005"),
                );
            }
        }

        // Final return
        self.emit_abc(OpCode::Return, 0, 0, 0)?; // Return Nil/0

        let func = self.current()?;
        let (opt_bytecode, opt_line_info) = crate::optimizer::optimize(
            func.bytecode.clone(),
            func.line_info.clone(),
            &mut func.max_slots,
        );
        func.bytecode = opt_bytecode;
        func.line_info = opt_line_info;

        Ok(self.current()?.bytecode.clone())
    }
}

fn resolve_diagnostic(error: &ResolveError) -> CompilerError {
    let (span, code, note) = match error {
        ResolveError::ProveBlockUnsupportedShape { span, reason, .. } => {
            (span, "E020", Some((*reason).to_string()))
        }
        ResolveError::MissingAwait {
            span, effect_path, ..
        } => (
            span,
            "E021",
            (!effect_path.is_empty()).then(|| format!("effect path: {}", effect_path.join(" -> "))),
        ),
        ResolveError::RestrictedEffect {
            span, effect_path, ..
        } => (
            span,
            "E022",
            (!effect_path.is_empty()).then(|| format!("effect path: {}", effect_path.join(" -> "))),
        ),
        _ => {
            return CompilerError::InternalError(format!(
                "unexpected resolver diagnostic after graph construction: {error}"
            ));
        }
    };
    let mut diagnostic = Diagnostic::error(error.to_string(), span.into()).with_code(code);
    if let Some(note) = note {
        diagnostic = diagnostic.with_note(note);
    }
    CompilerError::DiagnosticError(Box::new(diagnostic))
}

/// Returns true if a statement is a control-flow terminator (return, break, continue).
pub(crate) fn is_terminator(stmt: &Stmt) -> bool {
    matches!(
        stmt,
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. }
    )
}
