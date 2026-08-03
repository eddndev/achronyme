use std::collections::HashMap;
use std::path::{Path, PathBuf};

use achronyme_parser::ast::Stmt;
use achronyme_parser::parse_program;

use super::*;
use crate::annotate::{annotate_program, register_all, register_builtins};
use crate::builtins::{BuiltinEntry, BuiltinRegistry, VmFnHandle};
use crate::module_graph::{LoadedModule, ModuleSource};
use crate::symbol::{Arity, Availability};
use crate::{CancellationPolicy, CapabilitySet, NativeBehavior, ResourceEffect};

#[derive(Default)]
struct MockSource {
    files: HashMap<String, String>,
}

impl MockSource {
    fn with_main(source: &str) -> Self {
        Self {
            files: HashMap::from([("main".to_string(), source.to_string())]),
        }
    }
}

impl ModuleSource for MockSource {
    fn canonicalize(
        &mut self,
        _importer: Option<&Path>,
        relative: &str,
    ) -> Result<PathBuf, String> {
        self.files
            .contains_key(relative)
            .then(|| PathBuf::from(relative))
            .ok_or_else(|| format!("no such module `{relative}`"))
    }

    fn load(&mut self, canonical: &Path) -> Result<LoadedModule, String> {
        let key = canonical.to_string_lossy();
        let source = self.files.get(key.as_ref()).ok_or("missing source")?;
        let (program, errors) = parse_program(source);
        if !errors.is_empty() {
            return Err(errors[0].message.clone());
        }
        let exported_names = program
            .stmts
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Export { inner, .. } => match inner.as_ref() {
                    Stmt::FnDecl { name, .. } | Stmt::LetDecl { name, .. } => Some(name.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        Ok(LoadedModule {
            program,
            exported_names,
        })
    }
}

fn analyze_with_registry(
    source: &str,
    registry: BuiltinRegistry,
) -> (
    SymbolTable,
    ModuleGraph,
    ResolvedProgram,
    EffectInferenceResult,
) {
    let mut source = MockSource::with_main(source);
    let graph = ModuleGraph::build("main", &mut source).unwrap();
    let mut table = SymbolTable::with_registry(registry).unwrap();
    register_builtins(&mut table);
    register_all(&mut table, &graph).unwrap();
    let resolved = annotate_program(&graph, &table);
    let effects = infer_effects(&table, &graph, &resolved);
    (table, graph, resolved, effects)
}

fn analyze(
    source: &str,
) -> (
    SymbolTable,
    ModuleGraph,
    ResolvedProgram,
    EffectInferenceResult,
) {
    analyze_with_registry(source, BuiltinRegistry::default())
}

fn registry_with_sleep() -> BuiltinRegistry {
    let mut registry = BuiltinRegistry::default();
    registry.push(BuiltinEntry {
        name: "sleep",
        arity: Arity::Fixed(1),
        availability: Availability::Vm,
        vm_fn: Some(VmFnHandle(15)),
        prove_ir_lower: None,
        effects: EffectSet::TASK | EffectSet::IO_CLOCK,
        capabilities: CapabilitySet::CLOCK,
        behavior: NativeBehavior::Suspending,
        cancellation: CancellationPolicy::Cooperative,
        resource: ResourceEffect::None,
    });
    registry
}

#[test]
fn effects_propagate_through_calls_and_recursion() {
    let (table, _, _, effects) = analyze(
        "fn first() { second() }\n\
         fn second() { first(); time() }",
    );
    for name in ["first", "second"] {
        let symbol = table.lookup(name).unwrap();
        assert!(effects.functions[&symbol].contains(EffectSet::IO_CLOCK));
    }
}

#[test]
fn concurrent_syntax_adds_task_effect() {
    let (table, _, _, effects) = analyze(
        "fn work() { 1 }\n\
         fn run() { concurrent { let task = spawn work(); await task } }",
    );
    assert!(effects.functions[&table.lookup("run").unwrap()].contains(EffectSet::TASK));
    assert!(effects.functions[&table.lookup("work").unwrap()].is_empty());
}

#[test]
fn dynamic_fn_targets_union_their_effects() {
    let (table, _, _, effects) = analyze(
        "fn clocked() { time() }\n\
         fn pure() { 1 }\n\
         fn choose(flag) { let f = if flag { clocked } else { pure }; f() }",
    );
    assert!(effects.functions[&table.lookup("choose").unwrap()].contains(EffectSet::IO_CLOCK));
    assert!(!effects.functions[&table.lookup("choose").unwrap()].contains(EffectSet::UNKNOWN_HOST));
}

#[test]
fn suspending_call_requires_await() {
    let (table, graph, resolved, effects) =
        analyze_with_registry("fn wait() { sleep(1) }", registry_with_sleep());
    let diagnostics = validate_effects(&table, &graph, &resolved, &effects);
    assert!(matches!(
        diagnostics.as_slice(),
        [ResolveError::MissingAwait { .. }]
    ));
}

#[test]
fn awaited_and_spawned_suspending_calls_are_valid() {
    let (table, graph, resolved, effects) = analyze_with_registry(
        "fn waited() { await sleep(1) }\n\
         fn started() { concurrent { spawn sleep(1) } }",
        registry_with_sleep(),
    );
    assert!(validate_effects(&table, &graph, &resolved, &effects).is_empty());
}

#[test]
fn explicit_cancel_check_is_task_effectful_but_does_not_require_await() {
    let (table, graph, resolved, effects) = analyze("fn work() { cancel_check() }");
    assert!(effects.functions[&table.lookup("work").unwrap()].contains(EffectSet::TASK));
    assert!(validate_effects(&table, &graph, &resolved, &effects).is_empty());
}

#[test]
fn cancel_check_identity_not_name_controls_the_await_exception() {
    let (table, graph, resolved, effects) = analyze(
        "fn tasky() { concurrent { 1 } }\n\
         fn work() { let cancel_check = tasky; cancel_check() }",
    );
    assert!(matches!(
        validate_effects(&table, &graph, &resolved, &effects).as_slice(),
        [ResolveError::MissingAwait { .. }]
    ));
}

#[test]
fn explicit_cancel_check_remains_forbidden_in_prove() {
    let (table, graph, resolved, effects) = analyze("fn work() { prove { cancel_check() } }");
    assert!(matches!(
        validate_effects(&table, &graph, &resolved, &effects).as_slice(),
        [ResolveError::RestrictedEffect { .. }]
    ));
}

#[test]
fn suspending_calls_are_valid_race_entries_but_not_nested_arguments() {
    let (table, graph, resolved, effects) = analyze_with_registry(
        "fn valid() { concurrent { await [sleep(1), sleep(2)] as race } }",
        registry_with_sleep(),
    );
    assert!(validate_effects(&table, &graph, &resolved, &effects).is_empty());

    let (table, graph, resolved, effects) = analyze_with_registry(
        "fn wrap(value) { value }\n\
         fn invalid() { concurrent { await [wrap(sleep(1)), sleep(2)] as race } }",
        registry_with_sleep(),
    );
    assert!(matches!(
        validate_effects(&table, &graph, &resolved, &effects).as_slice(),
        [ResolveError::MissingAwait { .. }]
    ));
}

#[test]
fn indirect_host_effect_is_rejected_in_prove() {
    let (table, graph, resolved, effects) = analyze(
        "fn clocked() { time() }\n\
         fn proof() { prove { clocked() } }",
    );
    let diagnostics = validate_effects(&table, &graph, &resolved, &effects);
    assert!(matches!(
        diagnostics.as_slice(),
        [ResolveError::RestrictedEffect { context: "prove", effects, effect_path, .. }]
            if effects.contains(EffectSet::IO_CLOCK) && effect_path.iter().any(|step| step.contains("clocked"))
    ));
}

#[test]
fn unknown_dynamic_call_is_rejected_in_circuit() {
    let (table, graph, resolved, effects) = analyze("circuit Bad(cb: Public Field) { cb() }");
    let diagnostics = validate_effects(&table, &graph, &resolved, &effects);
    assert!(matches!(
        diagnostics.as_slice(),
        [ResolveError::RestrictedEffect { context: "circuit", effects, .. }]
            if effects.contains(EffectSet::UNKNOWN_HOST)
    ));
}

#[test]
fn selective_circom_template_call_is_not_unknown_host_effect() {
    let (table, graph, resolved, effects) = analyze(
        "import { Square } from \"fixture.circom\"\n\
         prove { Square()(1) }",
    );
    let diagnostics = validate_effects(&table, &graph, &resolved, &effects);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(resolved.circom_calls.len() >= 2);
    assert!(effects
        .expressions
        .values()
        .any(|effect| effect.contains(EffectSet::CIRCUIT)));
    assert!(!effects
        .expressions
        .values()
        .any(|effect| effect.contains(EffectSet::UNKNOWN_HOST)));
}

#[test]
fn namespaced_circom_template_is_a_valid_prove_call_shape() {
    let (table, graph, resolved, effects) = analyze(
        "import \"fixture.circom\" as P\n\
         prove { P.Square()(1) }",
    );
    let diagnostics = validate_effects(&table, &graph, &resolved, &effects);

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(resolved.circom_calls.len() >= 2);
}

#[test]
fn effects_propagate_across_namespace_imports() {
    let mut source = MockSource::default();
    source.files.insert(
        "lib".to_string(),
        "export fn clocked() { time() }".to_string(),
    );
    source.files.insert(
        "main".to_string(),
        "import \"lib\" as lib\nfn run() { lib::clocked() }".to_string(),
    );
    let graph = ModuleGraph::build("main", &mut source).unwrap();
    let mut table = SymbolTable::with_registry(BuiltinRegistry::default()).unwrap();
    register_builtins(&mut table);
    register_all(&mut table, &graph).unwrap();
    let resolved = annotate_program(&graph, &table);
    let effects = infer_effects(&table, &graph, &resolved);

    let run = table.lookup("run").unwrap();
    assert!(effects.functions[&run].contains(EffectSet::IO_CLOCK));
}

#[test]
fn resolver_build_appends_effect_diagnostics() {
    let mut source = MockSource::with_main("fn work() { concurrent { 1 } }\nwork()");
    let state = crate::build_resolver_state("main", &mut source).unwrap();

    assert!(matches!(
        state.resolved.diagnostics.as_slice(),
        [ResolveError::MissingAwait { .. }]
    ));
    assert!(
        state.effects.functions[&state.table.lookup("work").unwrap()].contains(EffectSet::TASK)
    );
}

#[test]
fn resolver_build_uses_external_native_metadata() {
    let mut source = MockSource::with_main("sleep(1)");
    let state =
        crate::build_resolver_state_with_registry("main", &mut source, registry_with_sleep())
            .unwrap();

    assert!(matches!(
        state.resolved.diagnostics.as_slice(),
        [ResolveError::MissingAwait { effects, .. }] if effects.contains(EffectSet::IO_CLOCK)
    ));
}
