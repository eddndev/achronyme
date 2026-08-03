use memory::{Function, Value};

use crate::loader::LoaderError;
use crate::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c, decode_opcode};
use crate::opcode::OpCode;
use crate::specs::{EffectSet, NativeBehavior};

use super::{
    control_flow, task_handles, CompiledProgram, BYTECODE_VERSION, EXECUTABLE_FORMAT_VERSION,
    LEGACY_BYTECODE_VERSION, LEGACY_CANONICAL_FORMAT_VERSION, MAX_BIGINTS, MAX_BLOBS, MAX_BLOB_LEN,
    MAX_BYTECODE, MAX_CIRCOM_ARGS, MAX_CIRCOM_HANDLES, MAX_CONSTANTS, MAX_FIELDS, MAX_FUNCTIONS,
    MAX_NATIVE_METADATA, MAX_STRINGS, MAX_STRING_LEN, MAX_UPVALUES,
};

impl CompiledProgram {
    /// Validate container-level and bytecode-level invariants before any heap
    /// state is mutated.
    pub fn validate(&self) -> Result<(), LoaderError> {
        match (self.format_version, self.bytecode_version) {
            (EXECUTABLE_FORMAT_VERSION, BYTECODE_VERSION)
            | (LEGACY_CANONICAL_FORMAT_VERSION, LEGACY_BYTECODE_VERSION) => {}
            (format, bytecode) => {
                return Err(LoaderError::Format(format!(
                    "unsupported format/bytecode pair 0x{format:02x}/{bytecode}"
                )));
            }
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
        check_len(
            self.native_metadata.len(),
            MAX_NATIVE_METADATA,
            "native metadata table",
        )?;
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
        for (&index, metadata) in &self.native_metadata {
            let required_effects = metadata.capabilities.required_effects();
            if !metadata.effects.contains(required_effects) {
                return Err(LoaderError::Format(format!(
                    "native metadata {index} capabilities require effects `{required_effects}`"
                )));
            }
            if metadata.behavior == NativeBehavior::Suspending
                && !metadata.effects.contains(EffectSet::TASK)
            {
                return Err(LoaderError::Format(format!(
                    "suspending native metadata {index} is missing the task effect"
                )));
            }
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
                OpCode::ScopeEnter | OpCode::ScopeExit | OpCode::CancelCheck
                    if instruction & 0x00ff_ffff != 0 =>
                {
                    return Err(LoaderError::Format(format!(
                        "{context} has non-zero operands for {opcode} at ip {ip}"
                    )));
                }
                OpCode::TaskForget => {
                    let task = decode_a(instruction) as usize;
                    if decode_b(instruction) != 0
                        || decode_c(instruction) != 0
                        || task >= function.max_slots as usize
                    {
                        return Err(LoaderError::Format(format!(
                            "{context} has invalid TaskForget operands at ip {ip}"
                        )));
                    }
                }
                OpCode::Spawn => {
                    let destination = decode_a(instruction) as usize;
                    let callee = decode_b(instruction) as usize;
                    let argument_count = decode_c(instruction) as usize;
                    let argument_end = callee
                        .checked_add(1)
                        .and_then(|start| start.checked_add(argument_count));
                    if destination >= function.max_slots as usize
                        || callee >= function.max_slots as usize
                        || argument_end.is_none_or(|end| end > function.max_slots as usize)
                    {
                        return Err(LoaderError::Format(format!(
                            "{context} has out-of-range Spawn registers at ip {ip}"
                        )));
                    }
                }
                OpCode::Await | OpCode::AwaitOutcome => {
                    let destination = decode_a(instruction) as usize;
                    let task = decode_b(instruction) as usize;
                    if decode_c(instruction) != 0
                        || destination >= function.max_slots as usize
                        || task >= function.max_slots as usize
                    {
                        return Err(LoaderError::Format(format!(
                            "{context} has invalid {opcode} operands at ip {ip}"
                        )));
                    }
                }
                OpCode::AwaitRace => {
                    let destination = decode_a(instruction) as usize;
                    let start = decode_b(instruction) as usize;
                    let count = decode_c(instruction) as usize;
                    let end = start.checked_add(count);
                    if count < 2
                        || destination >= function.max_slots as usize
                        || end.is_none_or(|end| end > function.max_slots as usize)
                        || end.is_some_and(|end| (start..end).contains(&destination))
                    {
                        return Err(LoaderError::Format(format!(
                            "{context} has invalid AwaitRace operands at ip {ip}"
                        )));
                    }
                }
                _ => {}
            }
        }
        control_flow::validate_reachable_returns(function, context)?;
        task_handles::validate_non_escaping(function, &self.functions, context)?;
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
