use std::collections::HashMap;

use achronyme_parser::ast::{Block, ElseBranch, Expr, ForIterable, Stmt};

use super::{EffectCause, EffectInferenceResult};
use crate::annotate::ResolvedProgram;
use crate::module_graph::{ModuleGraph, ModuleId};
use crate::symbol::{CallableKind, SymbolId};
use crate::{EffectSet, SymbolTable};

#[derive(Clone, Debug, Default)]
pub(super) struct CallTargets {
    pub(super) symbols: Vec<SymbolId>,
    pub(super) unknown: bool,
    pub(super) intrinsic: EffectSet,
}

#[derive(Default)]
struct FunctionInfo {
    direct_effects: EffectSet,
    direct_causes: Vec<(EffectSet, String)>,
    calls: Vec<CallTargets>,
}

impl FunctionInfo {
    fn add_effect(&mut self, effects: EffectSet, label: &str) {
        let added = effects.difference(self.direct_effects);
        self.direct_effects |= effects;
        if !added.is_empty() {
            self.direct_causes.push((added, label.to_string()));
        }
    }
}

pub(super) fn infer(
    table: &SymbolTable,
    graph: &ModuleGraph,
    resolved: &ResolvedProgram,
) -> EffectInferenceResult {
    let mut infos = HashMap::new();
    let mut function_ids = Vec::new();
    for (symbol, kind) in table.iter() {
        let CallableKind::UserFn {
            module, stmt_index, ..
        } = kind
        else {
            continue;
        };
        let Some(body) = graph
            .get(*module)
            .program
            .stmts
            .get(*stmt_index as usize)
            .and_then(function_body)
        else {
            continue;
        };
        let mut info = FunctionInfo::default();
        collect_block(&mut info, body, *module, resolved);
        infos.insert(symbol, info);
        function_ids.push(symbol);
    }

    let mut result = EffectInferenceResult::default();
    for symbol in &function_ids {
        let info = &infos[symbol];
        result.functions.insert(*symbol, info.direct_effects);
        for (effects, label) in &info.direct_causes {
            result.causes.entry(*symbol).or_default().push(EffectCause {
                effects: *effects,
                via: None,
                label: label.clone(),
            });
        }
    }

    loop {
        let mut changed = false;
        for symbol in &function_ids {
            let mut current = result.functions[symbol];
            for call in &infos[symbol].calls {
                if !call.intrinsic.is_empty() {
                    add_cause(
                        &mut result,
                        *symbol,
                        &mut current,
                        call.intrinsic,
                        None,
                        "circom template call",
                    );
                }
                if call.unknown {
                    add_cause(
                        &mut result,
                        *symbol,
                        &mut current,
                        EffectSet::UNKNOWN_HOST,
                        None,
                        "unresolved dynamic call",
                    );
                }
                for target in &call.symbols {
                    let resolved_target = table.resolve_alias(*target).unwrap_or(*target);
                    let target_effects =
                        effects_for_symbol(table, &result.functions, resolved_target);
                    let (via, label) = match table.try_get(resolved_target) {
                        Some(CallableKind::UserFn { qualified_name, .. }) => {
                            (Some(resolved_target), qualified_name.clone())
                        }
                        Some(CallableKind::Builtin { entry_index }) => {
                            let name = table
                                .builtin_registry()
                                .get(*entry_index)
                                .map_or("unknown builtin", |entry| entry.name);
                            (None, format!("builtin `{name}`"))
                        }
                        Some(CallableKind::CircomTemplate { template_name, .. }) => {
                            (None, format!("circuit `{template_name}`"))
                        }
                        _ => (None, format!("symbol {resolved_target}")),
                    };
                    add_cause(
                        &mut result,
                        *symbol,
                        &mut current,
                        target_effects,
                        via,
                        &label,
                    );
                }
            }
            if current != result.functions[symbol] {
                result.functions.insert(*symbol, current);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    super::expressions::collect(table, graph, resolved, &mut result);
    result
}

fn add_cause(
    result: &mut EffectInferenceResult,
    symbol: SymbolId,
    current: &mut EffectSet,
    effects: EffectSet,
    via: Option<SymbolId>,
    label: &str,
) {
    let added = effects.difference(*current);
    if added.is_empty() {
        return;
    }
    *current |= added;
    result.causes.entry(symbol).or_default().push(EffectCause {
        effects: added,
        via,
        label: label.to_string(),
    });
}

fn function_body(stmt: &Stmt) -> Option<&Block> {
    match stmt {
        Stmt::FnDecl { body, .. } => Some(body),
        Stmt::Export { inner, .. } => function_body(inner),
        _ => None,
    }
}

fn collect_block(
    info: &mut FunctionInfo,
    block: &Block,
    module: ModuleId,
    resolved: &ResolvedProgram,
) {
    for stmt in &block.stmts {
        collect_stmt(info, stmt, module, resolved);
    }
}

fn collect_stmt(
    info: &mut FunctionInfo,
    stmt: &Stmt,
    module: ModuleId,
    resolved: &ResolvedProgram,
) {
    match stmt {
        Stmt::LetDecl { value, .. } | Stmt::MutDecl { value, .. } => {
            collect_expr(info, value, module, resolved)
        }
        Stmt::Assignment { target, value, .. } => {
            collect_expr(info, target, module, resolved);
            collect_expr(info, value, module, resolved);
        }
        Stmt::Print { value, .. } => {
            info.add_effect(EffectSet::IO_CONSOLE, "print statement");
            collect_expr(info, value, module, resolved);
        }
        Stmt::Return {
            value: Some(value), ..
        }
        | Stmt::Expr(value) => collect_expr(info, value, module, resolved),
        Stmt::Export { inner, .. } => collect_stmt(info, inner, module, resolved),
        Stmt::CircuitDecl { .. } => info.add_effect(EffectSet::CIRCUIT, "circuit declaration"),
        _ => {}
    }
}

fn collect_expr(
    info: &mut FunctionInfo,
    expr: &Expr,
    module: ModuleId,
    resolved: &ResolvedProgram,
) {
    match expr {
        Expr::Number { .. }
        | Expr::FieldLit { .. }
        | Expr::BigIntLit { .. }
        | Expr::Bool { .. }
        | Expr::StringLit { .. }
        | Expr::Nil { .. }
        | Expr::Ident { .. }
        | Expr::StaticAccess { .. }
        | Expr::Error { .. } => {}
        Expr::BinOp { lhs, rhs, .. } => {
            collect_expr(info, lhs, module, resolved);
            collect_expr(info, rhs, module, resolved);
        }
        Expr::UnaryOp { operand, .. } => collect_expr(info, operand, module, resolved),
        Expr::Call {
            id, callee, args, ..
        } => {
            info.calls.push(call_targets(module, *id, callee, resolved));
            collect_expr(info, callee, module, resolved);
            for arg in args {
                collect_expr(info, &arg.value, module, resolved);
            }
        }
        Expr::Index { object, index, .. } => {
            collect_expr(info, object, module, resolved);
            collect_expr(info, index, module, resolved);
        }
        Expr::DotAccess { object, .. } => collect_expr(info, object, module, resolved),
        Expr::If {
            condition,
            then_block,
            else_branch,
            ..
        } => {
            collect_expr(info, condition, module, resolved);
            collect_block(info, then_block, module, resolved);
            match else_branch {
                Some(ElseBranch::Block(block)) => collect_block(info, block, module, resolved),
                Some(ElseBranch::If(expr)) => collect_expr(info, expr, module, resolved),
                None => {}
            }
        }
        Expr::For { iterable, body, .. } => {
            collect_iterable(info, iterable, module, resolved);
            collect_block(info, body, module, resolved);
        }
        Expr::While {
            condition, body, ..
        } => {
            collect_expr(info, condition, module, resolved);
            collect_block(info, body, module, resolved);
        }
        Expr::Forever { body, .. } | Expr::Block { block: body, .. } => {
            collect_block(info, body, module, resolved)
        }
        Expr::Concurrent { body, .. } => {
            info.add_effect(EffectSet::TASK, "concurrent scope");
            collect_block(info, body, module, resolved);
        }
        Expr::Spawn { call, .. } => {
            info.add_effect(EffectSet::TASK, "spawn expression");
            collect_expr(info, call, module, resolved);
        }
        Expr::Await { task, .. } => {
            info.add_effect(EffectSet::TASK, "await expression");
            collect_expr(info, task, module, resolved);
        }
        Expr::Prove { .. } => info.add_effect(EffectSet::PROVE, "prove expression"),
        Expr::FnExpr { .. } => {}
        Expr::Array { elements, .. } => {
            for element in elements {
                collect_expr(info, element, module, resolved);
            }
        }
        Expr::Map { pairs, .. } => {
            for (_, value) in pairs {
                collect_expr(info, value, module, resolved);
            }
        }
    }
}

fn collect_iterable(
    info: &mut FunctionInfo,
    iterable: &ForIterable,
    module: ModuleId,
    resolved: &ResolvedProgram,
) {
    match iterable {
        ForIterable::Range { .. } => {}
        ForIterable::ExprRange { end, .. } => collect_expr(info, end, module, resolved),
        ForIterable::Expr(expr) => collect_expr(info, expr, module, resolved),
    }
}

pub(super) fn call_targets(
    module: ModuleId,
    call_id: achronyme_parser::ast::ExprId,
    callee: &Expr,
    resolved: &ResolvedProgram,
) -> CallTargets {
    if resolved.circom_calls.contains(&(module, call_id)) {
        return CallTargets {
            symbols: Vec::new(),
            unknown: false,
            intrinsic: EffectSet::CIRCUIT,
        };
    }
    if let Some(targets) = resolved.call_targets.get(&(module, call_id)) {
        return CallTargets {
            symbols: targets.clone(),
            unknown: false,
            intrinsic: EffectSet::empty(),
        };
    }
    if let Some(symbol) = resolved.annotations.get(&(module, callee.id())) {
        return CallTargets {
            symbols: vec![*symbol],
            unknown: false,
            intrinsic: EffectSet::empty(),
        };
    }
    CallTargets {
        symbols: Vec::new(),
        unknown: matches!(callee, Expr::Ident { .. } | Expr::FnExpr { .. }),
        intrinsic: EffectSet::empty(),
    }
}

pub(super) fn effects_for_symbol(
    table: &SymbolTable,
    functions: &HashMap<SymbolId, EffectSet>,
    symbol: SymbolId,
) -> EffectSet {
    match table.try_get(symbol) {
        Some(CallableKind::Builtin { entry_index }) => table
            .builtin_registry()
            .get(*entry_index)
            .map_or(EffectSet::UNKNOWN_HOST, |entry| entry.effects),
        Some(CallableKind::UserFn { .. }) => functions.get(&symbol).copied().unwrap_or_default(),
        Some(CallableKind::CircomTemplate { .. }) => EffectSet::CIRCUIT,
        Some(CallableKind::FnAlias { target }) => effects_for_symbol(table, functions, *target),
        Some(CallableKind::Constant { .. }) => EffectSet::empty(),
        None => EffectSet::UNKNOWN_HOST,
    }
}

pub(super) fn chain(
    result: &EffectInferenceResult,
    table: &SymbolTable,
    symbol: SymbolId,
    effect: EffectSet,
) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = symbol;
    let mut seen = std::collections::HashSet::new();
    while seen.insert(current) {
        let name = match table.try_get(current) {
            Some(CallableKind::UserFn { qualified_name, .. }) => qualified_name.clone(),
            _ => format!("{current}"),
        };
        chain.push(name);
        let Some(cause) = result
            .causes
            .get(&current)
            .and_then(|causes| causes.iter().find(|cause| cause.effects.intersects(effect)))
        else {
            break;
        };
        if let Some(next) = cause.via {
            current = next;
        } else {
            chain.push(cause.label.clone());
            break;
        }
    }
    chain
}
