use achronyme_parser::ast::{AwaitMode, Block, Expr};
use akron::opcode::OpCode;

use super::ExpressionCompiler;
use crate::codegen::Compiler;
use crate::control_flow::ControlFlowCompiler;
use crate::error::CompilerError;

impl Compiler {
    pub(super) fn try_compile_cancel_check_call(
        &mut self,
        callee: &Expr,
        args: &[&Expr],
    ) -> Result<Option<u8>, CompilerError> {
        if !self.is_resolved_builtin(callee, "cancel_check") {
            return Ok(None);
        }
        if !args.is_empty() {
            return Err(CompilerError::CompileError(
                "cancel_check() takes no arguments".into(),
                self.cur_span(),
            ));
        }
        self.emit_abc(OpCode::CancelCheck, 0, 0, 0)?;
        let result = self.alloc_reg()?;
        self.emit_abx(OpCode::LoadNil, result, 0)?;
        Ok(Some(result))
    }

    fn is_resolved_builtin(&self, callee: &Expr, expected: &str) -> bool {
        if let (Some(program), Some(table), Some(module)) = (
            self.resolved_program.as_ref(),
            self.resolver_symbol_table.as_ref(),
            self.resolver_root_module,
        ) {
            if let Some(symbol) = program.annotations.get(&(module, callee.id())) {
                if let Ok(symbol) = table.resolve_alias(*symbol) {
                    if let resolve::CallableKind::Builtin { entry_index } = table.get(symbol) {
                        return table
                            .builtin_registry()
                            .get(*entry_index)
                            .is_some_and(|entry| entry.name == expected);
                    }
                    return false;
                }
            }
        }

        let Expr::Ident { name, .. } = callee else {
            return false;
        };
        if name != expected
            || self
                .compilers
                .iter()
                .any(|compiler| compiler.resolve_local(name).is_some())
        {
            return false;
        }
        let Some(global) = self.global_symbols.get(name) else {
            return false;
        };
        self.native_registry
            .lookup(name)
            .and_then(|entry| entry.vm_fn)
            .is_some_and(|handle| global.index == handle.as_u32() as u16)
    }

    pub(super) fn compile_concurrent(&mut self, body: &Block) -> Result<u8, CompilerError> {
        let target = self.alloc_reg()?;
        self.emit_abc(OpCode::ScopeEnter, 0, 0, 0)?;
        self.current()?.concurrent_depth = self
            .current_ref()?
            .concurrent_depth
            .checked_add(1)
            .ok_or_else(|| {
                CompilerError::CompilerLimitation(
                    "too many nested concurrent scopes".into(),
                    self.cur_span(),
                )
            })?;
        self.compile_block(body, target)?;
        self.current()?.concurrent_depth -= 1;
        self.emit_abc(OpCode::ScopeExit, 0, 0, 0)?;
        Ok(target)
    }

    pub(super) fn compile_spawn(&mut self, call: &Expr) -> Result<u8, CompilerError> {
        let Expr::Call { callee, args, .. } = call else {
            return Err(CompilerError::InternalError(
                "parser produced spawn with a non-call operand".into(),
            ));
        };
        if args.iter().any(|arg| arg.name.is_some()) {
            return Err(CompilerError::CompileError(
                "spawn does not accept circuit keyword arguments".into(),
                self.cur_span(),
            ));
        }
        if matches!(callee.as_ref(), Expr::DotAccess { .. }) {
            return Err(CompilerError::CompilerLimitation(
                "spawn of method calls is not supported in Achronyme 0.1.0; wrap the call in a function"
                    .into(),
                self.cur_span(),
            ));
        }

        let callee_reg = self.compile_expr(callee)?;
        let arg_count = args.len();
        if arg_count > u8::MAX as usize {
            return Err(CompilerError::CompilerLimitation(
                format!("spawn call has {arg_count} arguments (maximum is 255)"),
                self.cur_span(),
            ));
        }
        for arg in args {
            self.compile_expr(&arg.value)?;
        }
        self.emit_abc(OpCode::Spawn, callee_reg, callee_reg, arg_count as u8)?;
        for _ in 0..arg_count {
            let top = self.current_ref()?.reg_top - 1;
            self.free_reg(top)?;
        }
        Ok(callee_reg)
    }

    pub(super) fn compile_await(
        &mut self,
        task: &Expr,
        mode: AwaitMode,
    ) -> Result<u8, CompilerError> {
        if mode == AwaitMode::Race {
            return self.compile_await_race(task);
        }
        let opcode = match mode {
            AwaitMode::Propagate => OpCode::Await,
            AwaitMode::Outcome => OpCode::AwaitOutcome,
            AwaitMode::Race => unreachable!("race handled above"),
        };
        if matches!(task, Expr::Call { .. }) {
            let implicit_scope = self.current_ref()?.concurrent_depth == 0;
            if implicit_scope {
                self.emit_abc(OpCode::ScopeEnter, 0, 0, 0)?;
                self.current()?.concurrent_depth = 1;
            }
            let task_reg = self.compile_spawn(task)?;
            self.emit_abc(opcode, task_reg, task_reg, 0)?;
            if implicit_scope {
                self.current()?.concurrent_depth = 0;
                self.emit_abc(OpCode::ScopeExit, 0, 0, 0)?;
            }
            return Ok(task_reg);
        }
        let task_reg = self.compile_expr(task)?;
        self.emit_abc(opcode, task_reg, task_reg, 0)?;
        Ok(task_reg)
    }

    fn compile_await_race(&mut self, task: &Expr) -> Result<u8, CompilerError> {
        let Expr::Array { elements, .. } = task else {
            return Err(CompilerError::CompileError(
                "`await ... as race` expects a list of task handles".into(),
                self.cur_span(),
            ));
        };
        if !(2..=u8::MAX as usize).contains(&elements.len()) {
            return Err(CompilerError::CompileError(
                "task race requires between 2 and 255 handles".into(),
                self.cur_span(),
            ));
        }
        let target = self.alloc_reg()?;
        let start = self.current_ref()?.reg_top;
        for (index, task) in elements.iter().enumerate() {
            let register = if matches!(task, Expr::Call { .. }) {
                self.compile_spawn(task)?
            } else {
                self.compile_expr(task)?
            };
            if register != start.wrapping_add(index as u8) {
                return Err(CompilerError::CompilerLimitation(
                    "register allocation fragmentation in task race".into(),
                    self.cur_span(),
                ));
            }
        }
        self.emit_abc(OpCode::AwaitRace, target, start, elements.len() as u8)?;
        for _ in elements {
            let top = self.current_ref()?.reg_top - 1;
            self.free_reg(top)?;
        }
        Ok(target)
    }

    pub(crate) fn emit_concurrent_scope_cleanup(
        &mut self,
        target_depth: u16,
    ) -> Result<(), CompilerError> {
        let current_depth = self.current_ref()?.concurrent_depth;
        if target_depth > current_depth {
            return Err(CompilerError::InternalError(
                "concurrent scope cleanup target exceeds current depth".into(),
            ));
        }
        for _ in target_depth..current_depth {
            self.emit_abc(OpCode::ScopeExit, 0, 0, 0)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use akron::opcode::instruction::decode_opcode;

    use super::*;

    fn opcodes(source: &str) -> Vec<OpCode> {
        let mut compiler = Compiler::new();
        compiler
            .compile(source)
            .unwrap()
            .into_iter()
            .map(|word| OpCode::from_u8(decode_opcode(word)).unwrap())
            .collect()
    }

    #[test]
    fn lowers_scope_spawn_and_await_in_source_order() {
        let ops = opcodes(
            "fn add_one(x) { x + 1 }\n\
             concurrent { let task = spawn add_one(1); await task }",
        );
        let enter = ops.iter().position(|op| *op == OpCode::ScopeEnter).unwrap();
        let spawn = ops.iter().position(|op| *op == OpCode::Spawn).unwrap();
        let await_ = ops.iter().position(|op| *op == OpCode::Await).unwrap();
        let exit = ops.iter().position(|op| *op == OpCode::ScopeExit).unwrap();
        assert!(enter < spawn && spawn < await_ && await_ < exit);
    }

    #[test]
    fn return_emits_scope_cleanup_before_return() {
        let mut compiler = Compiler::new();
        compiler
            .compile("fn early() { concurrent { return 7 } }")
            .unwrap();
        let function = compiler
            .prototypes
            .iter()
            .find(|function| function.name == "early")
            .unwrap();
        let ops: Vec<_> = function
            .chunk
            .iter()
            .map(|word| OpCode::from_u8(decode_opcode(*word)).unwrap())
            .collect();
        assert!(ops
            .windows(2)
            .any(|pair| { pair == [OpCode::ScopeExit, OpCode::Return] }));
    }

    #[test]
    fn awaited_call_gets_an_implicit_structured_scope() {
        let ops = opcodes("fn answer() { 42 }\nlet result = await answer()");
        let task_ops: Vec<_> = ops
            .into_iter()
            .filter(|op| {
                matches!(
                    op,
                    OpCode::ScopeEnter | OpCode::Spawn | OpCode::Await | OpCode::ScopeExit
                )
            })
            .collect();
        assert_eq!(
            task_ops,
            vec![
                OpCode::ScopeEnter,
                OpCode::Spawn,
                OpCode::Await,
                OpCode::ScopeExit
            ]
        );
    }

    #[test]
    fn recoverable_await_lowers_to_distinct_outcome_opcode() {
        let ops = opcodes(
            "fn fail() { assert(false) }\n\
             concurrent { let task = spawn fail(); await task as outcome }",
        );
        assert!(ops.contains(&OpCode::AwaitOutcome));
        assert!(!ops.contains(&OpCode::Await));
    }

    #[test]
    fn task_race_lowers_without_building_an_escaping_list() {
        let ops = opcodes(
            "fn value(x) { x }\n\
             concurrent { let a = spawn value(1); let b = spawn value(2); await [a, b] as race }",
        );
        assert!(ops.contains(&OpCode::AwaitRace));
        assert!(!ops.contains(&OpCode::BuildList));
    }

    #[test]
    fn cancel_check_lowers_to_the_dedicated_opcode() {
        let ops = opcodes("cancel_check()");
        assert!(ops.contains(&OpCode::CancelCheck));
        assert!(!ops.contains(&OpCode::Call));
    }

    #[test]
    fn aliases_of_cancel_check_keep_intrinsic_dispatch() {
        let mut compiler = Compiler::new();
        compiler
            .compile("fn invoke() { let check = cancel_check; check() }")
            .unwrap();
        let function = compiler
            .prototypes
            .iter()
            .find(|function| function.name == "invoke")
            .unwrap();
        let ops: Vec<_> = function
            .chunk
            .iter()
            .map(|word| OpCode::from_u8(decode_opcode(*word)).unwrap())
            .collect();
        assert!(ops.contains(&OpCode::CancelCheck), "ops={ops:?}");
        assert!(!ops.contains(&OpCode::Call), "ops={ops:?}");
    }

    #[test]
    fn local_shadowing_of_cancel_check_remains_a_normal_call() {
        let mut compiler = Compiler::new();
        compiler
            .compile("fn invoke(cancel_check) { cancel_check() }")
            .unwrap();
        let function = compiler
            .prototypes
            .iter()
            .find(|function| function.name == "invoke")
            .unwrap();
        let ops: Vec<_> = function
            .chunk
            .iter()
            .map(|word| OpCode::from_u8(decode_opcode(*word)).unwrap())
            .collect();
        assert!(ops.contains(&OpCode::Call));
        assert!(!ops.contains(&OpCode::CancelCheck));
    }

    #[test]
    fn cancel_check_rejects_arguments() {
        let error = Compiler::new().compile("cancel_check(1)").unwrap_err();
        assert!(error
            .to_string()
            .contains("cancel_check() takes no arguments"));
    }
}
