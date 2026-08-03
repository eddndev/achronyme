use std::collections::{HashMap, HashSet};

use achronyme_parser::ast::Program;
use achronyme_parser::Diagnostic;

mod collect;
mod visit;
mod walk;

use collect::{close_transitive_captures, collect_function_summaries, collect_mutable_bindings};
use visit::visit_statements_for_concurrent;

#[derive(Clone, Default)]
struct CaptureSummary {
    direct: HashSet<String>,
    calls: HashSet<String>,
    transitive: HashSet<String>,
}

pub(super) fn warnings(program: &Program) -> Vec<Diagnostic> {
    let mut mutable_bindings = HashMap::new();
    collect_mutable_bindings(&program.stmts, &mut mutable_bindings);
    if mutable_bindings.is_empty() {
        return Vec::new();
    }

    let mut summaries = HashMap::new();
    collect_function_summaries(&program.stmts, &mut summaries);
    close_transitive_captures(&mut summaries);

    let mut output = Vec::new();
    visit_statements_for_concurrent(&program.stmts, &mutable_bindings, &summaries, &mut output);
    output
}
