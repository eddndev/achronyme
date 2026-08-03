pub mod ast;
pub mod diagnostic;
pub mod error;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod render;
pub mod token;

pub use diagnostic::{Diagnostic, Label, Severity, SpanRange, Suggestion};
pub use error::ParseError;
pub use formatter::{format_program, FormatError};
pub use lexer::unescape;
/// Re-export the hand-written parser as the primary API.
pub use parser::{parse_block, parse_program};
pub use render::{atty_stderr, ColorMode, DiagnosticRenderer};
