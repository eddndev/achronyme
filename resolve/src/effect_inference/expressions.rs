use std::collections::HashMap;

use achronyme_parser::ast::{Block, ElseBranch, Expr, ForIterable, Stmt};

use super::graph::{call_targets, effects_for_symbol};
use super::EffectInferenceResult;
use crate::annotate::{AnnotationKey, ResolvedProgram};
use crate::module_graph::{ModuleGraph, ModuleId};
use crate::symbol::SymbolId;
use crate::{EffectSet, SymbolTable};

pub(super) fn collect(
    table: &SymbolTable,
    graph: &ModuleGraph,
    resolved: &ResolvedProgram,
    result: &mut EffectInferenceResult,
) {
    for module in graph.iter() {
        let mut collector = ExpressionCollector {
            table,
            resolved,
            functions: &result.functions,
            module: module.id,
            output: &mut result.expressions,
        };
        for stmt in &module.program.stmts {
            collector.stmt(stmt);
        }
    }
}

struct ExpressionCollector<'a> {
    table: &'a SymbolTable,
    resolved: &'a ResolvedProgram,
    functions: &'a HashMap<SymbolId, EffectSet>,
    module: ModuleId,
    output: &'a mut HashMap<AnnotationKey, EffectSet>,
}

impl ExpressionCollector<'_> {
    fn record(&mut self, expr: &Expr, effects: EffectSet) {
        self.output.insert((self.module, expr.id()), effects);
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LetDecl { value, .. } | Stmt::MutDecl { value, .. } => self.expr(value),
            Stmt::Assignment { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Stmt::FnDecl { body, .. } | Stmt::CircuitDecl { body, .. } => self.block(body),
            Stmt::Print { value, .. } => self.expr(value),
            Stmt::Return {
                value: Some(value), ..
            }
            | Stmt::Expr(value) => self.expr(value),
            Stmt::Export { inner, .. } => self.stmt(inner),
            _ => {}
        }
    }

    fn block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.stmt(stmt);
        }
    }

    fn expr(&mut self, expr: &Expr) {
        let intrinsic = match expr {
            Expr::Call {
                id, callee, args, ..
            } => {
                self.expr(callee);
                for arg in args {
                    self.expr(&arg.value);
                }
                let targets = call_targets(self.module, *id, callee, self.resolved);
                let mut effects = targets.intrinsic;
                if targets.unknown {
                    effects |= EffectSet::UNKNOWN_HOST;
                }
                for symbol in targets.symbols {
                    let symbol = self.table.resolve_alias(symbol).unwrap_or(symbol);
                    effects |= effects_for_symbol(self.table, self.functions, symbol);
                }
                effects
            }
            Expr::Concurrent { body, .. } => {
                self.block(body);
                EffectSet::TASK
            }
            Expr::Spawn { call, .. } => {
                self.expr(call);
                EffectSet::TASK
            }
            Expr::Await { task, .. } => {
                self.expr(task);
                EffectSet::TASK
            }
            Expr::Prove { body, .. } => {
                self.block(body);
                EffectSet::PROVE
            }
            Expr::BinOp { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
                EffectSet::empty()
            }
            Expr::UnaryOp { operand, .. } => {
                self.expr(operand);
                EffectSet::empty()
            }
            Expr::Index { object, index, .. } => {
                self.expr(object);
                self.expr(index);
                EffectSet::empty()
            }
            Expr::DotAccess { object, .. } => {
                self.expr(object);
                EffectSet::empty()
            }
            Expr::If {
                condition,
                then_block,
                else_branch,
                ..
            } => {
                self.expr(condition);
                self.block(then_block);
                match else_branch {
                    Some(ElseBranch::Block(block)) => self.block(block),
                    Some(ElseBranch::If(expr)) => self.expr(expr),
                    None => {}
                }
                EffectSet::empty()
            }
            Expr::For { iterable, body, .. } => {
                match iterable {
                    ForIterable::Range { .. } => {}
                    ForIterable::ExprRange { end, .. } => self.expr(end),
                    ForIterable::Expr(expr) => self.expr(expr),
                }
                self.block(body);
                EffectSet::empty()
            }
            Expr::While {
                condition, body, ..
            } => {
                self.expr(condition);
                self.block(body);
                EffectSet::empty()
            }
            Expr::Forever { body, .. }
            | Expr::Block { block: body, .. }
            | Expr::FnExpr { body, .. } => {
                self.block(body);
                EffectSet::empty()
            }
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.expr(element);
                }
                EffectSet::empty()
            }
            Expr::Map { pairs, .. } => {
                for (_, value) in pairs {
                    self.expr(value);
                }
                EffectSet::empty()
            }
            _ => EffectSet::empty(),
        };
        self.record(expr, intrinsic);
    }
}
