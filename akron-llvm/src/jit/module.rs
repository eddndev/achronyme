use std::ffi::{c_char, CStr, CString};
use std::ptr;

use super::ffi::{ContextRef, LlvmApi, ModuleRef, ThreadSafeModuleRef};
use super::{JitError, OPTIMIZATION_PIPELINE};

pub(super) struct ParsedModule {
    context: ContextRef,
    module: ModuleRef,
}

impl ParsedModule {
    pub(super) unsafe fn dispose(self, llvm: &LlvmApi) {
        unsafe {
            (llvm.dispose_module)(self.module);
            (llvm.context_dispose)(self.context);
        }
    }
}

pub(super) unsafe fn parse(llvm: &LlvmApi, ir: &str) -> Result<ParsedModule, JitError> {
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
        let detail = take_message(llvm, message);
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
    Ok(ParsedModule { context, module })
}

pub(super) unsafe fn optimize(llvm: &LlvmApi, parsed: &ParsedModule) -> Result<(), JitError> {
    unsafe { verify(llvm, parsed.module, "before optimization")? };

    let options = unsafe { (llvm.create_pass_builder_options)() };
    if options.is_null() {
        return Err(JitError::Llvm(
            "could not create LLVM pass builder options".to_string(),
        ));
    }
    let pipeline = CString::new(OPTIMIZATION_PIPELINE).expect("pipeline has no NUL");
    let pass_error =
        unsafe { (llvm.run_passes)(parsed.module, pipeline.as_ptr(), ptr::null_mut(), options) };
    unsafe { (llvm.dispose_pass_builder_options)(options) };
    if let Some(message) = unsafe { llvm.error_message(pass_error) } {
        return Err(JitError::Llvm(format!(
            "LLVM {OPTIMIZATION_PIPELINE} pipeline failed: {message}"
        )));
    }

    unsafe { verify(llvm, parsed.module, "after optimization") }
}

pub(super) unsafe fn into_thread_safe(
    llvm: &LlvmApi,
    parsed: ParsedModule,
) -> Result<ThreadSafeModuleRef, JitError> {
    let thread_context = unsafe { (llvm.create_thread_safe_context)(parsed.context) };
    if thread_context.is_null() {
        unsafe { parsed.dispose(llvm) };
        return Err(JitError::Llvm(
            "could not create LLVM ThreadSafeContext".to_string(),
        ));
    }
    let thread_module = unsafe { (llvm.create_thread_safe_module)(parsed.module, thread_context) };
    unsafe { (llvm.dispose_thread_safe_context)(thread_context) };
    if thread_module.is_null() {
        return Err(JitError::Llvm(
            "could not create LLVM ThreadSafeModule".to_string(),
        ));
    }
    Ok(thread_module)
}

unsafe fn verify(llvm: &LlvmApi, module: ModuleRef, stage: &str) -> Result<(), JitError> {
    let mut message: *mut c_char = ptr::null_mut();
    let failed = unsafe { (llvm.verify_module)(module, 2, &mut message) };
    if failed == 0 {
        if !message.is_null() {
            unsafe { (llvm.dispose_message)(message) };
        }
        return Ok(());
    }
    let detail = take_message(llvm, message);
    Err(JitError::Llvm(format!(
        "generated LLVM IR failed verification {stage}: {detail}"
    )))
}

fn take_message(llvm: &LlvmApi, message: *mut c_char) -> String {
    if message.is_null() {
        return "LLVM parser returned no diagnostics".to_string();
    }
    let detail = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    unsafe { (llvm.dispose_message)(message) };
    detail
}
