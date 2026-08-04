//! Transitive callable-effect inference and source-level effect validation.

use std::collections::HashMap;

use crate::annotate::{AnnotationKey, ResolvedProgram};
use crate::module_graph::ModuleGraph;
use crate::symbol::SymbolId;
use crate::{EffectSet, ResolveError, SymbolTable};

mod expressions;
mod graph;
mod validate;

#[cfg(test)]
mod tests;

/// One source of effects inferred for a user function.
#[derive(Clone, Debug)]
pub struct EffectCause {
    /// Effect bits introduced by this cause.
    pub effects: EffectSet,
    /// User function to follow for transitive provenance.
    pub via: Option<SymbolId>,
    /// Terminal label for syntax, builtin, or unknown dynamic dispatch.
    pub label: String,
}

/// Complete output of the effect-inference fixed point.
#[derive(Clone, Debug, Default)]
pub struct EffectInferenceResult {
    /// Inferred effect set for every registered user function.
    pub functions: HashMap<SymbolId, EffectSet>,
    /// Inferred effects for source expressions keyed by module and expression id.
    pub expressions: HashMap<AnnotationKey, EffectSet>,
    /// Provenance used to render transitive effect paths.
    pub causes: HashMap<SymbolId, Vec<EffectCause>>,
}

impl EffectInferenceResult {
    /// Union every function-level and expression-level inferred effect.
    pub fn all_effects(&self) -> EffectSet {
        self.functions
            .values()
            .chain(self.expressions.values())
            .copied()
            .fold(EffectSet::empty(), |all, effects| all | effects)
    }
}

/// Infer effects for every user function and relevant expression.
pub fn infer_effects(
    table: &SymbolTable,
    graph: &ModuleGraph,
    resolved: &ResolvedProgram,
) -> EffectInferenceResult {
    graph::infer(table, graph, resolved)
}

/// Validate explicit suspension and proof/circuit effect boundaries.
pub fn validate_effects(
    table: &SymbolTable,
    graph: &ModuleGraph,
    resolved: &ResolvedProgram,
    effects: &EffectInferenceResult,
) -> Vec<ResolveError> {
    validate::validate(table, graph, resolved, effects)
}

/// Build a readable path from a function to one requested effect.
pub fn effect_chain(
    result: &EffectInferenceResult,
    table: &SymbolTable,
    symbol: SymbolId,
    effect: EffectSet,
) -> Vec<String> {
    graph::chain(result, table, symbol, effect)
}
