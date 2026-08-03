//! Canonical source rendering for parsed Achronyme programs.

mod expr;
mod stmt;

use std::fmt;

use crate::ast::Program;

/// Error returned when a recovered error node cannot be rendered as source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatError {
    message: &'static str,
}

impl FormatError {
    pub(super) const fn recovered_node(kind: &'static str) -> Self {
        Self { message: kind }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot format recovered {} node", self.message)
    }
}

impl std::error::Error for FormatError {}

/// Render a parsed program using the canonical four-space source style.
///
/// Source comments and other trivia are not present in the AST and therefore
/// are not reproduced. The output is stable, parseable, and idempotent for
/// every non-recovery AST node.
pub fn format_program(program: &Program) -> Result<String, FormatError> {
    let mut formatter = Formatter::default();
    for (index, statement) in program.stmts.iter().enumerate() {
        if index > 0 {
            formatter.output.push('\n');
        }
        formatter.write_stmt(statement)?;
    }
    if !program.stmts.is_empty() {
        formatter.output.push('\n');
    }
    Ok(formatter.output)
}

#[derive(Default)]
struct Formatter {
    output: String,
    indent: usize,
}

impl Formatter {
    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }
}
