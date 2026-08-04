use crate::error::RuntimeError;
use crate::globals::GlobalEntry;
use crate::module::{builtin_modules, NativeDef};
use crate::native::{NativeAsyncRequest, NativeObj};
use memory::Value;
use resolve::{Arity, BuiltinEntry};

pub(crate) struct PreparedAsyncNative {
    pub(crate) request: NativeAsyncRequest,
    pub(crate) resource: crate::specs::ResourceEffect,
}

/// Trait for native function registration
pub trait NativeRegistry {
    fn define_native(&mut self, def: &NativeDef) -> Result<(), RuntimeError>;
    fn bootstrap_natives(&mut self) -> Result<(), RuntimeError>;
}

impl NativeRegistry for super::vm::VM {
    fn define_native(&mut self, def: &NativeDef) -> Result<(), RuntimeError> {
        let name_string = def.name.to_string();

        // 1. Intern string (still needed for debugging/reflection later, but not for lookup key)
        if !self.interner.contains_key(&name_string) {
            let h = self.heap.alloc_string(name_string.clone())?;
            self.interner.insert(name_string.clone(), h);
        }

        // 2. Register Native Object
        let native = NativeObj {
            name: name_string,
            func: def.func,
            arity: def.arity,
            effects: def.effects,
            capabilities: def.capabilities,
            behavior: def.behavior,
            cancellation: def.cancellation,
            resource: def.resource,
            async_start: def.async_start,
        };
        self.natives.push(native);
        let native_idx = (self.natives.len() - 1) as u32;

        // 3. Register in Globals (Direct Push)
        // Compiler guarantees 0=print, 1=len, etc.
        let val = Value::native(native_idx);
        self.globals.push(GlobalEntry::new(val, false));
        Ok(())
    }

    fn bootstrap_natives(&mut self) -> Result<(), RuntimeError> {
        if !self.natives.is_empty() || !self.globals.is_empty() {
            panic!("VM must be empty before bootstrapping natives");
        }

        let modules = builtin_modules();

        {
            let mut seen = std::collections::HashSet::new();
            for module in &modules {
                assert!(
                    seen.insert(module.name()),
                    "Duplicate builtin module name: '{}'",
                    module.name()
                );
            }
        }

        let registry = resolve::BuiltinRegistry::default();
        let vm_entries = registry.vm_entries_by_handle();

        let mut all_defs = Vec::with_capacity(vm_entries.len());
        for module in &modules {
            all_defs.extend(module.natives());
        }

        assert_eq!(
            all_defs.len(),
            vm_entries.len(),
            "NativeModule definitions ({}) != registry VM entries ({})",
            all_defs.len(),
            vm_entries.len(),
        );
        for (i, (def, entry)) in all_defs.iter().zip(vm_entries.iter()).enumerate() {
            assert_eq!(
                def.name, entry.name,
                "Native index {i}: module says '{}' but registry says '{}'",
                def.name, entry.name,
            );
            assert_native_definition_matches_registry(i, def, entry);
        }

        for (def, entry) in all_defs.iter().zip(vm_entries) {
            let canonical = canonical_native_definition(def, entry);
            self.define_native(&canonical)?;
        }
        Ok(())
    }
}

fn registry_arity(entry: &BuiltinEntry) -> isize {
    match entry.arity {
        Arity::Fixed(value) => value as isize,
        Arity::Variadic => -1,
        Arity::Range(_, _) => panic!(
            "VM builtin {} uses a range arity unsupported by NativeDef",
            entry.name
        ),
    }
}

fn assert_native_definition_matches_registry(
    index: usize,
    definition: &NativeDef,
    entry: &BuiltinEntry,
) {
    assert_eq!(definition.arity, registry_arity(entry), "arity at {index}");
    assert_eq!(definition.effects, entry.effects, "effects at {index}");
    assert_eq!(
        definition.capabilities, entry.capabilities,
        "capabilities at {index}"
    );
    assert_eq!(definition.behavior, entry.behavior, "behavior at {index}");
    assert_eq!(
        definition.cancellation, entry.cancellation,
        "cancellation at {index}"
    );
    assert_eq!(definition.resource, entry.resource, "resource at {index}");
}

fn canonical_native_definition(definition: &NativeDef, entry: &BuiltinEntry) -> NativeDef {
    NativeDef {
        name: entry.name,
        func: definition.func,
        arity: registry_arity(entry),
        effects: entry.effects,
        capabilities: entry.capabilities,
        behavior: entry.behavior,
        cancellation: entry.cancellation,
        resource: entry.resource,
        async_start: definition.async_start,
    }
}

impl super::vm::VM {
    /// Invoke a native through the single capability and arity gate shared by
    /// interpreter, reentrant calls, JIT helpers, and the task scheduler.
    pub(crate) fn invoke_native(
        &mut self,
        handle: u32,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        let (func, arity, capabilities, name) = {
            let native = self
                .natives
                .get(handle as usize)
                .ok_or(RuntimeError::FunctionNotFound)?;
            (
                native.func,
                native.arity,
                native.capabilities,
                native.name.clone(),
            )
        };
        if arity != -1 && arity as usize != args.len() {
            return Err(RuntimeError::arity_mismatch(format!(
                "Expected {arity} args, got {}",
                args.len()
            )));
        }
        self.host_policy.require(capabilities, &name)?;
        self.native_call_depth = self.native_call_depth.saturating_add(1);
        let result = func(self, args);
        self.native_call_depth = self.native_call_depth.saturating_sub(1);
        result
    }

    pub(crate) fn prepare_async_native(
        &mut self,
        callee: Value,
        args: &[Value],
    ) -> Result<Option<PreparedAsyncNative>, RuntimeError> {
        if !callee.is_native() {
            return Ok(None);
        }
        let handle = callee.as_handle().ok_or(RuntimeError::FunctionNotFound)?;
        let (start, arity, capabilities, resource, name) = {
            let native = self
                .natives
                .get(handle as usize)
                .ok_or(RuntimeError::FunctionNotFound)?;
            (
                native.async_start,
                native.arity,
                native.capabilities,
                native.resource,
                native.name.clone(),
            )
        };
        let Some(start) = start else {
            return Ok(None);
        };
        if arity != -1 && arity as usize != args.len() {
            return Err(RuntimeError::arity_mismatch(format!(
                "Expected {arity} args, got {}",
                args.len()
            )));
        }
        self.host_policy.require(capabilities, &name)?;
        start(self, args).map(|request| Some(PreparedAsyncNative { request, resource }))
    }

    pub(crate) fn materialize_async_output(
        &mut self,
        output: crate::native::NativeAsyncOutput,
    ) -> Result<Value, RuntimeError> {
        use crate::native::NativeAsyncOutput;
        match output {
            NativeAsyncOutput::Nil => Ok(Value::nil()),
            NativeAsyncOutput::Int(value) => {
                Value::try_int(value).ok_or(RuntimeError::IntegerOverflow)
            }
            NativeAsyncOutput::Bool(value) => Ok(Value::bool(value)),
            NativeAsyncOutput::String(value) => Ok(Value::string(self.heap.alloc_string(value)?)),
            NativeAsyncOutput::Bytes(value) => Ok(Value::bytes(self.heap.alloc_bytes(value)?)),
            NativeAsyncOutput::Value(value) => Ok(value),
            NativeAsyncOutput::FileResource { handle, file } => {
                self.resources.activate_file(handle, file)
            }
            NativeAsyncOutput::Resource { kind, handle } => {
                self.resources.activate_network(handle, kind)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VM;

    #[test]
    fn test_native_alignment() {
        let registry = resolve::BuiltinRegistry::default();
        let native_count = registry.vm_native_count();
        let vm = VM::new();

        assert_eq!(vm.globals.len(), native_count);

        let first_val = vm.globals[0].value;
        assert!(first_val.is_native());

        for i in 0..native_count {
            assert!(vm.globals[i].value.is_native());
        }
    }

    #[test]
    fn test_module_names_match_registry() {
        let registry = resolve::BuiltinRegistry::default();
        let vm_entries = registry.vm_entries_by_handle();

        let modules = builtin_modules();
        let mut all_names: Vec<&str> = Vec::new();
        for module in &modules {
            for def in module.natives() {
                all_names.push(def.name);
            }
        }

        assert_eq!(all_names.len(), vm_entries.len());
        for (i, (name, entry)) in all_names.iter().zip(vm_entries.iter()).enumerate() {
            assert_eq!(
                *name, entry.name,
                "Mismatch at index {i}: module='{}' vs registry='{}'",
                name, entry.name
            );
        }
    }

    #[test]
    fn test_native_objects_match_registry_metadata() {
        let registry = resolve::BuiltinRegistry::default();
        let vm_entries = registry.vm_entries_by_handle();
        let vm = VM::new();

        assert_eq!(vm.natives.len(), vm_entries.len());
        for (native, entry) in vm.natives.iter().zip(vm_entries.iter()) {
            assert_eq!(native.effects, entry.effects, "effects for {}", entry.name);
            assert_eq!(
                native.capabilities, entry.capabilities,
                "capabilities for {}",
                entry.name
            );
            assert_eq!(
                native.behavior, entry.behavior,
                "behavior for {}",
                entry.name
            );
            assert_eq!(
                native.cancellation, entry.cancellation,
                "cancellation for {}",
                entry.name
            );
            assert_eq!(
                native.resource, entry.resource,
                "resource effect for {}",
                entry.name
            );
        }
    }

    #[test]
    fn test_each_module_has_natives() {
        let modules = builtin_modules();
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].name(), "core");
        assert_eq!(modules[1].name(), "bigint");
        assert_eq!(modules[2].name(), "task_control");

        for module in &modules {
            assert!(
                !module.natives().is_empty(),
                "Module '{}' has no natives",
                module.name()
            );
        }
    }

    fn privileged_dummy(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        Ok(Value::int(7))
    }

    #[test]
    fn native_invocation_enforces_host_capabilities() {
        use crate::specs::{
            CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, ResourceEffect,
        };

        let mut vm = VM::new();
        vm.define_native(&NativeDef {
            name: "privileged_dummy",
            func: privileged_dummy,
            arity: 0,
            effects: EffectSet::IO_FILE,
            capabilities: CapabilitySet::FILE_READ,
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::None,
            async_start: None,
        })
        .unwrap();
        let handle = (vm.natives.len() - 1) as u32;

        assert!(matches!(
            vm.call_value(Value::native(handle), &[]),
            Err(RuntimeError::CapabilityDenied(_))
        ));
        vm.host_policy.grant(CapabilitySet::FILE_READ);
        assert_eq!(
            vm.call_value(Value::native(handle), &[]).unwrap(),
            Value::int(7)
        );
    }
}
