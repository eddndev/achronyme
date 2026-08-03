//! Canonical, owned representation of an Akron bytecode program.
//!
//! CompiledProgram is the boundary shared by the bytecode compiler,
//! interpreter, ACHB serializer, and native backends. Values in its constant
//! pools use table-relative handles. VM::load_program validates the image,
//! materializes heap objects, and remaps handles exactly once.

mod capabilities;
mod control_flow;
mod decode;
mod encode;
mod load;
mod task_handles;
mod validation;

pub use capabilities::{ProgramCapabilities, ProgramNativeMetadata};

use std::collections::HashMap;

use memory::field::PrimeId;
use memory::{BigInt, CircomHandle, FieldElement, Function};

use crate::opcode::instruction::{decode_bx, decode_opcode};
use crate::opcode::OpCode;
use crate::specs::{CapabilitySet, EffectSet, NativeMeta};

/// Current ACHB container version. v13 adds canonical native effect metadata.
pub const EXECUTABLE_FORMAT_VERSION: u8 = 0x0D;
/// Previous canonical container emitted by Achronyme 0.0.1.
pub const LEGACY_CANONICAL_FORMAT_VERSION: u8 = 0x0C;

/// Version of the register-bytecode contract carried by the container.
pub const BYTECODE_VERSION: u16 = 2;
pub const LEGACY_BYTECODE_VERSION: u16 = 1;

pub(super) const MAX_STRINGS: u32 = 1_000_000;
pub(super) const MAX_STRING_LEN: u32 = 1024;
pub(super) const MAX_FIELDS: u32 = 1_000_000;
pub(super) const MAX_BIGINTS: u32 = 1_000_000;
pub(super) const MAX_BLOBS: u32 = 100_000;
pub(super) const MAX_BLOB_LEN: u32 = 64 * 1024 * 1024;
pub(super) const MAX_CIRCOM_HANDLES: u32 = 100_000;
pub(super) const MAX_CIRCOM_ARGS: u32 = 1024;
pub(super) const MAX_CONSTANTS: u32 = 1_000_000;
pub(super) const MAX_FUNCTIONS: u32 = 100_000;
pub(super) const MAX_UPVALUES: u32 = 1024;
pub(super) const MAX_BYTECODE: u32 = 1_000_000;
pub(super) const MAX_NATIVE_METADATA: u32 = u16::MAX as u32 + 1;

/// Owned bytecode image. Constant handles refer to the tables in this value
/// until VM::load_program remaps them into the VM heap.
#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub format_version: u8,
    pub bytecode_version: u16,
    pub capabilities: ProgramCapabilities,
    pub prime_id: PrimeId,
    pub strings: Vec<String>,
    pub fields: Vec<FieldElement>,
    pub bigints: Vec<BigInt>,
    pub blobs: Vec<Vec<u8>>,
    pub circom_handles: Vec<CircomHandle>,
    pub functions: Vec<Function>,
    pub main: Function,
    pub debug_symbols: HashMap<u16, String>,
    pub native_metadata: HashMap<u16, ProgramNativeMetadata>,
}

impl CompiledProgram {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        prime_id: PrimeId,
        strings: Vec<String>,
        fields: Vec<FieldElement>,
        bigints: Vec<BigInt>,
        blobs: Vec<Vec<u8>>,
        circom_handles: Vec<CircomHandle>,
        functions: Vec<Function>,
        main: Function,
        debug_symbols: HashMap<u16, String>,
    ) -> Self {
        let mut program = Self {
            format_version: EXECUTABLE_FORMAT_VERSION,
            bytecode_version: BYTECODE_VERSION,
            capabilities: ProgramCapabilities::empty(),
            prime_id,
            strings,
            fields,
            bigints,
            blobs,
            circom_handles,
            functions,
            main,
            debug_symbols,
            native_metadata: Self::builtin_native_metadata(),
        };
        program.capabilities = program.derived_capabilities();
        program
    }

    fn builtin_native_metadata() -> HashMap<u16, ProgramNativeMetadata> {
        Self::native_metadata_from_registry(&resolve::BuiltinRegistry::default())
    }

    fn native_metadata_from_registry(
        registry: &resolve::BuiltinRegistry,
    ) -> HashMap<u16, ProgramNativeMetadata> {
        registry
            .vm_entries_by_handle()
            .into_iter()
            .map(|entry| {
                let index = entry
                    .vm_fn
                    .expect("VM builtin metadata must have a VM handle")
                    .as_u32() as u16;
                (
                    index,
                    ProgramNativeMetadata {
                        effects: entry.effects,
                        capabilities: entry.capabilities,
                        behavior: entry.behavior,
                        cancellation: entry.cancellation,
                        resource: entry.resource,
                    },
                )
            })
            .collect()
    }

    /// Replace artifact native metadata with one audited canonical registry.
    pub fn register_native_registry(&mut self, registry: &resolve::BuiltinRegistry) {
        registry
            .audit()
            .expect("cannot register invalid native metadata in a program");
        self.native_metadata = Self::native_metadata_from_registry(registry);
        self.capabilities = self.derived_capabilities();
    }

    /// Register compiler-visible metadata for native globals after the builtins.
    pub fn register_extra_natives(&mut self, start: u16, metadata: &[NativeMeta]) {
        for (offset, meta) in metadata.iter().enumerate() {
            let offset = u16::try_from(offset).expect("native metadata count exceeds u16");
            let index = start
                .checked_add(offset)
                .expect("native metadata index exceeds u16");
            assert!(
                self.native_metadata
                    .insert(index, ProgramNativeMetadata::from_native_meta(meta))
                    .is_none(),
                "native metadata index {index} is already registered"
            );
        }
        self.capabilities = self.derived_capabilities();
    }

    /// Preserve source-level inferred effects in the artifact capability manifest.
    pub fn record_inferred_effects(&mut self, effects: EffectSet) {
        self.capabilities |= ProgramCapabilities::from_effects(effects);
    }

    pub fn instruction_count(&self) -> usize {
        self.main.chunk.len()
            + self
                .functions
                .iter()
                .map(|function| function.chunk.len())
                .sum::<usize>()
    }

    /// Effects requested by instructions and referenced native globals.
    /// Unused registry entries do not add authority to the manifest.
    pub fn requested_effects(&self) -> EffectSet {
        let mut effects = self.capabilities.inferred_effects();
        for function in std::iter::once(&self.main).chain(self.functions.iter()) {
            for &instruction in &function.chunk {
                match OpCode::from_u8(decode_opcode(instruction)) {
                    Some(
                        OpCode::ScopeEnter
                        | OpCode::ScopeExit
                        | OpCode::Spawn
                        | OpCode::Await
                        | OpCode::AwaitOutcome
                        | OpCode::TaskForget
                        | OpCode::AwaitRace
                        | OpCode::CancelCheck,
                    ) => effects |= EffectSet::TASK,
                    Some(OpCode::Prove) => effects |= EffectSet::PROVE,
                    Some(OpCode::CallCircomTemplate) => effects |= EffectSet::CIRCUIT,
                    Some(OpCode::GetGlobal) => {
                        if let Some(metadata) = self.native_metadata.get(&decode_bx(instruction)) {
                            effects |= metadata.effects;
                        }
                    }
                    _ => {}
                }
            }
        }
        effects
    }

    /// Exact host authorities requested by referenced native globals.
    pub fn requested_host_capabilities(&self) -> CapabilitySet {
        let mut capabilities = CapabilitySet::empty();
        if self
            .capabilities
            .contains(ProgramCapabilities::UNKNOWN_HOST)
        {
            capabilities |= CapabilitySet::UNKNOWN_HOST;
        }
        for function in std::iter::once(&self.main).chain(self.functions.iter()) {
            for &instruction in &function.chunk {
                if OpCode::from_u8(decode_opcode(instruction)) == Some(OpCode::GetGlobal) {
                    if let Some(metadata) = self.native_metadata.get(&decode_bx(instruction)) {
                        capabilities |= metadata.capabilities;
                    }
                }
            }
        }
        capabilities
    }

    pub fn derived_capabilities(&self) -> ProgramCapabilities {
        let mut capabilities = ProgramCapabilities::empty();
        for function in std::iter::once(&self.main).chain(self.functions.iter()) {
            for &instruction in &function.chunk {
                match OpCode::from_u8(decode_opcode(instruction)) {
                    Some(OpCode::Prove) => capabilities |= ProgramCapabilities::PROVE,
                    Some(OpCode::CallCircomTemplate) => capabilities |= ProgramCapabilities::CIRCOM,
                    Some(
                        OpCode::ScopeEnter
                        | OpCode::ScopeExit
                        | OpCode::Spawn
                        | OpCode::Await
                        | OpCode::AwaitOutcome
                        | OpCode::TaskForget
                        | OpCode::AwaitRace
                        | OpCode::CancelCheck,
                    ) => capabilities |= ProgramCapabilities::TASKS,
                    Some(OpCode::GetGlobal) => {
                        if let Some(metadata) = self.native_metadata.get(&decode_bx(instruction)) {
                            capabilities |= metadata.program_capabilities();
                        }
                    }
                    _ => {}
                }
            }
        }
        capabilities
    }
}

#[cfg(test)]
mod tests;
