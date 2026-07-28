use std::collections::HashMap;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use memory::field::PrimeId;
use memory::{BigInt, BigIntWidth, CircomHandle, FieldElement, Function, Value, I60_MAX, I60_MIN};

use crate::loader::{validate_field_limbs, LoaderError};
use crate::specs::{
    SER_TAG_BIGINT, SER_TAG_BYTES, SER_TAG_CIRCOM_HANDLE, SER_TAG_FALSE, SER_TAG_FIELD,
    SER_TAG_INT, SER_TAG_NIL, SER_TAG_STRING, SER_TAG_TRUE,
};

use super::{
    CompiledProgram, ProgramCapabilities, EXECUTABLE_FORMAT_VERSION, MAX_BIGINTS, MAX_BLOBS,
    MAX_BLOB_LEN, MAX_BYTECODE, MAX_CIRCOM_ARGS, MAX_CIRCOM_HANDLES, MAX_CONSTANTS, MAX_FIELDS,
    MAX_FUNCTIONS, MAX_STRINGS, MAX_STRING_LEN, MAX_UPVALUES,
};

impl CompiledProgram {
    /// Read a current-format ACHB image without mutating a VM.
    pub fn read_executable<R: Read>(reader: &mut R) -> Result<Self, LoaderError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic[..3] != b"ACH" {
            return Err(LoaderError::Format("invalid ACHB magic".to_string()));
        }
        if magic[3] != EXECUTABLE_FORMAT_VERSION {
            return Err(LoaderError::Format(format!(
                "CompiledProgram reader requires format 0x{EXECUTABLE_FORMAT_VERSION:02x}, got 0x{:02x}",
                magic[3]
            )));
        }
        Self::read_v12_body(reader)
    }

    pub(crate) fn read_v12_body<R: Read>(reader: &mut R) -> Result<Self, LoaderError> {
        let prime_byte = reader.read_u8()?;
        let prime_id = PrimeId::from_byte(prime_byte).ok_or_else(|| {
            LoaderError::Format(format!("unknown PrimeId byte 0x{prime_byte:02x}"))
        })?;
        let bytecode_version = reader.read_u16::<LittleEndian>()?;
        let capability_bits = reader.read_u64::<LittleEndian>()?;
        let capabilities = ProgramCapabilities::from_bits(capability_bits).ok_or_else(|| {
            LoaderError::Format(format!("unknown capability bits 0x{capability_bits:x}"))
        })?;
        let main_max_slots = reader.read_u16::<LittleEndian>()?;

        let strings = read_strings(reader)?;
        let fields = read_fields(reader, prime_id)?;
        let bigints = read_bigints(reader)?;
        let blobs = read_blobs(reader)?;
        let circom_handles = read_circom_handles(reader)?;

        let main_constants = read_constants(reader)?;
        let function_count = read_count(reader, MAX_FUNCTIONS, "prototype")?;
        let mut functions = Vec::with_capacity(function_count);
        for _ in 0..function_count {
            functions.push(read_function(reader)?);
        }

        let main_chunk = read_u32_values(reader, MAX_BYTECODE, "main bytecode")?;
        let main_line_info = read_u32_values(reader, MAX_BYTECODE, "main line table")?;
        let debug_symbols = read_debug_symbols(reader)?;

        let program = Self {
            format_version: EXECUTABLE_FORMAT_VERSION,
            bytecode_version,
            capabilities,
            prime_id,
            strings,
            fields,
            bigints,
            blobs,
            circom_handles,
            functions,
            main: Function {
                name: "main".to_string(),
                arity: 0,
                max_slots: main_max_slots,
                chunk: main_chunk,
                constants: main_constants,
                upvalue_info: Vec::new(),
                line_info: main_line_info,
            },
            debug_symbols,
        };
        program.validate()?;
        Ok(program)
    }
}

fn read_strings<R: Read>(reader: &mut R) -> Result<Vec<String>, LoaderError> {
    let count = read_count(reader, MAX_STRINGS, "string")?;
    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = read_sized_bytes(reader, MAX_STRING_LEN, "string")?;
        strings.push(
            String::from_utf8(bytes)
                .map_err(|_| LoaderError::Format("invalid UTF-8 string".to_string()))?,
        );
    }
    Ok(strings)
}

fn read_fields<R: Read>(
    reader: &mut R,
    prime_id: PrimeId,
) -> Result<Vec<FieldElement>, LoaderError> {
    let count = read_count(reader, MAX_FIELDS, "field")?;
    let mut fields = Vec::with_capacity(count);
    for index in 0..count {
        let limbs = [
            reader.read_u64::<LittleEndian>()?,
            reader.read_u64::<LittleEndian>()?,
            reader.read_u64::<LittleEndian>()?,
            reader.read_u64::<LittleEndian>()?,
        ];
        if !validate_field_limbs(limbs, prime_id) {
            return Err(LoaderError::Format(format!(
                "field constant {index} exceeds {} modulus",
                prime_id.name()
            )));
        }
        fields.push(FieldElement::from_canonical(limbs));
    }
    Ok(fields)
}

fn read_bigints<R: Read>(reader: &mut R) -> Result<Vec<BigInt>, LoaderError> {
    let count = read_count(reader, MAX_BIGINTS, "BigInt")?;
    let mut bigints = Vec::with_capacity(count);
    for _ in 0..count {
        let (width, limb_count) = match reader.read_u8()? {
            0 => (BigIntWidth::W256, 4),
            1 => (BigIntWidth::W512, 8),
            tag => {
                return Err(LoaderError::Format(format!(
                    "unknown BigInt width tag 0x{tag:02x}"
                )));
            }
        };
        let mut limbs = Vec::with_capacity(limb_count);
        for _ in 0..limb_count {
            limbs.push(reader.read_u64::<LittleEndian>()?);
        }
        bigints.push(
            BigInt::from_limbs(limbs, width)
                .ok_or_else(|| LoaderError::Format("invalid BigInt limbs".to_string()))?,
        );
    }
    Ok(bigints)
}

fn read_blobs<R: Read>(reader: &mut R) -> Result<Vec<Vec<u8>>, LoaderError> {
    let count = read_count(reader, MAX_BLOBS, "bytes blob")?;
    let mut blobs = Vec::with_capacity(count);
    for _ in 0..count {
        blobs.push(read_sized_bytes(reader, MAX_BLOB_LEN, "bytes blob")?);
    }
    Ok(blobs)
}

fn read_circom_handles<R: Read>(reader: &mut R) -> Result<Vec<CircomHandle>, LoaderError> {
    let count = read_count(reader, MAX_CIRCOM_HANDLES, "circom handle")?;
    let mut handles = Vec::with_capacity(count);
    for _ in 0..count {
        let library_id = reader.read_u32::<LittleEndian>()?;
        let name = read_sized_bytes(reader, MAX_STRING_LEN, "circom template name")?;
        let template_name = String::from_utf8(name)
            .map_err(|_| LoaderError::Format("invalid UTF-8 circom template name".to_string()))?;
        let argument_count = read_count(reader, MAX_CIRCOM_ARGS, "circom argument")?;
        let mut template_args = Vec::with_capacity(argument_count);
        for _ in 0..argument_count {
            template_args.push(reader.read_u64::<LittleEndian>()?);
        }
        handles.push(CircomHandle {
            library_id,
            template_name,
            template_args,
        });
    }
    Ok(handles)
}

fn read_count<R: Read>(reader: &mut R, maximum: u32, context: &str) -> Result<usize, LoaderError> {
    let count = reader.read_u32::<LittleEndian>()?;
    if count > maximum {
        return Err(LoaderError::Security(format!(
            "{context} count {count} exceeds limit {maximum}"
        )));
    }
    Ok(count as usize)
}

fn read_sized_bytes<R: Read>(
    reader: &mut R,
    maximum: u32,
    context: &str,
) -> Result<Vec<u8>, LoaderError> {
    let length = reader.read_u32::<LittleEndian>()?;
    if length > maximum {
        return Err(LoaderError::Security(format!(
            "{context} length {length} exceeds limit {maximum}"
        )));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_constants<R: Read>(reader: &mut R) -> Result<Vec<Value>, LoaderError> {
    let count = read_count(reader, MAX_CONSTANTS, "constant")?;
    let mut constants = Vec::with_capacity(count);
    for _ in 0..count {
        let value = match reader.read_u8()? {
            SER_TAG_INT => {
                let integer = reader.read_i64::<LittleEndian>()?;
                if !(I60_MIN..=I60_MAX).contains(&integer) {
                    return Err(LoaderError::Security(format!(
                        "integer constant {integer} outside i60 range"
                    )));
                }
                Value::int(integer)
            }
            SER_TAG_STRING => Value::string(reader.read_u32::<LittleEndian>()?),
            SER_TAG_FALSE => Value::bool(false),
            SER_TAG_TRUE => Value::bool(true),
            SER_TAG_FIELD => Value::field(reader.read_u32::<LittleEndian>()?),
            SER_TAG_BIGINT => Value::bigint(reader.read_u32::<LittleEndian>()?),
            SER_TAG_BYTES => Value::bytes(reader.read_u32::<LittleEndian>()?),
            SER_TAG_CIRCOM_HANDLE => Value::circom_handle(reader.read_u32::<LittleEndian>()?),
            SER_TAG_NIL => Value::nil(),
            tag => {
                return Err(LoaderError::Format(format!(
                    "unknown constant tag 0x{tag:02x}"
                )));
            }
        };
        constants.push(value);
    }
    Ok(constants)
}

fn read_function<R: Read>(reader: &mut R) -> Result<Function, LoaderError> {
    let name = read_sized_bytes(reader, MAX_STRING_LEN, "function name")?;
    let name = String::from_utf8(name)
        .map_err(|_| LoaderError::Format("invalid UTF-8 function name".to_string()))?;
    let arity = reader.read_u8()?;
    let max_slots = reader.read_u16::<LittleEndian>()?;
    let constants = read_constants(reader)?;
    let upvalue_count = read_count(reader, MAX_UPVALUES, "function upvalue")?;
    let mut upvalue_info = vec![0; upvalue_count * 2];
    reader.read_exact(&mut upvalue_info)?;
    let chunk = read_u32_values(reader, MAX_BYTECODE, "function bytecode")?;
    let line_info = read_u32_values(reader, MAX_BYTECODE, "function line table")?;
    Ok(Function {
        name,
        arity,
        max_slots,
        chunk,
        constants,
        upvalue_info,
        line_info,
    })
}

fn read_u32_values<R: Read>(
    reader: &mut R,
    maximum: u32,
    context: &str,
) -> Result<Vec<u32>, LoaderError> {
    let count = read_count(reader, maximum, context)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(reader.read_u32::<LittleEndian>()?);
    }
    Ok(values)
}

fn read_debug_symbols<R: Read>(reader: &mut R) -> Result<HashMap<u16, String>, LoaderError> {
    let mut magic = [0; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xDB, 0x67] {
        return Err(LoaderError::Format(
            "invalid or missing debug section".to_string(),
        ));
    }
    let count = reader.read_u16::<LittleEndian>()? as usize;
    let mut symbols = HashMap::with_capacity(count);
    for _ in 0..count {
        let index = reader.read_u16::<LittleEndian>()?;
        let length = reader.read_u16::<LittleEndian>()? as usize;
        if length > MAX_STRING_LEN as usize {
            return Err(LoaderError::Security(
                "debug symbol name too long".to_string(),
            ));
        }
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes)?;
        let name = String::from_utf8(bytes)
            .map_err(|_| LoaderError::Format("invalid UTF-8 debug symbol".to_string()))?;
        symbols.insert(index, name);
    }
    Ok(symbols)
}
