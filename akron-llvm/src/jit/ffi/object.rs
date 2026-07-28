use std::ffi::{c_char, c_void, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::time::{Duration, Instant};

use super::{
    ErrorRef, GetBufferSizeFn, GetBufferStartFn, JitDylibRef, LlJitRef, LlvmApi, MemoryBufferRef,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TargetIdentity {
    pub triple: String,
    pub cpu: String,
    pub features: String,
}

pub(crate) struct ObjectCapture {
    get_buffer_start: GetBufferStartFn,
    get_buffer_size: GetBufferSizeFn,
    object: Option<Vec<u8>>,
    elapsed: Duration,
    failed: bool,
}

impl ObjectCapture {
    pub(crate) fn new(llvm: &LlvmApi) -> Self {
        Self {
            get_buffer_start: llvm.get_buffer_start,
            get_buffer_size: llvm.get_buffer_size,
            object: None,
            elapsed: Duration::ZERO,
            failed: false,
        }
    }

    pub(crate) fn into_object(self) -> Option<(Vec<u8>, Duration)> {
        (!self.failed).then_some((self.object?, self.elapsed))
    }

    fn capture(&mut self, object: MemoryBufferRef) -> Result<(), ()> {
        if object.is_null() {
            return Err(());
        }
        let start = unsafe { (self.get_buffer_start)(object) };
        let size = unsafe { (self.get_buffer_size)(object) };
        if start.is_null() || size == 0 {
            return Err(());
        }
        let started = Instant::now();
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).map_err(|_| ())?;
        unsafe {
            bytes.set_len(size);
            ptr::copy_nonoverlapping(start.cast::<u8>(), bytes.as_mut_ptr(), size);
        }
        self.elapsed += started.elapsed();
        self.object = Some(bytes);
        Ok(())
    }
}

unsafe extern "C" fn capture_object(
    context: *mut c_void,
    object_in_out: *mut MemoryBufferRef,
) -> ErrorRef {
    if context.is_null() {
        return ptr::null_mut();
    }
    let capture = unsafe { &mut *context.cast::<ObjectCapture>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        let object = unsafe { object_in_out.as_ref().copied() }.ok_or(())?;
        capture.capture(object)
    }));
    if !matches!(result, Ok(Ok(()))) {
        capture.failed = true;
    }
    ptr::null_mut()
}

unsafe extern "C" fn pass_object_through(
    _context: *mut c_void,
    _object_in_out: *mut MemoryBufferRef,
) -> ErrorRef {
    ptr::null_mut()
}

impl LlvmApi {
    pub unsafe fn target_identity(
        &self,
        jit: LlJitRef,
    ) -> Result<TargetIdentity, super::super::JitError> {
        let triple = unsafe { (self.get_jit_triple)(jit) };
        if triple.is_null() {
            return Err(super::super::JitError::Llvm(
                "LLJIT returned a null target triple".to_string(),
            ));
        }
        let triple = unsafe { CStr::from_ptr(triple) }
            .to_string_lossy()
            .into_owned();
        let cpu = unsafe { self.take_owned_string((self.get_host_cpu_name)(), "host CPU")? };
        let features =
            unsafe { self.take_owned_string((self.get_host_cpu_features)(), "host CPU features")? };
        Ok(TargetIdentity {
            triple,
            cpu,
            features,
        })
    }

    pub unsafe fn install_object_capture(
        &self,
        jit: LlJitRef,
        capture: &mut ObjectCapture,
    ) -> Result<(), super::super::JitError> {
        let layer = unsafe { (self.get_object_transform_layer)(jit) };
        if layer.is_null() {
            return Err(super::super::JitError::Llvm(
                "LLJIT returned a null object transform layer".to_string(),
            ));
        }
        unsafe {
            (self.set_object_transform)(
                layer,
                capture_object,
                ptr::from_mut(capture).cast::<c_void>(),
            )
        };
        Ok(())
    }

    pub unsafe fn reset_object_transform(&self, jit: LlJitRef) {
        let layer = unsafe { (self.get_object_transform_layer)(jit) };
        if !layer.is_null() {
            unsafe { (self.set_object_transform)(layer, pass_object_through, ptr::null_mut()) };
        }
    }

    pub unsafe fn add_object(
        &self,
        jit: LlJitRef,
        dylib: JitDylibRef,
        object: &[u8],
    ) -> Result<(), super::super::JitError> {
        let name = c"akron-jit-object";
        let buffer = unsafe {
            (self.create_memory_buffer)(object.as_ptr().cast(), object.len(), name.as_ptr())
        };
        if buffer.is_null() {
            return Err(super::super::JitError::Llvm(
                "could not create a JIT object buffer".to_string(),
            ));
        }
        let error = unsafe { (self.add_object_file)(jit, dylib, buffer) };
        if let Some(message) = unsafe { self.error_message(error) } {
            return Err(super::super::JitError::Llvm(format!(
                "could not add object to LLJIT: {message}"
            )));
        }
        Ok(())
    }

    unsafe fn take_owned_string(
        &self,
        value: *mut c_char,
        description: &str,
    ) -> Result<String, super::super::JitError> {
        if value.is_null() {
            return Err(super::super::JitError::Llvm(format!(
                "LLVM returned a null {description}"
            )));
        }
        let owned = unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned();
        unsafe { (self.dispose_message)(value) };
        Ok(owned)
    }
}
