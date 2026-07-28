use std::ffi::CString;
use std::ptr;
use std::time::Instant;

use akron::CompiledProgram;

use crate::{lower_program_with_options, LlvmTierOptions};

use super::cache::{CacheIdentity, CacheMetadata, CachedObject, JitCacheConfig, ObjectCache};
use super::ffi::{LlJitRef, LlvmApi, ObjectCapture, TargetIdentity};
use super::{module, JitCacheStats, JitCompileTimings, JitEngine, JitError};

const LOWERING_REVISION: u32 = 2;
const CACHE_PIPELINE_ID: &str = "default<O2>;orc-lljit-object-transform-v1";

impl JitEngine {
    pub fn compile(program: &CompiledProgram) -> Result<Self, JitError> {
        Self::compile_with_cache_and_options(
            program,
            &JitCacheConfig::disabled(),
            LlvmTierOptions::default(),
        )
    }

    pub fn compile_with_options(
        program: &CompiledProgram,
        options: LlvmTierOptions,
    ) -> Result<Self, JitError> {
        Self::compile_with_cache_and_options(program, &JitCacheConfig::disabled(), options)
    }

    pub fn compile_cached(program: &CompiledProgram) -> Result<Self, JitError> {
        Self::compile_with_cache_and_options(
            program,
            &JitCacheConfig::from_environment(),
            LlvmTierOptions::default(),
        )
    }

    pub fn compile_with_cache(
        program: &CompiledProgram,
        config: &JitCacheConfig,
    ) -> Result<Self, JitError> {
        Self::compile_with_cache_and_options(program, config, LlvmTierOptions::default())
    }

    pub fn compile_with_cache_and_options(
        program: &CompiledProgram,
        config: &JitCacheConfig,
        options: LlvmTierOptions,
    ) -> Result<Self, JitError> {
        let runtime_requirement = options.runtime_requirement();
        let mut timings = JitCompileTimings::default();
        let (llvm, library_open, symbol_bind) = LlvmApi::load_with_timings()?;
        timings.llvm_library_open = library_open;
        timings.llvm_symbol_bind = symbol_bind;
        let started = Instant::now();
        llvm.initialize_native_target();
        timings.llvm_target_init = started.elapsed();
        timings.llvm_setup = library_open + symbol_bind + timings.llvm_target_init;

        let mut jit = create_jit(&llvm, &mut timings)?;
        let cache = ObjectCache::from_config(config);
        let mut cache_stats = JitCacheStats::default();
        let mut cache_key = None;

        if cache.is_enabled() {
            let started = Instant::now();
            let identity = match unsafe { llvm.target_identity(jit) } {
                Ok(identity) => identity,
                Err(error) => {
                    unsafe { llvm.dispose_jit_ignoring_error(jit) };
                    return Err(error);
                }
            };
            cache_key = cache_identity(program, &llvm, &identity, options).map(|value| value.key());
            timings.cache_key = started.elapsed();
        }

        if let Some(key) = cache_key {
            cache_stats.lookups = 1;
            let started = Instant::now();
            let cached = cache.lookup(key);
            timings.cache_lookup = started.elapsed();
            if let Some(cached) = cached {
                if let Ok(entry) = add_object_and_resolve(
                    &llvm,
                    jit,
                    &cached.object,
                    crate::lower::ENTRY_SYMBOL,
                    &mut timings,
                ) {
                    cache_stats.hits = 1;
                    return Ok(build_engine(
                        llvm,
                        jit,
                        entry,
                        cached.metadata,
                        timings,
                        cache_stats,
                        runtime_requirement,
                    ));
                }
                cache.invalidate(key);
                unsafe { llvm.dispose_jit_ignoring_error(jit) };
                jit = create_jit(&llvm, &mut timings)?;
            }
            cache_stats.misses = 1;
        }

        let started = Instant::now();
        let lowered = match lower_program_with_options(program, options) {
            Ok(lowered) => lowered,
            Err(error) => {
                unsafe { llvm.dispose_jit_ignoring_error(jit) };
                return Err(error.into());
            }
        };
        timings.lowering = started.elapsed();
        let metadata = cache_metadata(&lowered);

        let entry = if let Some(key) = cache_key {
            let mut capture = ObjectCapture::new(&llvm);
            if let Err(error) = unsafe { llvm.install_object_capture(jit, &mut capture) } {
                unsafe { llvm.dispose_jit_ignoring_error(jit) };
                return Err(error);
            }
            let entry =
                add_ir_and_resolve(&llvm, jit, &lowered.ir, lowered.entry_symbol, &mut timings)?;
            unsafe { llvm.reset_object_transform(jit) };
            if let Some((object, elapsed)) = capture.into_object() {
                timings.object_capture = elapsed;
                let cached = CachedObject { metadata, object };
                let started = Instant::now();
                cache.store(key, &cached);
                timings.cache_store = started.elapsed();
            }
            entry
        } else {
            add_ir_and_resolve(&llvm, jit, &lowered.ir, lowered.entry_symbol, &mut timings)?
        };

        Ok(build_engine(
            llvm,
            jit,
            entry,
            metadata,
            timings,
            cache_stats,
            runtime_requirement,
        ))
    }
}

fn cache_metadata(lowered: &crate::LoweredModule) -> CacheMetadata {
    CacheMetadata {
        instruction_count: lowered.instruction_count,
        native_instruction_count: lowered.native_instruction_count,
        direct_instruction_count: lowered.direct_instruction_count,
        runtime_instruction_count: lowered.runtime_instruction_count,
        compiled_call_count: lowered.compiled_call_count,
    }
}

fn cache_identity(
    program: &CompiledProgram,
    llvm: &LlvmApi,
    target: &TargetIdentity,
    options: LlvmTierOptions,
) -> Option<CacheIdentity> {
    let runtime = options.runtime_requirement();
    let mut canonical = Vec::new();
    program.write_executable(&mut canonical).ok()?;
    Some(CacheIdentity {
        program: canonical,
        runtime_abi_version: runtime.version,
        runtime_abi_size: runtime.size,
        runtime_capabilities: runtime.capabilities.bits(),
        llvm_version: llvm.version,
        target_triple: target.triple.clone(),
        cpu: target.cpu.clone(),
        features: target.features.clone(),
        optimization_pipeline: CACHE_PIPELINE_ID.to_string(),
        lowering_revision: LOWERING_REVISION,
    })
}

fn create_jit(llvm: &LlvmApi, timings: &mut JitCompileTimings) -> Result<LlJitRef, JitError> {
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
    timings.lljit_creation += started.elapsed();
    Ok(jit)
}

fn add_ir_and_resolve(
    llvm: &LlvmApi,
    jit: LlJitRef,
    ir: &str,
    entry_symbol: &str,
    timings: &mut JitCompileTimings,
) -> Result<akron::compiled::CompiledEntry, JitError> {
    let parsed = parse_and_optimize(llvm, ir, timings, jit)?;
    let started = Instant::now();
    let module = match unsafe { module::into_thread_safe(llvm, parsed) } {
        Ok(module) => module,
        Err(error) => {
            unsafe { llvm.dispose_jit_ignoring_error(jit) };
            return Err(error);
        }
    };
    timings.thread_safe_module = started.elapsed();

    let started = Instant::now();
    let dylib = unsafe { (llvm.get_main_jit_dylib)(jit) };
    let add_error = unsafe { (llvm.add_ir_module)(jit, dylib, module) };
    if let Some(message) = unsafe { llvm.error_message(add_error) } {
        unsafe { llvm.dispose_jit_ignoring_error(jit) };
        return Err(JitError::Llvm(format!(
            "could not add LLVM module to LLJIT: {message}"
        )));
    }
    timings.module_add = started.elapsed();
    resolve_entry(llvm, jit, entry_symbol, timings)
        .inspect_err(|_| unsafe { llvm.dispose_jit_ignoring_error(jit) })
}

fn parse_and_optimize(
    llvm: &LlvmApi,
    ir: &str,
    timings: &mut JitCompileTimings,
    jit: LlJitRef,
) -> Result<module::ParsedModule, JitError> {
    let started = Instant::now();
    let parsed = match unsafe { module::parse(llvm, ir) } {
        Ok(parsed) => parsed,
        Err(error) => {
            unsafe { llvm.dispose_jit_ignoring_error(jit) };
            return Err(error);
        }
    };
    timings.ir_parse = started.elapsed();

    let started = Instant::now();
    if let Err(error) = unsafe { module::optimize(llvm, &parsed) } {
        unsafe {
            parsed.dispose(llvm);
            llvm.dispose_jit_ignoring_error(jit);
        }
        return Err(error);
    }
    timings.llvm_optimization = started.elapsed();
    Ok(parsed)
}

fn add_object_and_resolve(
    llvm: &LlvmApi,
    jit: LlJitRef,
    object: &[u8],
    entry_symbol: &str,
    timings: &mut JitCompileTimings,
) -> Result<akron::compiled::CompiledEntry, JitError> {
    let started = Instant::now();
    let dylib = unsafe { (llvm.get_main_jit_dylib)(jit) };
    unsafe { llvm.add_object(jit, dylib, object)? };
    timings.module_add = started.elapsed();
    resolve_entry(llvm, jit, entry_symbol, timings)
}

fn resolve_entry(
    llvm: &LlvmApi,
    jit: LlJitRef,
    entry_symbol: &str,
    timings: &mut JitCompileTimings,
) -> Result<akron::compiled::CompiledEntry, JitError> {
    let started = Instant::now();
    let symbol = CString::new(entry_symbol).expect("entry symbol has no NUL");
    let mut address = 0u64;
    let lookup_error = unsafe { (llvm.lookup)(jit, &mut address, symbol.as_ptr()) };
    if let Some(message) = unsafe { llvm.error_message(lookup_error) } {
        return Err(JitError::Llvm(format!(
            "could not resolve JIT entry: {message}"
        )));
    }
    if address == 0 {
        return Err(JitError::Llvm(
            "LLVM resolved the JIT entry to a null address".to_string(),
        ));
    }
    timings.lookup_materialization = started.elapsed();
    Ok(unsafe { std::mem::transmute::<usize, akron::compiled::CompiledEntry>(address as usize) })
}

fn build_engine(
    llvm: LlvmApi,
    jit: LlJitRef,
    entry: akron::compiled::CompiledEntry,
    metadata: CacheMetadata,
    compile_timings: JitCompileTimings,
    cache_stats: JitCacheStats,
    runtime_requirement: crate::options::RuntimeRequirement,
) -> JitEngine {
    JitEngine {
        llvm,
        jit,
        entry,
        instruction_count: metadata.instruction_count,
        native_instruction_count: metadata.native_instruction_count,
        direct_instruction_count: metadata.direct_instruction_count,
        runtime_instruction_count: metadata.runtime_instruction_count,
        compiled_call_count: metadata.compiled_call_count,
        compile_timings,
        cache_stats,
        runtime_requirement,
    }
}
