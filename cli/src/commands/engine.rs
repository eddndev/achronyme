use akron::{CompiledProgram, RuntimeError, VM};
use anyhow::{anyhow, Result};
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ExecutionEngine {
    #[default]
    Interpreter,
    Jit,
    Auto,
}

pub(crate) enum PreparedEngine {
    Interpreter,
    #[cfg(feature = "llvm")]
    Jit(Box<akron_llvm::JitEngine>),
}

pub(crate) enum ExecutionError {
    Runtime(RuntimeError),
    #[cfg(feature = "llvm")]
    Engine(anyhow::Error),
}

impl PreparedEngine {
    pub(crate) fn prepare(selection: ExecutionEngine, program: &CompiledProgram) -> Result<Self> {
        match selection {
            ExecutionEngine::Interpreter => Ok(Self::Interpreter),
            ExecutionEngine::Jit => prepare_jit(program),
            ExecutionEngine::Auto => match prepare_jit(program) {
                Ok(engine) => Ok(engine),
                Err(error) => {
                    eprintln!("LLVM JIT fallback: {error}");
                    Ok(Self::Interpreter)
                }
            },
        }
    }

    pub(crate) fn execute(self, vm: &mut VM) -> Result<(), ExecutionError> {
        match self {
            Self::Interpreter => vm.interpret().map_err(ExecutionError::Runtime),
            #[cfg(feature = "llvm")]
            Self::Jit(engine) => match engine.execute(vm) {
                Ok(result) => {
                    if std::env::var_os("AKRON_ENGINE_TRACE").is_some() {
                        match result.outcome {
                            akron_llvm::ExecutionOutcome::Native => eprintln!(
                                "LLVM JIT native: {}/{} bytecode instructions lowered",
                                engine.native_instruction_count(),
                                engine.instruction_count()
                            ),
                            akron_llvm::ExecutionOutcome::InterpreterBailout { instruction } => {
                                eprintln!(
                                    "LLVM JIT bailout: resumed interpreter at bytecode instruction {instruction} ({}/{} instructions directly lowered)",
                                    engine.native_instruction_count(),
                                    engine.instruction_count()
                                );
                            }
                        }
                    }
                    Ok(())
                }
                Err(akron_llvm::JitError::Runtime(error)) => Err(ExecutionError::Runtime(error)),
                Err(error) => Err(ExecutionError::Engine(anyhow!(error))),
            },
        }
    }
}

#[cfg(feature = "llvm")]
fn prepare_jit(program: &CompiledProgram) -> Result<PreparedEngine> {
    match akron_llvm::JitEngine::compile(program) {
        Ok(engine) => Ok(PreparedEngine::Jit(Box::new(engine))),
        Err(akron_llvm::JitError::Lowering(error)) => {
            Err(anyhow!("LLVM JIT does not support this program: {error}"))
        }
        Err(error) => Err(anyhow!("LLVM JIT initialization failed: {error}")),
    }
}

#[cfg(not(feature = "llvm"))]
fn prepare_jit(_program: &CompiledProgram) -> Result<PreparedEngine> {
    Err(anyhow!(
        "this ach build does not include LLVM; rebuild with `--features llvm`"
    ))
}
