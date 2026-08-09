use crate::codegen::Compiler;
use crate::control_flow::ControlFlowCompiler;
use crate::error::CompilerError;
use crate::function_compiler::FunctionCompiler;
use crate::types::Local;
use achronyme_parser::ast::*;
use achronyme_parser::Diagnostic;
use akron::opcode::OpCode;
use memory::Function;

pub trait FunctionDefinitionCompiler {
    fn compile_fn_core(
        &mut self,
        name: Option<&str>,
        params: &[TypedParam],
        body: &Block,
    ) -> Result<u8, CompilerError>;
}

impl FunctionDefinitionCompiler for Compiler {
    fn compile_fn_core(
        &mut self,
        name: Option<&str>,
        params: &[TypedParam],
        body: &Block,
    ) -> Result<u8, CompilerError> {
        let fn_name = name.unwrap_or("lambda").to_string();
        if params.len() > 255 {
            return Err(CompilerError::CompilerLimitation(
                format!(
                    "function `{fn_name}` has {} parameters (maximum is 255)",
                    params.len()
                ),
                self.cur_span(),
            ));
        }
        let arity = params.len() as u8;

        let global_idx = if name.is_some() {
            if self.next_global_idx == u16::MAX {
                return Err(CompilerError::TooManyConstants(self.cur_span()));
            }
            let idx = self.next_global_idx;
            self.next_global_idx += 1;
            let global_name = match &self.module_prefix {
                Some(prefix) => format!("{prefix}::{fn_name}"),
                None => fn_name.clone(),
            };
            self.global_symbols.insert(
                global_name,
                crate::types::GlobalEntry {
                    index: idx,
                    type_ann: None,
                    is_mutable: false,
                    param_names: None,
                },
            );
            Some(idx)
        } else {
            None
        };

        // Use the current span (pointing to the fn declaration) for param locals
        let fn_span = self.current_span.clone();

        // Preserve the resolver module that owns this function body. The active
        // source path covers namespace and selective imports; the dispatch key
        // remains the fallback for callers without path context. Lambdas and
        // other nested functions inherit the module of their enclosing body.
        let resolver_module = self
            .resolver_current_module
            .or_else(|| {
                name.and_then(|declared_name| {
                    let fn_key = match &self.module_prefix {
                        Some(prefix) => format!("{prefix}::{declared_name}"),
                        None => declared_name.to_string(),
                    };
                    self.resolver_module_by_key
                        .as_ref()
                        .and_then(|modules| modules.get(&fn_key).copied())
                })
            })
            .or_else(|| {
                self.current_ref()
                    .ok()
                    .and_then(|func| func.resolver_module)
            })
            .or(self.resolver_root_module);

        let mut function_compiler = FunctionCompiler::new(fn_name.clone(), arity);
        function_compiler.resolver_module = resolver_module;
        self.compilers.push(function_compiler);

        for (i, param) in params.iter().enumerate() {
            self.current()?.locals.push(Local {
                name: param.name.clone(),
                depth: 0,
                is_captured: false,
                is_mutable: false,
                is_read: false,
                is_mutated: false,
                reg: i as u8,
                span: fn_span.clone(),
                type_ann: param.type_ann.clone(),
            });
        }

        let body_reg = self.alloc_reg()?;
        self.compile_block(body, body_reg)?;

        self.emit_abc(OpCode::Return, body_reg, 1, 0)?;

        // Check for unused function parameters before popping the compiler
        let param_warns = {
            let func = self.current()?;
            let mut warns = Vec::new();
            for local in &func.locals {
                if local.depth == 0 && !local.name.starts_with('_') && !local.is_read {
                    if let Some(ref span) = local.span {
                        warns.push(
                            Diagnostic::warning(
                                format!("unused function parameter: `{}`", local.name),
                                span.into(),
                            )
                            .with_code("W001")
                            .with_note(format!(
                                "if this is intentional, prefix with underscore: `_{}`",
                                local.name
                            )),
                        );
                    }
                }
            }
            warns
        };
        self.warnings.extend(param_warns);

        let mut compiled_func = self
            .compilers
            .pop()
            .ok_or_else(|| CompilerError::InternalError("compiler stack underflow".into()))?;
        compiled_func.max_slots = compiled_func.max_slots.max(compiled_func.reg_top as u16);

        let (opt_bytecode, opt_line_info) = crate::optimizer::optimize(
            compiled_func.bytecode,
            compiled_func.line_info,
            &mut compiled_func.max_slots,
        );

        let func = Function {
            name: compiled_func.name,
            arity: compiled_func.arity,
            max_slots: compiled_func.max_slots,
            chunk: opt_bytecode,
            constants: compiled_func.constants,
            upvalue_info: compiled_func
                .upvalues
                .iter()
                .flat_map(|u| vec![u.is_local as u8, u.index])
                .collect(),
            line_info: opt_line_info,
        };

        let global_func_idx = self.prototypes.len();
        self.prototypes.push(func);

        let target_reg = self.alloc_reg()?;
        self.emit_abx(OpCode::Closure, target_reg, global_func_idx as u16)?;

        if let Some(idx) = global_idx {
            self.emit_abx(OpCode::DefGlobalLet, target_reg, idx)?;
        }

        Ok(target_reg)
    }
}
