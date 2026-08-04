use std::collections::HashMap;
use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};
use memory::{BigIntWidth, Function, Value};

use crate::loader::LoaderError;
use crate::specs::{
    SER_TAG_BIGINT, SER_TAG_BYTES, SER_TAG_CIRCOM_HANDLE, SER_TAG_FALSE, SER_TAG_FIELD,
    SER_TAG_INT, SER_TAG_NIL, SER_TAG_STRING, SER_TAG_TRUE,
};

use super::validation::required_handle;
use super::{CompiledProgram, EXECUTABLE_FORMAT_VERSION};

impl CompiledProgram {
    /// Serialize the current, self-describing ACHB format.
    pub fn write_executable<W: Write>(&self, writer: &mut W) -> Result<(), LoaderError> {
        self.validate()?;
        if self.format_version != EXECUTABLE_FORMAT_VERSION {
            return Err(LoaderError::Format(
                "legacy programs must be recompiled before serialization".to_string(),
            ));
        }

        writer.write_all(b"ACH")?;
        writer.write_u8(EXECUTABLE_FORMAT_VERSION)?;
        writer.write_u8(self.prime_id.to_byte())?;
        writer.write_u16::<LittleEndian>(self.bytecode_version)?;
        writer.write_u64::<LittleEndian>(self.capabilities.bits())?;
        writer.write_u16::<LittleEndian>(self.main.max_slots)?;

        write_count(writer, self.strings.len(), "string")?;
        for string in &self.strings {
            write_sized_bytes(writer, string.as_bytes(), "string")?;
        }

        write_count(writer, self.fields.len(), "field")?;
        for field in &self.fields {
            for limb in field.to_canonical() {
                writer.write_u64::<LittleEndian>(limb)?;
            }
        }

        write_count(writer, self.bigints.len(), "BigInt")?;
        for bigint in &self.bigints {
            writer.write_u8(match bigint.width() {
                BigIntWidth::W256 => 0,
                BigIntWidth::W512 => 1,
            })?;
            for &limb in bigint.limbs() {
                writer.write_u64::<LittleEndian>(limb)?;
            }
        }

        write_count(writer, self.blobs.len(), "bytes blob")?;
        for blob in &self.blobs {
            write_sized_bytes(writer, blob, "bytes blob")?;
        }

        write_count(writer, self.circom_handles.len(), "circom handle")?;
        for handle in &self.circom_handles {
            writer.write_u32::<LittleEndian>(handle.library_id)?;
            write_sized_bytes(
                writer,
                handle.template_name.as_bytes(),
                "circom template name",
            )?;
            write_count(
                writer,
                handle.template_args.len(),
                "circom template argument",
            )?;
            for &argument in &handle.template_args {
                writer.write_u64::<LittleEndian>(argument)?;
            }
        }

        write_constants(writer, &self.main.constants)?;
        write_count(writer, self.functions.len(), "prototype")?;
        for function in &self.functions {
            write_function(writer, function)?;
        }
        write_u32_values(writer, &self.main.chunk, "main bytecode")?;
        write_u32_values(writer, &self.main.line_info, "main line table")?;
        write_debug_symbols(writer, &self.debug_symbols)?;
        write_native_metadata(writer, &self.native_metadata)?;
        Ok(())
    }
}

fn write_native_metadata<W: Write>(
    writer: &mut W,
    metadata: &HashMap<u16, super::ProgramNativeMetadata>,
) -> Result<(), LoaderError> {
    write_count(writer, metadata.len(), "native metadata")?;
    let mut sorted: Vec<_> = metadata.iter().collect();
    sorted.sort_by_key(|(index, _)| **index);
    for (&index, metadata) in sorted {
        writer.write_u16::<LittleEndian>(index)?;
        writer.write_u32::<LittleEndian>(metadata.effects.bits())?;
        writer.write_u32::<LittleEndian>(metadata.capabilities.bits())?;
        writer.write_u8(metadata.behavior.to_byte())?;
        writer.write_u8(metadata.cancellation.to_byte())?;
        let (operation, kind) = metadata.resource.to_bytes();
        writer.write_u8(operation)?;
        writer.write_u8(kind)?;
    }
    Ok(())
}

fn write_count<W: Write>(writer: &mut W, count: usize, context: &str) -> Result<(), LoaderError> {
    let count = u32::try_from(count)
        .map_err(|_| LoaderError::Security(format!("{context} count exceeds u32")))?;
    writer.write_u32::<LittleEndian>(count)?;
    Ok(())
}

fn write_sized_bytes<W: Write>(
    writer: &mut W,
    bytes: &[u8],
    context: &str,
) -> Result<(), LoaderError> {
    write_count(writer, bytes.len(), context)?;
    writer.write_all(bytes)?;
    Ok(())
}

fn write_constants<W: Write>(writer: &mut W, constants: &[Value]) -> Result<(), LoaderError> {
    write_count(writer, constants.len(), "constant")?;
    for &value in constants {
        if let Some(integer) = value.as_int() {
            writer.write_u8(SER_TAG_INT)?;
            writer.write_i64::<LittleEndian>(integer)?;
        } else if value.is_string() {
            writer.write_u8(SER_TAG_STRING)?;
            writer.write_u32::<LittleEndian>(required_handle(value, "String")?)?;
        } else if let Some(boolean) = value.as_bool() {
            writer.write_u8(if boolean { SER_TAG_TRUE } else { SER_TAG_FALSE })?;
        } else if value.is_field() {
            writer.write_u8(SER_TAG_FIELD)?;
            writer.write_u32::<LittleEndian>(required_handle(value, "Field")?)?;
        } else if value.is_bigint() {
            writer.write_u8(SER_TAG_BIGINT)?;
            writer.write_u32::<LittleEndian>(required_handle(value, "BigInt")?)?;
        } else if value.is_bytes() {
            writer.write_u8(SER_TAG_BYTES)?;
            writer.write_u32::<LittleEndian>(required_handle(value, "Bytes")?)?;
        } else if value.is_circom_handle() {
            writer.write_u8(SER_TAG_CIRCOM_HANDLE)?;
            writer.write_u32::<LittleEndian>(required_handle(value, "Circom")?)?;
        } else if value.is_nil() {
            writer.write_u8(SER_TAG_NIL)?;
        } else {
            return Err(LoaderError::Format(format!(
                "unsupported serialized constant {value:?}"
            )));
        }
    }
    Ok(())
}

fn write_function<W: Write>(writer: &mut W, function: &Function) -> Result<(), LoaderError> {
    write_sized_bytes(writer, function.name.as_bytes(), "function name")?;
    writer.write_u8(function.arity)?;
    writer.write_u16::<LittleEndian>(function.max_slots)?;
    write_constants(writer, &function.constants)?;
    write_count(writer, function.upvalue_info.len() / 2, "function upvalue")?;
    writer.write_all(&function.upvalue_info)?;
    write_u32_values(writer, &function.chunk, "function bytecode")?;
    write_u32_values(writer, &function.line_info, "function line table")?;
    Ok(())
}

fn write_u32_values<W: Write>(
    writer: &mut W,
    values: &[u32],
    context: &str,
) -> Result<(), LoaderError> {
    write_count(writer, values.len(), context)?;
    for &value in values {
        writer.write_u32::<LittleEndian>(value)?;
    }
    Ok(())
}

fn write_debug_symbols<W: Write>(
    writer: &mut W,
    symbols: &HashMap<u16, String>,
) -> Result<(), LoaderError> {
    let count = u16::try_from(symbols.len())
        .map_err(|_| LoaderError::Security("too many debug symbols".to_string()))?;
    writer.write_all(&[0xDB, 0x67])?;
    writer.write_u16::<LittleEndian>(count)?;
    let mut sorted: Vec<_> = symbols.iter().collect();
    sorted.sort_by_key(|(index, _)| **index);
    for (&index, name) in sorted {
        let length = u16::try_from(name.len())
            .map_err(|_| LoaderError::Security("debug symbol name too long".to_string()))?;
        writer.write_u16::<LittleEndian>(index)?;
        writer.write_u16::<LittleEndian>(length)?;
        writer.write_all(name.as_bytes())?;
    }
    Ok(())
}
