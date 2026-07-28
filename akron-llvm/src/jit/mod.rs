mod ffi;

use std::ffi::{c_char, CStr, CString};
use std::fmt;
use std::ptr;

use akron::compiled::{
    runtime_api, CompiledEntry, RuntimeCapabilities, RuntimeContext, RUNTIME_ABI_VERSION,
};
use akron::{CompiledProgram, RuntimeError, VM};
use memory::Value;

use crate::{lower_program, LoweringError};

use self::ffi::{LlJitRef, LlvmApi, ModuleRef, ThreadSafeModuleRef};

const OPTIMIZATION_PIPELINE: &str = "default<O2>";

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
}

impl JitEngine {
    pub fn compile(program: &CompiledProgram) -> Result<Self, JitError> {
        let lowered = lower_program(program)?;
        let instruction_count = lowered.instruction_count;
        let native_instruction_count = lowered.native_instruction_count;
        let llvm = LlvmApi::load()?;
        llvm.initialize_native_target();

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

        let module = match unsafe { parse_module(&llvm, &lowered.ir) } {
            Ok(module) => module,
            Err(error) => {
                unsafe { llvm.dispose_jit_ignoring_error(jit) };
                return Err(error);
            }
        };
        let dylib = unsafe { (llvm.get_main_jit_dylib)(jit) };
        let add_error = unsafe { (llvm.add_ir_module)(jit, dylib, module) };
        if let Some(message) = unsafe { llvm.error_message(add_error) } {
            unsafe { llvm.dispose_jit_ignoring_error(jit) };
            return Err(JitError::Llvm(format!(
                "could not add LLVM module to LLJIT: {message}"
            )));
        }

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

        let entry = unsafe { std::mem::transmute::<usize, CompiledEntry>(address as usize) };
        Ok(Self {
            llvm,
            jit,
            entry,
            instruction_count,
            native_instruction_count,
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

    pub fn execute(&self, vm: &mut VM) -> Result<ExecutionResult, JitError> {
        runtime_api()
            .validate(
                RUNTIME_ABI_VERSION,
                std::mem::size_of::<akron::compiled::RuntimeApi>() as u32,
                RuntimeCapabilities::LLVM_TIER1,
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
        Ok(ExecutionResult { value, outcome })
    }
}

impl Drop for JitEngine {
    fn drop(&mut self) {
        unsafe { self.llvm.dispose_jit_ignoring_error(self.jit) };
        self.jit = ptr::null_mut();
    }
}

unsafe fn parse_module(llvm: &LlvmApi, ir: &str) -> Result<ThreadSafeModuleRef, JitError> {
    let context = unsafe { (llvm.context_create)() };
    if context.is_null() {
        return Err(JitError::Llvm("could not create LLVM context".to_string()));
    }

    let name = c"akron-jit";
    let buffer = unsafe {
        (llvm.create_memory_buffer)(ir.as_ptr().cast::<c_char>(), ir.len(), name.as_ptr())
    };
    let mut module: ModuleRef = ptr::null_mut();
    let mut message: *mut c_char = ptr::null_mut();
    let failed = unsafe { (llvm.parse_ir)(context, buffer, &mut module, &mut message) };
    if failed != 0 {
        let detail = take_parse_message(llvm, message);
        unsafe { (llvm.context_dispose)(context) };
        return Err(JitError::Llvm(format!(
            "invalid generated LLVM IR: {detail}"
        )));
    }
    if module.is_null() {
        unsafe { (llvm.context_dispose)(context) };
        return Err(JitError::Llvm(
            "LLVM parser returned a null module".to_string(),
        ));
    }

    if let Err(error) = unsafe { verify_and_optimize(llvm, module) } {
        unsafe {
            (llvm.dispose_module)(module);
            (llvm.context_dispose)(context);
        }
        return Err(error);
    }

    let thread_context = unsafe { (llvm.create_thread_safe_context)(context) };
    if thread_context.is_null() {
        unsafe {
            (llvm.dispose_module)(module);
            (llvm.context_dispose)(context);
        }
        return Err(JitError::Llvm(
            "could not create LLVM ThreadSafeContext".to_string(),
        ));
    }
    let thread_module = unsafe { (llvm.create_thread_safe_module)(module, thread_context) };
    unsafe { (llvm.dispose_thread_safe_context)(thread_context) };
    if thread_module.is_null() {
        return Err(JitError::Llvm(
            "could not create LLVM ThreadSafeModule".to_string(),
        ));
    }
    Ok(thread_module)
}

unsafe fn verify_and_optimize(llvm: &LlvmApi, module: ModuleRef) -> Result<(), JitError> {
    unsafe { verify_module(llvm, module, "before optimization")? };

    let options = unsafe { (llvm.create_pass_builder_options)() };
    if options.is_null() {
        return Err(JitError::Llvm(
            "could not create LLVM pass builder options".to_string(),
        ));
    }
    let pipeline = CString::new(OPTIMIZATION_PIPELINE).expect("pipeline has no NUL");
    let pass_error =
        unsafe { (llvm.run_passes)(module, pipeline.as_ptr(), ptr::null_mut(), options) };
    unsafe { (llvm.dispose_pass_builder_options)(options) };
    if let Some(message) = unsafe { llvm.error_message(pass_error) } {
        return Err(JitError::Llvm(format!(
            "LLVM {OPTIMIZATION_PIPELINE} pipeline failed: {message}"
        )));
    }

    unsafe { verify_module(llvm, module, "after optimization") }
}

unsafe fn verify_module(llvm: &LlvmApi, module: ModuleRef, stage: &str) -> Result<(), JitError> {
    let mut message: *mut c_char = ptr::null_mut();
    let failed = unsafe { (llvm.verify_module)(module, 2, &mut message) };
    if failed == 0 {
        if !message.is_null() {
            unsafe { (llvm.dispose_message)(message) };
        }
        return Ok(());
    }
    let detail = take_parse_message(llvm, message);
    Err(JitError::Llvm(format!(
        "generated LLVM IR failed verification {stage}: {detail}"
    )))
}

fn take_parse_message(llvm: &LlvmApi, message: *mut c_char) -> String {
    if message.is_null() {
        return "LLVM parser returned no diagnostics".to_string();
    }
    let detail = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    unsafe { (llvm.dispose_message)(message) };
    detail
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
