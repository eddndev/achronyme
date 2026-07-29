//! Canonical, owned representation of an Akron bytecode program.
//!
//! CompiledProgram is the boundary shared by the bytecode compiler,
//! interpreter, ACHB serializer, and native backends. Values in its constant
//! pools use table-relative handles. VM::load_program validates the image,
//! materializes heap objects, and remaps handles exactly once.

mod control_flow;
mod decode;
mod encode;
mod load;

use std::collections::HashMap;
use std::ops::{BitOr, BitOrAssign};

use memory::field::PrimeId;
use memory::{BigInt, CircomHandle, FieldElement, Function, Value};

use crate::loader::LoaderError;
use crate::opcode::instruction::{decode_bx, decode_opcode};
use crate::opcode::OpCode;

/// Current ACHB container version.
pub const EXECUTABLE_FORMAT_VERSION: u8 = 0x0C;

/// Version of the register-bytecode contract carried by the container.
pub const BYTECODE_VERSION: u16 = 1;

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

/// Runtime services required by a program image.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ProgramCapabilities(u64);

impl ProgramCapabilities {
    pub const PROVE: Self = Self(1 << 0);
    pub const CIRCOM: Self = Self(1 << 1);
    pub const NATIVE_CALLS: Self = Self(1 << 2);
    pub const FILE_IO: Self = Self(1 << 3);
    pub const VERIFY: Self = Self(1 << 4);

    const KNOWN_BITS: u64 =
        Self::PROVE.0 | Self::CIRCOM.0 | Self::NATIVE_CALLS.0 | Self::FILE_IO.0 | Self::VERIFY.0;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn from_bits(bits: u64) -> Option<Self> {
        (bits & !Self::KNOWN_BITS == 0).then_some(Self(bits))
    }
}

impl BitOr for ProgramCapabilities {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ProgramCapabilities {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

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
        };
        program.capabilities = program.derived_capabilities();
        program
    }

    pub fn instruction_count(&self) -> usize {
        self.main.chunk.len()
            + self
                .functions
                .iter()
                .map(|function| function.chunk.len())
                .sum::<usize>()
    }

    pub fn derived_capabilities(&self) -> ProgramCapabilities {
        let mut capabilities = ProgramCapabilities::empty();
        let verify_global = resolve::BuiltinRegistry::default()
            .lookup("verify_proof")
            .and_then(|entry| entry.vm_fn)
            .and_then(|handle| u16::try_from(handle.as_u32()).ok());
        for function in std::iter::once(&self.main).chain(self.functions.iter()) {
            for &instruction in &function.chunk {
                match OpCode::from_u8(decode_opcode(instruction)) {
                    Some(OpCode::Prove) => capabilities |= ProgramCapabilities::PROVE,
                    Some(OpCode::CallCircomTemplate) => capabilities |= ProgramCapabilities::CIRCOM,
                    Some(OpCode::GetGlobal) if Some(decode_bx(instruction)) == verify_global => {
                        capabilities |= ProgramCapabilities::VERIFY
                    }
                    Some(OpCode::Call) | Some(OpCode::MethodCall) => {
                        capabilities |= ProgramCapabilities::NATIVE_CALLS
                    }
                    _ => {}
                }
            }
        }
        capabilities
    }

    /// Validate container-level and bytecode-level invariants before any heap
    /// state is mutated.
    pub fn validate(&self) -> Result<(), LoaderError> {
        if self.format_version != EXECUTABLE_FORMAT_VERSION {
            return Err(LoaderError::Format(format!(
                "unsupported in-memory program format 0x{:02x}",
                self.format_version
            )));
        }
        if self.bytecode_version != BYTECODE_VERSION {
            return Err(LoaderError::Format(format!(
                "unsupported bytecode version {}",
                self.bytecode_version
            )));
        }
        self.validate_tables()?;

        let derived = self.derived_capabilities();
        if !self.capabilities.contains(derived) {
            return Err(LoaderError::Format(format!(
                "capability manifest 0x{:x} does not cover bytecode requirements 0x{:x}",
                self.capabilities.bits(),
                derived.bits()
            )));
        }
        if self.main.arity != 0 {
            return Err(LoaderError::Format(
                "main function must have arity 0".to_string(),
            ));
        }

        self.validate_function(&self.main, "main")?;
        for (index, function) in self.functions.iter().enumerate() {
            self.validate_function(function, &format!("prototype {index} ({})", function.name))?;
        }
        Ok(())
    }

    fn validate_tables(&self) -> Result<(), LoaderError> {
        check_len(self.strings.len(), MAX_STRINGS, "string table")?;
        check_len(self.fields.len(), MAX_FIELDS, "field table")?;
        check_len(self.bigints.len(), MAX_BIGINTS, "BigInt table")?;
        check_len(self.blobs.len(), MAX_BLOBS, "bytes table")?;
        check_len(
            self.circom_handles.len(),
            MAX_CIRCOM_HANDLES,
            "circom handle table",
        )?;
        check_len(self.functions.len(), MAX_FUNCTIONS, "function table")?;
        for string in &self.strings {
            check_len(string.len(), MAX_STRING_LEN, "string")?;
        }
        for blob in &self.blobs {
            check_len(blob.len(), MAX_BLOB_LEN, "bytes blob")?;
        }
        for handle in &self.circom_handles {
            check_len(
                handle.template_name.len(),
                MAX_STRING_LEN,
                "circom template name",
            )?;
            check_len(
                handle.template_args.len(),
                MAX_CIRCOM_ARGS,
                "circom template arguments",
            )?;
        }
        Ok(())
    }

    fn validate_function(&self, function: &Function, context: &str) -> Result<(), LoaderError> {
        check_len(function.chunk.len(), MAX_BYTECODE, "function bytecode")?;
        check_len(
            function.constants.len(),
            MAX_CONSTANTS,
            "function constants",
        )?;
        if !function.upvalue_info.len().is_multiple_of(2) {
            return Err(LoaderError::Format(format!(
                "{context} has odd upvalue_info length {}",
                function.upvalue_info.len()
            )));
        }
        check_len(
            function.upvalue_info.len() / 2,
            MAX_UPVALUES,
            "function upvalues",
        )?;
        for pair in function.upvalue_info.chunks_exact(2) {
            if pair[0] > 1 {
                return Err(LoaderError::Format(format!(
                    "{context} has invalid upvalue locality flag {}",
                    pair[0]
                )));
            }
        }
        if !function.line_info.is_empty() && function.line_info.len() != function.chunk.len() {
            return Err(LoaderError::Format(format!(
                "{context} line table length {} does not match bytecode length {}",
                function.line_info.len(),
                function.chunk.len()
            )));
        }

        for (index, value) in function.constants.iter().enumerate() {
            self.validate_constant(*value, context, index)?;
        }

        for (ip, &instruction) in function.chunk.iter().enumerate() {
            let opcode_byte = decode_opcode(instruction);
            let opcode = OpCode::from_u8(opcode_byte).ok_or_else(|| {
                LoaderError::Format(format!(
                    "{context} contains unknown opcode 0x{opcode_byte:02x} at ip {ip}"
                ))
            })?;
            let bx = decode_bx(instruction) as usize;
            match opcode {
                OpCode::LoadConst | OpCode::Prove if bx >= function.constants.len() => {
                    return Err(LoaderError::Format(format!(
                        "{context} references constant {bx} at ip {ip}, but has {} constants",
                        function.constants.len()
                    )));
                }
                OpCode::Closure if bx >= self.functions.len() => {
                    return Err(LoaderError::Format(format!(
                        "{context} references prototype {bx} at ip {ip}, but program has {}",
                        self.functions.len()
                    )));
                }
                OpCode::Jump | OpCode::JumpIfFalse | OpCode::ForIter
                    if bx >= function.chunk.len() =>
                {
                    return Err(LoaderError::Format(format!(
                        "{context} jump target {bx} is outside bytecode at ip {ip}"
                    )));
                }
                _ => {}
            }
        }
        control_flow::validate_reachable_returns(function, context)?;
        Ok(())
    }

    fn validate_constant(
        &self,
        value: Value,
        context: &str,
        index: usize,
    ) -> Result<(), LoaderError> {
        let handle = value.as_handle().map(|value| value as usize);
        let valid = if value.is_int() || value.is_nil() || value.is_bool() {
            true
        } else if value.is_string() {
            handle.is_some_and(|handle| handle < self.strings.len())
        } else if value.is_field() {
            handle.is_some_and(|handle| handle < self.fields.len())
        } else if value.is_bigint() {
            handle.is_some_and(|handle| handle < self.bigints.len())
        } else if value.is_bytes() {
            handle.is_some_and(|handle| handle < self.blobs.len())
        } else if value.is_circom_handle() {
            handle.is_some_and(|handle| handle < self.circom_handles.len())
        } else {
            false
        };

        if valid {
            Ok(())
        } else {
            Err(LoaderError::Format(format!(
                "{context} constant {index} has unsupported type or out-of-range handle: {value:?}"
            )))
        }
    }
}

pub(super) fn check_len(length: usize, maximum: u32, context: &str) -> Result<(), LoaderError> {
    if length > maximum as usize {
        Err(LoaderError::Security(format!(
            "{context} length {length} exceeds limit {maximum}"
        )))
    } else {
        Ok(())
    }
}

pub(super) fn required_handle(value: Value, context: &str) -> Result<u32, LoaderError> {
    value
        .as_handle()
        .ok_or_else(|| LoaderError::Format(format!("{context} value has no handle")))
}

#[cfg(test)]
mod tests;
