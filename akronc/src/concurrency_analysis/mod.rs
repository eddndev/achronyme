//! Static checks that make the first concurrency release safe by default.

mod captures;
mod ownership;

use achronyme_parser::ast::Program;
use achronyme_parser::Diagnostic;
use akron::specs::NativeMeta;

pub(crate) struct AnalysisOutput {
    pub(crate) error: Option<Box<Diagnostic>>,
    pub(crate) warnings: Vec<Diagnostic>,
}

pub(crate) fn analyze(program: &Program, extra_natives: &[NativeMeta]) -> AnalysisOutput {
    AnalysisOutput {
        error: ownership::check(program, extra_natives).err(),
        warnings: captures::warnings(program),
    }
}
