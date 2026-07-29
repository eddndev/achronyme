mod lower;
mod options;

#[cfg(feature = "aot")]
mod aot;

#[cfg(feature = "llvm")]
mod jit;

#[cfg(feature = "aot")]
pub use aot::{AotArtifact, AotCompiler, AotError, AotOptions, ClangVersion};
pub use lower::{lower_program, lower_program_with_options, LoweredModule, LoweringError};
pub use options::LlvmTierOptions;

#[cfg(feature = "llvm")]
pub use jit::{
    ExecutionOutcome, ExecutionResult, JitCacheConfig, JitCacheStats, JitCompileTimings, JitEngine,
    JitError, LlvmVersion,
};
