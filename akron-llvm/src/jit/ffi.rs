mod object;

use std::env;
use std::ffi::{c_char, c_void, CStr, OsString};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use libloading::Library;

pub(super) use object::{ObjectCapture, TargetIdentity};

pub(super) type ContextRef = *mut c_void;
pub(super) type ModuleRef = *mut c_void;
pub(super) type MemoryBufferRef = *mut c_void;
pub(super) type ErrorRef = *mut c_void;
pub(super) type ThreadSafeContextRef = *mut c_void;
pub(super) type ThreadSafeModuleRef = *mut c_void;
pub(super) type LlJitRef = *mut c_void;
pub(super) type JitDylibRef = *mut c_void;
pub(super) type PassBuilderOptionsRef = *mut c_void;
pub(super) type ObjectTransformLayerRef = *mut c_void;

type GetVersionFn = unsafe extern "C" fn(*mut u32, *mut u32, *mut u32);
type VoidFn = unsafe extern "C" fn();
type ContextCreateFn = unsafe extern "C" fn() -> ContextRef;
type ContextDisposeFn = unsafe extern "C" fn(ContextRef);
type DisposeModuleFn = unsafe extern "C" fn(ModuleRef);
type CreateMemoryBufferFn =
    unsafe extern "C" fn(*const c_char, usize, *const c_char) -> MemoryBufferRef;
type ParseIrFn =
    unsafe extern "C" fn(ContextRef, MemoryBufferRef, *mut ModuleRef, *mut *mut c_char) -> i32;
type DisposeMessageFn = unsafe extern "C" fn(*mut c_char);
type CreateThreadSafeContextFn = unsafe extern "C" fn(ContextRef) -> ThreadSafeContextRef;
type DisposeThreadSafeContextFn = unsafe extern "C" fn(ThreadSafeContextRef);
type CreateThreadSafeModuleFn =
    unsafe extern "C" fn(ModuleRef, ThreadSafeContextRef) -> ThreadSafeModuleRef;
type CreateLlJitFn = unsafe extern "C" fn(*mut LlJitRef, *mut c_void) -> ErrorRef;
type DisposeLlJitFn = unsafe extern "C" fn(LlJitRef) -> ErrorRef;
type GetMainJitDylibFn = unsafe extern "C" fn(LlJitRef) -> JitDylibRef;
type AddIrModuleFn = unsafe extern "C" fn(LlJitRef, JitDylibRef, ThreadSafeModuleRef) -> ErrorRef;
type AddObjectFileFn = unsafe extern "C" fn(LlJitRef, JitDylibRef, MemoryBufferRef) -> ErrorRef;
type LookupFn = unsafe extern "C" fn(LlJitRef, *mut u64, *const c_char) -> ErrorRef;
type GetJitTripleFn = unsafe extern "C" fn(LlJitRef) -> *const c_char;
type GetHostStringFn = unsafe extern "C" fn() -> *mut c_char;
type GetBufferStartFn = unsafe extern "C" fn(MemoryBufferRef) -> *const c_char;
type GetBufferSizeFn = unsafe extern "C" fn(MemoryBufferRef) -> usize;
type ObjectTransformFn = unsafe extern "C" fn(*mut c_void, *mut MemoryBufferRef) -> ErrorRef;
type GetObjectTransformLayerFn = unsafe extern "C" fn(LlJitRef) -> ObjectTransformLayerRef;
type SetObjectTransformFn =
    unsafe extern "C" fn(ObjectTransformLayerRef, ObjectTransformFn, *mut c_void);
type GetErrorMessageFn = unsafe extern "C" fn(ErrorRef) -> *mut c_char;
type DisposeErrorMessageFn = unsafe extern "C" fn(*mut c_char);
type VerifyModuleFn = unsafe extern "C" fn(ModuleRef, i32, *mut *mut c_char) -> i32;
type RunPassesFn =
    unsafe extern "C" fn(ModuleRef, *const c_char, *mut c_void, PassBuilderOptionsRef) -> ErrorRef;
type CreatePassBuilderOptionsFn = unsafe extern "C" fn() -> PassBuilderOptionsRef;
type DisposePassBuilderOptionsFn = unsafe extern "C" fn(PassBuilderOptionsRef);

pub(super) struct LlvmApi {
    _library: Arc<Library>,
    pub path: String,
    pub version: super::LlvmVersion,
    pub context_create: ContextCreateFn,
    pub context_dispose: ContextDisposeFn,
    pub dispose_module: DisposeModuleFn,
    pub create_memory_buffer: CreateMemoryBufferFn,
    pub parse_ir: ParseIrFn,
    pub dispose_message: DisposeMessageFn,
    pub create_thread_safe_context: CreateThreadSafeContextFn,
    pub dispose_thread_safe_context: DisposeThreadSafeContextFn,
    pub create_thread_safe_module: CreateThreadSafeModuleFn,
    pub create_lljit: CreateLlJitFn,
    pub dispose_lljit: DisposeLlJitFn,
    pub get_main_jit_dylib: GetMainJitDylibFn,
    pub add_ir_module: AddIrModuleFn,
    pub add_object_file: AddObjectFileFn,
    pub lookup: LookupFn,
    get_jit_triple: GetJitTripleFn,
    get_host_cpu_name: GetHostStringFn,
    get_host_cpu_features: GetHostStringFn,
    get_buffer_start: GetBufferStartFn,
    get_buffer_size: GetBufferSizeFn,
    get_object_transform_layer: GetObjectTransformLayerFn,
    set_object_transform: SetObjectTransformFn,
    pub get_error_message: GetErrorMessageFn,
    pub dispose_error_message: DisposeErrorMessageFn,
    pub verify_module: VerifyModuleFn,
    pub run_passes: RunPassesFn,
    pub create_pass_builder_options: CreatePassBuilderOptionsFn,
    pub dispose_pass_builder_options: DisposePassBuilderOptionsFn,
    initialize_target_info: VoidFn,
    initialize_target: VoidFn,
    initialize_target_mc: VoidFn,
    initialize_asm_printer: VoidFn,
}

struct SharedLibrary {
    library: Arc<Library>,
    path: String,
    version: super::LlvmVersion,
}

static LLVM_LIBRARY: OnceLock<Result<SharedLibrary, String>> = OnceLock::new();
static NATIVE_TARGET_INITIALIZED: OnceLock<()> = OnceLock::new();

impl LlvmApi {
    pub fn load_with_timings() -> Result<(Self, Duration, Duration), super::JitError> {
        let started = Instant::now();
        let shared = LLVM_LIBRARY
            .get_or_init(load_library)
            .as_ref()
            .map_err(|error| super::JitError::Library(error.clone()))?;
        let library_open = started.elapsed();
        let started = Instant::now();
        let api = unsafe { Self::load_symbols(shared) }.map_err(super::JitError::Library)?;
        Ok((api, library_open, started.elapsed()))
    }

    unsafe fn load_symbols(shared: &SharedLibrary) -> Result<Self, String> {
        let library = &shared.library;
        #[cfg(target_arch = "x86_64")]
        let target_prefix = "X86";
        #[cfg(target_arch = "aarch64")]
        let target_prefix = "AArch64";
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return Err("only x86_64 and aarch64 hosts are currently supported".to_string());

        let target_symbol = |suffix: &str| format!("LLVMInitialize{target_prefix}{suffix}\0");
        Ok(Self {
            path: shared.path.clone(),
            version: shared.version,
            context_create: unsafe { symbol(library, b"LLVMContextCreate\0")? },
            context_dispose: unsafe { symbol(library, b"LLVMContextDispose\0")? },
            dispose_module: unsafe { symbol(library, b"LLVMDisposeModule\0")? },
            create_memory_buffer: unsafe {
                symbol(library, b"LLVMCreateMemoryBufferWithMemoryRangeCopy\0")?
            },
            parse_ir: unsafe { symbol(library, b"LLVMParseIRInContext\0")? },
            dispose_message: unsafe { symbol(library, b"LLVMDisposeMessage\0")? },
            create_thread_safe_context: unsafe {
                symbol(
                    library,
                    b"LLVMOrcCreateNewThreadSafeContextFromLLVMContext\0",
                )?
            },
            dispose_thread_safe_context: unsafe {
                symbol(library, b"LLVMOrcDisposeThreadSafeContext\0")?
            },
            create_thread_safe_module: unsafe {
                symbol(library, b"LLVMOrcCreateNewThreadSafeModule\0")?
            },
            create_lljit: unsafe { symbol(library, b"LLVMOrcCreateLLJIT\0")? },
            dispose_lljit: unsafe { symbol(library, b"LLVMOrcDisposeLLJIT\0")? },
            get_main_jit_dylib: unsafe { symbol(library, b"LLVMOrcLLJITGetMainJITDylib\0")? },
            add_ir_module: unsafe { symbol(library, b"LLVMOrcLLJITAddLLVMIRModule\0")? },
            add_object_file: unsafe { symbol(library, b"LLVMOrcLLJITAddObjectFile\0")? },
            lookup: unsafe { symbol(library, b"LLVMOrcLLJITLookup\0")? },
            get_jit_triple: unsafe { symbol(library, b"LLVMOrcLLJITGetTripleString\0")? },
            get_host_cpu_name: unsafe { symbol(library, b"LLVMGetHostCPUName\0")? },
            get_host_cpu_features: unsafe { symbol(library, b"LLVMGetHostCPUFeatures\0")? },
            get_buffer_start: unsafe { symbol(library, b"LLVMGetBufferStart\0")? },
            get_buffer_size: unsafe { symbol(library, b"LLVMGetBufferSize\0")? },
            get_object_transform_layer: unsafe {
                symbol(library, b"LLVMOrcLLJITGetObjTransformLayer\0")?
            },
            set_object_transform: unsafe {
                symbol(library, b"LLVMOrcObjectTransformLayerSetTransform\0")?
            },
            get_error_message: unsafe { symbol(library, b"LLVMGetErrorMessage\0")? },
            dispose_error_message: unsafe { symbol(library, b"LLVMDisposeErrorMessage\0")? },
            verify_module: unsafe { symbol(library, b"LLVMVerifyModule\0")? },
            run_passes: unsafe { symbol(library, b"LLVMRunPasses\0")? },
            create_pass_builder_options: unsafe {
                symbol(library, b"LLVMCreatePassBuilderOptions\0")?
            },
            dispose_pass_builder_options: unsafe {
                symbol(library, b"LLVMDisposePassBuilderOptions\0")?
            },
            initialize_target_info: unsafe {
                symbol(library, target_symbol("TargetInfo").as_bytes())?
            },
            initialize_target: unsafe { symbol(library, target_symbol("Target").as_bytes())? },
            initialize_target_mc: unsafe { symbol(library, target_symbol("TargetMC").as_bytes())? },
            initialize_asm_printer: unsafe {
                symbol(library, target_symbol("AsmPrinter").as_bytes())?
            },
            _library: Arc::clone(library),
        })
    }

    pub fn initialize_native_target(&self) {
        NATIVE_TARGET_INITIALIZED.get_or_init(|| unsafe {
            (self.initialize_target_info)();
            (self.initialize_target)();
            (self.initialize_target_mc)();
            (self.initialize_asm_printer)();
        });
    }

    pub unsafe fn error_message(&self, error: ErrorRef) -> Option<String> {
        if error.is_null() {
            return None;
        }
        let message = unsafe { (self.get_error_message)(error) };
        if message.is_null() {
            return Some("LLVM returned an error without a message".to_string());
        }
        let owned = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        unsafe { (self.dispose_error_message)(message) };
        Some(owned)
    }

    pub unsafe fn dispose_jit_ignoring_error(&self, jit: LlJitRef) {
        if jit.is_null() {
            return;
        }
        let error = unsafe { (self.dispose_lljit)(jit) };
        let _ = unsafe { self.error_message(error) };
    }
}

fn load_library() -> Result<SharedLibrary, String> {
    let candidates = library_candidates(env::var_os("AKRON_LLVM_DYLIB"));
    let mut errors = Vec::new();
    for candidate in candidates {
        match unsafe { load_library_candidate(&candidate) } {
            Ok(library) => return Ok(library),
            Err(error) => errors.push(format!("{}: {error}", candidate.to_string_lossy())),
        }
    }
    Err(format!(
        "could not load a supported LLVM 21 shared library: {}",
        errors.join("; ")
    ))
}

unsafe fn load_library_candidate(path: &OsString) -> Result<SharedLibrary, String> {
    let library = Arc::new(unsafe { Library::new(path) }.map_err(|error| error.to_string())?);
    let get_version: GetVersionFn = unsafe { symbol(&library, b"LLVMGetVersion\0")? };
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;
    unsafe { get_version(&mut major, &mut minor, &mut patch) };
    if major != 21 {
        return Err(format!(
            "unsupported LLVM C API {major}.{minor}.{patch}; this backend requires LLVM 21"
        ));
    }

    Ok(SharedLibrary {
        library,
        path: path.to_string_lossy().into_owned(),
        version: super::LlvmVersion {
            major,
            minor,
            patch,
        },
    })
}

unsafe fn symbol<T: Copy>(library: &Library, name: &[u8]) -> Result<T, String> {
    unsafe { library.get::<T>(name) }
        .map(|symbol| *symbol)
        .map_err(|error| error.to_string())
}

fn library_candidates(selected: Option<OsString>) -> Vec<OsString> {
    if let Some(path) = selected {
        return vec![path];
    }
    let mut candidates = Vec::new();
    #[cfg(target_os = "linux")]
    {
        candidates.extend(
            [
                "libLLVM.so.21.1",
                "libLLVM.so.21",
                "libLLVM-21.so",
                "/usr/lib64/libLLVM.so.21.1",
                "/lib64/libLLVM.so.21.1",
            ]
            .into_iter()
            .map(OsString::from),
        );
    }
    #[cfg(target_os = "macos")]
    candidates.extend(
        ["libLLVM.dylib", "/opt/homebrew/opt/llvm/lib/libLLVM.dylib"]
            .into_iter()
            .map(OsString::from),
    );
    candidates
}

#[cfg(test)]
mod tests {
    use super::library_candidates;
    use std::ffi::OsString;

    #[test]
    fn explicit_library_path_disables_runtime_name_fallbacks() {
        let selected = OsString::from("/opt/llvm-21/lib/libLLVM.so");

        assert_eq!(library_candidates(Some(selected.clone())), vec![selected]);
    }
}
