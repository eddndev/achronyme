mod ffi;
mod module;

use std::ffi::CString;
use std::fmt;
use std::ptr;
use std::time::{Duration, Instant};

use akron::compiled::{
    runtime_api, CompiledEntry, ExecutionStats, RuntimeCapabilities, RuntimeContext,
    RUNTIME_ABI_VERSION,
};
use akron::{CompiledProgram, RuntimeError, VM};
use memory::Value;

use crate::{lower_program, LoweringError};

use self::ffi::{LlJitRef, LlvmApi};

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
    pub lljit_creation: Duration,
    pub ir_parse: Duration,
    pub llvm_optimization: Duration,
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
    pub fn compile(program: &CompiledProgram) -> Result<Self, JitError> {
        let mut compile_timings = JitCompileTimings::default();
        let started = Instant::now();
        let lowered = lower_program(program)?;
        compile_timings.lowering = started.elapsed();
        let instruction_count = lowered.instruction_count;
        let native_instruction_count = lowered.native_instruction_count;
        let direct_instruction_count = lowered.direct_instruction_count;
        let runtime_instruction_count = lowered.runtime_instruction_count;
        let compiled_call_count = lowered.compiled_call_count;
        let started = Instant::now();
        let llvm = LlvmApi::load()?;
        llvm.initialize_native_target();
        compile_timings.llvm_setup = started.elapsed();

        let started = Instant::now();
        let mut jit = ptr::null_mut();
        let create_error = unsafe { (llvm.create_lljit)(&mut jit, ptr::null_mut()) };
        if let Some(message) = unsafe { llvm.error_message(create_error) } {
            return Err(JitError::Llvm(format!("could not create LLJIT: {message}")));
        }
        if jit.is_null() {
            return Err(JitError::Llvm(
                "LLVM reported success but returned a null LLJIT".to_string(),
            ));
        }
        compile_timings.lljit_creation = started.elapsed();

        let started = Instant::now();
        let parsed = match unsafe { module::parse(&llvm, &lowered.ir) } {
            Ok(parsed) => parsed,
            Err(error) => {
                unsafe { llvm.dispose_jit_ignoring_error(jit) };
                return Err(error);
            }
        };
        compile_timings.ir_parse = started.elapsed();

        let started = Instant::now();
        if let Err(error) = unsafe { module::optimize(&llvm, &parsed) } {
            unsafe {
                parsed.dispose(&llvm);
                llvm.dispose_jit_ignoring_error(jit);
            }
            return Err(error);
        }
        compile_timings.llvm_optimization = started.elapsed();

        let started = Instant::now();
        let module = match unsafe { module::into_thread_safe(&llvm, parsed) } {
            Ok(module) => module,
            Err(error) => {
                unsafe { llvm.dispose_jit_ignoring_error(jit) };
                return Err(error);
            }
        };
        compile_timings.thread_safe_module = started.elapsed();

        let started = Instant::now();
        let dylib = unsafe { (llvm.get_main_jit_dylib)(jit) };
        let add_error = unsafe { (llvm.add_ir_module)(jit, dylib, module) };
        if let Some(message) = unsafe { llvm.error_message(add_error) } {
            unsafe { llvm.dispose_jit_ignoring_error(jit) };
            return Err(JitError::Llvm(format!(
                "could not add LLVM module to LLJIT: {message}"
            )));
        }
        compile_timings.module_add = started.elapsed();

        let started = Instant::now();
        let symbol = CString::new(lowered.entry_symbol).expect("entry symbol has no NUL");
        let mut address = 0u64;
        let lookup_error = unsafe { (llvm.lookup)(jit, &mut address, symbol.as_ptr()) };
        if let Some(message) = unsafe { llvm.error_message(lookup_error) } {
            unsafe { llvm.dispose_jit_ignoring_error(jit) };
            return Err(JitError::Llvm(format!(
                "could not resolve JIT entry: {message}"
            )));
        }
        if address == 0 {
            unsafe { llvm.dispose_jit_ignoring_error(jit) };
            return Err(JitError::Llvm(
                "LLVM resolved the JIT entry to a null address".to_string(),
            ));
        }
        compile_timings.lookup_materialization = started.elapsed();

        let entry = unsafe { std::mem::transmute::<usize, CompiledEntry>(address as usize) };
        Ok(Self {
            llvm,
            jit,
            entry,
            instruction_count,
            native_instruction_count,
            direct_instruction_count,
            runtime_instruction_count,
            compiled_call_count,
            compile_timings,
            cache_stats: JitCacheStats::default(),
        })
    }

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
