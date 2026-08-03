use achronyme_parser::ast::{AwaitMode, Block, ElseBranch, Expr, ForIterable, Span, Stmt};

use super::graph::{call_targets, chain, effects_for_symbol};
use super::EffectInferenceResult;
use crate::annotate::ResolvedProgram;
use crate::module_graph::{ModuleGraph, ModuleId};
use crate::{EffectSet, ResolveError, SymbolTable};

pub(super) fn validate(
    table: &SymbolTable,
    graph: &ModuleGraph,
    resolved: &ResolvedProgram,
    effects: &EffectInferenceResult,
) -> Vec<ResolveError> {
    let mut diagnostics = Vec::new();
    for module in graph.iter() {
        let mut validator = Validator {
            table,
            resolved,
            effects,
            module: module.id,
            diagnostics: &mut diagnostics,
        };
        for stmt in &module.program.stmts {
            validator.stmt(stmt, None);
        }
    }
    diagnostics
}

struct Validator<'a> {
    table: &'a SymbolTable,
    resolved: &'a ResolvedProgram,
    effects: &'a EffectInferenceResult,
    module: ModuleId,
    diagnostics: &'a mut Vec<ResolveError>,
}

impl Validator<'_> {
    fn stmt(&mut self, stmt: &Stmt, restricted: Option<&'static str>) {
        match stmt {
            Stmt::LetDecl { value, .. } | Stmt::MutDecl { value, .. } => {
                self.expr(value, restricted, false)
            }
            Stmt::Assignment { target, value, .. } => {
                self.expr(target, restricted, false);
                self.expr(value, restricted, false);
            }
            Stmt::FnDecl { body, .. } => self.block(body, restricted),
            Stmt::CircuitDecl { body, .. } => self.block(body, Some("circuit")),
            Stmt::Print { value, span } => {
                self.reject_restricted(
                    span,
                    restricted,
                    EffectSet::IO_CONSOLE,
                    vec!["print statement".to_string()],
                );
                self.expr(value, restricted, false);
            }
            Stmt::Return {
                value: Some(value), ..
            }
            | Stmt::Expr(value) => self.expr(value, restricted, false),
            Stmt::Export { inner, .. } => self.stmt(inner, restricted),
            _ => {}
        }
    }

    fn block(&mut self, block: &Block, restricted: Option<&'static str>) {
        for stmt in &block.stmts {
            self.stmt(stmt, restricted);
        }
    }

    fn expr(&mut self, expr: &Expr, restricted: Option<&'static str>, task_call_allowed: bool) {
        let intrinsic = self
            .effects
            .expressions
            .get(&(self.module, expr.id()))
            .copied()
            .unwrap_or_default();
        let forbidden = intrinsic.intersection(EffectSet::FORBIDDEN_IN_CIRCUIT);
        if !forbidden.is_empty() {
            let path = self.path_for(expr, forbidden);
            self.reject_restricted(expr.span(), restricted, forbidden, path);
        }
        if restricted.is_none()
            && matches!(expr, Expr::Call { .. })
            && intrinsic.contains(EffectSet::TASK)
            && !task_call_allowed
            && !self.is_explicit_cancel_check(expr)
        {
            self.diagnostics.push(ResolveError::MissingAwait {
                span: expr.span().clone(),
                effects: intrinsic,
                effect_path: self.path_for(expr, EffectSet::TASK),
            });
        }

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
                self.expr(lhs, restricted, false);
                self.expr(rhs, restricted, false);
            }
            Expr::UnaryOp { operand, .. } => self.expr(operand, restricted, false),
            Expr::Call { callee, args, .. } => {
                self.expr(callee, restricted, false);
                for arg in args {
                    self.expr(&arg.value, restricted, false);
                }
            }
            Expr::Index { object, index, .. } => {
                self.expr(object, restricted, false);
                self.expr(index, restricted, false);
            }
            Expr::DotAccess { object, .. } => self.expr(object, restricted, false),
            Expr::If {
                condition,
                then_block,
                else_branch,
                ..
            } => {
                self.expr(condition, restricted, false);
                self.block(then_block, restricted);
                match else_branch {
                    Some(ElseBranch::Block(block)) => self.block(block, restricted),
                    Some(ElseBranch::If(expr)) => self.expr(expr, restricted, false),
                    None => {}
                }
            }
            Expr::For { iterable, body, .. } => {
                match iterable {
                    ForIterable::Range { .. } => {}
                    ForIterable::ExprRange { end, .. } => self.expr(end, restricted, false),
                    ForIterable::Expr(expr) => self.expr(expr, restricted, false),
                }
                self.block(body, restricted);
            }
            Expr::While {
                condition, body, ..
            } => {
                self.expr(condition, restricted, false);
                self.block(body, restricted);
            }
            Expr::Forever { body, .. }
            | Expr::Concurrent { body, .. }
            | Expr::Block { block: body, .. }
            | Expr::FnExpr { body, .. } => self.block(body, restricted),
            Expr::Spawn { call, .. } => self.expr(call, restricted, true),
            Expr::Await { task, mode, .. } => {
                if *mode == AwaitMode::Race {
                    if let Expr::Array { elements, .. } = task.as_ref() {
                        for element in elements {
                            self.expr(element, restricted, true);
                        }
                    } else {
                        self.expr(task, restricted, true);
                    }
                } else {
                    self.expr(task, restricted, true);
                }
            }
            Expr::Prove { body, .. } => self.block(body, Some("prove")),
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.expr(element, restricted, false);
                }
            }
            Expr::Map { pairs, .. } => {
                for (_, value) in pairs {
                    self.expr(value, restricted, false);
                }
            }
        }
    }

    fn is_explicit_cancel_check(&self, expr: &Expr) -> bool {
        let Expr::Call { id, callee, .. } = expr else {
            return false;
        };
        let targets = call_targets(self.module, *id, callee, self.resolved);
        let [symbol] = targets.symbols.as_slice() else {
            return false;
        };
        let Ok(symbol) = self.table.resolve_alias(*symbol) else {
            return false;
        };
        let crate::CallableKind::Builtin { entry_index } = self.table.get(symbol) else {
            return false;
        };
        self.table
            .builtin_registry()
            .get(*entry_index)
            .is_some_and(|entry| entry.name == "cancel_check")
    }

    fn reject_restricted(
        &mut self,
        span: &Span,
        restricted: Option<&'static str>,
        effects: EffectSet,
        effect_path: Vec<String>,
    ) {
        let Some(context) = restricted else {
            return;
        };
        self.diagnostics.push(ResolveError::RestrictedEffect {
            span: span.clone(),
            context,
            effects,
            effect_path,
        });
    }

    fn path_for(&self, expr: &Expr, requested: EffectSet) -> Vec<String> {
        let Expr::Call { id, callee, .. } = expr else {
            return vec![match expr {
                Expr::Concurrent { .. } => "concurrent scope",
                Expr::Spawn { .. } => "spawn expression",
                Expr::Await { .. } => "await expression",
                _ => "effectful expression",
            }
            .to_string()];
        };
        let targets = call_targets(self.module, *id, callee, self.resolved);
        if targets.unknown {
            return vec!["unresolved dynamic call".to_string()];
        }
        for symbol in targets.symbols {
            let symbol = self.table.resolve_alias(symbol).unwrap_or(symbol);
            if effects_for_symbol(self.table, &self.effects.functions, symbol).intersects(requested)
            {
                let path = chain(self.effects, self.table, symbol, requested);
                if !path.is_empty() {
                    return path;
                }
            }
        }
        Vec::new()
    }
}
