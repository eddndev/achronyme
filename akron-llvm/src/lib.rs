mod lower;

#[cfg(feature = "aot")]
mod aot;

#[cfg(feature = "llvm")]
mod jit;

#[cfg(feature = "aot")]
pub use aot::{AotArtifact, AotCompiler, AotError, AotOptions, ClangVersion};
pub use lower::{lower_program, LoweredModule, LoweringError};

#[cfg(feature = "llvm")]
pub use jit::{ExecutionOutcome, ExecutionResult, JitEngine, JitError, LlvmVersion};
