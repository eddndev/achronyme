use std::collections::HashMap;

use achronyme_parser::ast::{Block, Expr, Program, Span};
use achronyme_parser::Diagnostic;
use akron::specs::{NativeMeta, ResourceEffect, ResourceKind};

mod analyzer;
mod discovery;
mod summaries;

use analyzer::Analyzer;
use discovery::collect_function_defs;
use summaries::infer_function_summaries;

#[derive(Clone)]
struct FunctionDef {
    params: Vec<String>,
    body: Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParamMode {
    kind: ResourceKind,
    consumes: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct FunctionSummary {
    params: Vec<String>,
    modes: Vec<Option<ParamMode>>,
    return_kind: Option<ResourceKind>,
}

#[derive(Clone)]
struct ResourceBinding {
    kind: ResourceKind,
    state: OwnershipState,
}

#[derive(Clone)]
enum OwnershipState {
    Active,
    Unavailable {
        span: Span,
        action: String,
        closed: bool,
    },
}

#[derive(Clone, Default)]
struct Binding {
    resource: Option<ResourceBinding>,
}

type NativeResources = HashMap<String, ResourceEffect>;
type OwnershipResult<T> = Result<T, Box<Diagnostic>>;

pub(super) fn check(program: &Program, extra_natives: &[NativeMeta]) -> OwnershipResult<()> {
    let natives = native_resources(extra_natives);
    let mut functions = HashMap::new();
    collect_function_defs(&program.stmts, &mut functions);
    let summaries = infer_function_summaries(&functions, &natives);

    Analyzer::new(&natives, &summaries).analyze_statements(&program.stmts)?;
    for (name, function) in &functions {
        let summary = summaries.get(name).cloned().unwrap_or_default();
        let mut analyzer = Analyzer::new(&natives, &summaries);
        analyzer.push_scope();
        for (index, param) in function.params.iter().enumerate() {
            let resource =
                summary
                    .modes
                    .get(index)
                    .and_then(|mode| *mode)
                    .map(|mode| ResourceBinding {
                        kind: mode.kind,
                        state: OwnershipState::Active,
                    });
            analyzer.declare(param, resource);
        }
        analyzer.analyze_statements(&function.body.stmts)?;
    }
    Ok(())
}

fn native_resources(extra_natives: &[NativeMeta]) -> NativeResources {
    let mut resources = HashMap::new();
    for entry in resolve::BuiltinRegistry::default().entries() {
        resources.insert(entry.name.to_string(), entry.resource);
    }
    for native in extra_natives {
        resources.insert(native.name.to_string(), native.resource);
    }
    resources
}

fn direct_ident(expression: &Expr) -> Option<(&str, &Span)> {
    match expression {
        Expr::Ident { name, span, .. } => Some((name, span)),
        _ => None,
    }
}

fn callee_name(expression: &Expr) -> Option<&str> {
    match expression {
        Expr::Ident { name, .. } => Some(name),
        _ => None,
    }
}

fn merge_scopes(
    before: &[HashMap<String, Binding>],
    left: &[HashMap<String, Binding>],
    right: &[HashMap<String, Binding>],
) -> Vec<HashMap<String, Binding>> {
    let mut merged = before.to_vec();
    for (scope_index, scope) in merged.iter_mut().enumerate() {
        for (name, binding) in scope {
            let Some(resource) = binding.resource.as_mut() else {
                continue;
            };
            let state = [left, right].into_iter().find_map(|branch| {
                branch
                    .get(scope_index)
                    .and_then(|scope| scope.get(name))
                    .and_then(|binding| binding.resource.as_ref())
                    .and_then(|resource| match &resource.state {
                        OwnershipState::Unavailable { .. } => Some(resource.state.clone()),
                        OwnershipState::Active => None,
                    })
            });
            if let Some(state) = state {
                resource.state = state;
            }
        }
    }
    merged
}

fn aggregate_resource_error(span: &Span) -> Box<Diagnostic> {
    Box::new(
        Diagnostic::error(
            "owned resources cannot be stored in arrays or maps in Achronyme 0.1.0",
            span.into(),
        )
        .with_code("E025")
        .with_note(
            "keep the opaque resource in one lexical binding so cleanup remains deterministic",
        ),
    )
}
