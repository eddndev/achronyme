mod cache;
mod compile;
mod ffi;
mod module;

use std::fmt;
use std::ptr;
use std::time::Duration;

use akron::compiled::{
    runtime_api, CompiledEntry, ExecutionStats, RuntimeCapabilities, RuntimeContext,
    RUNTIME_ABI_VERSION,
};
use akron::{RuntimeError, VM};
use memory::Value;

use crate::LoweringError;

use self::ffi::{LlJitRef, LlvmApi};

pub use cache::JitCacheConfig;

pub(super) const OPTIMIZATION_PIPELINE: &str = "default<O2>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlvmVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub struct JitEngine {
    llvm: LlvmApi,
    jit: LlJitRef,
    entry: CompiledEntry,
    instruction_count: usize,
    native_instruction_count: usize,
    direct_instruction_count: usize,
    runtime_instruction_count: usize,
    compiled_call_count: usize,
    compile_timings: JitCompileTimings,
    cache_stats: JitCacheStats,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JitCompileTimings {
    pub lowering: Duration,
    pub llvm_setup: Duration,
    pub llvm_library_open: Duration,
    pub llvm_symbol_bind: Duration,
    pub llvm_target_init: Duration,
    pub lljit_creation: Duration,
    pub cache_key: Duration,
    pub cache_lookup: Duration,
    pub ir_parse: Duration,
    pub llvm_optimization: Duration,
    pub object_capture: Duration,
    pub cache_store: Duration,
    pub thread_safe_module: Duration,
    pub module_add: Duration,
    pub lookup_materialization: Duration,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JitCacheStats {
    pub lookups: u64,
    pub hits: u64,
    pub misses: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionOutcome {
    Native,
    InterpreterBailout { instruction: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExecutionResult {
    pub value: Value,
    pub outcome: ExecutionOutcome,
    pub stats: ExecutionStats,
}

impl JitEngine {
    pub fn llvm_version(&self) -> LlvmVersion {
        self.llvm.version
    }

    pub fn llvm_library_path(&self) -> &str {
        &self.llvm.path
    }

    pub fn optimization_pipeline(&self) -> &'static str {
        OPTIMIZATION_PIPELINE
    }

    pub fn instruction_count(&self) -> usize {
        self.instruction_count
    }

    pub fn native_instruction_count(&self) -> usize {
        self.native_instruction_count
    }

    pub fn direct_instruction_count(&self) -> usize {
        self.direct_instruction_count
    }

    pub fn runtime_instruction_count(&self) -> usize {
        self.runtime_instruction_count
    }

    pub fn compiled_call_count(&self) -> usize {
        self.compiled_call_count
    }

    pub fn compile_timings(&self) -> JitCompileTimings {
        self.compile_timings
    }

    pub fn cache_stats(&self) -> JitCacheStats {
        self.cache_stats
    }

    pub fn execute(&self, vm: &mut VM) -> Result<ExecutionResult, JitError> {
        runtime_api()
            .validate(
                RUNTIME_ABI_VERSION,
                std::mem::size_of::<akron::compiled::RuntimeApi>() as u32,
                RuntimeCapabilities::LLVM_TIER2,
            )
            .map_err(|error| JitError::Abi(error.to_string()))?;
        if vm.frames.len() != 1 {
            return Err(JitError::State(format!(
                "scalar JIT requires exactly one main frame, found {}",
                vm.frames.len()
            )));
        }
        let base = u32::try_from(vm.frames[0].base)
            .map_err(|_| JitError::State("frame base exceeds u32".to_string()))?;
        let mut result_bits = Value::nil().to_abi_bits();
        let mut context = RuntimeContext::new(vm);
        let status = unsafe {
            (self.entry)(
                runtime_api(),
                context.as_opaque(),
                0,
                base,
                &mut result_bits,
            )
        };
        let bailout_ip = context.bailout_ip();
        let stats = context.stats();
        context.finish(status).map_err(JitError::Runtime)?;

        let value = Value::from_abi_bits(result_bits)
            .ok_or_else(|| JitError::State("JIT returned noncanonical Value bits".to_string()))?;
        if bailout_ip.is_none() {
            vm.last_result = value;
        }
        vm.frames.pop();
        let outcome = match bailout_ip {
            Some(instruction) => ExecutionOutcome::InterpreterBailout { instruction },
            None => ExecutionOutcome::Native,
        };
        Ok(ExecutionResult {
            value,
            outcome,
            stats,
        })
    }
}

impl Drop for JitEngine {
    fn drop(&mut self) {
        unsafe { self.llvm.dispose_jit_ignoring_error(self.jit) };
        self.jit = ptr::null_mut();
    }
}

#[derive(Debug)]
pub enum JitError {
    Lowering(LoweringError),
    Library(String),
    Llvm(String),
    Abi(String),
    State(String),
    Runtime(RuntimeError),
}

impl fmt::Display for JitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lowering(error) => write!(formatter, "{error}"),
            Self::Library(message) => write!(formatter, "LLVM library error: {message}"),
            Self::Llvm(message) => write!(formatter, "LLVM JIT error: {message}"),
            Self::Abi(message) => write!(formatter, "runtime ABI error: {message}"),
            Self::State(message) => write!(formatter, "JIT state error: {message}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for JitError {}

impl From<LoweringError> for JitError {
    fn from(error: LoweringError) -> Self {
        Self::Lowering(error)
    }
}

#[cfg(test)]
mod tests;
