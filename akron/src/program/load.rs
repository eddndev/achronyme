use memory::{Closure, Value};

use crate::loader::LoaderError;
use crate::opcode::instruction::{decode_bx, decode_opcode};
use crate::{CallFrame, GlobalEntry, OpCode, VM};

use super::{required_handle, CompiledProgram};

impl VM {
    /// Materialize a validated program into this VM.
    pub fn load_program(&mut self, mut program: CompiledProgram) -> Result<(), LoaderError> {
        program.validate()?;
        if !self.frames.is_empty() || !self.prototypes.is_empty() {
            return Err(LoaderError::Format(
                "VM already contains a loaded program".to_string(),
            ));
        }

        self.last_result = Value::nil();
        self.last_error_location = None;

        let required_globals = required_global_slots(&program, self.globals.len());
        self.globals
            .resize(required_globals, GlobalEntry::undefined());

        self.prime_id = program.prime_id;
        self.import_strings(program.strings);
        self.heap.import_bytes(program.blobs);
        self.heap.import_circom_handles(program.circom_handles);

        let field_map = self.heap.import_fields(program.fields)?;
        let bigint_map = self.heap.import_bigints(program.bigints)?;
        for function in &mut program.functions {
            remap_constants(&mut function.constants, &field_map, &bigint_map)?;
        }
        remap_constants(&mut program.main.constants, &field_map, &bigint_map)?;

        self.debug_symbols = Some(program.debug_symbols);
        for function in program.functions {
            self.prototypes.push(self.heap.alloc_function(function)?);
        }

        let function = self.heap.alloc_function(program.main)?;
        let closure = self.heap.alloc_closure(Closure {
            function,
            upvalues: Vec::new(),
        })?;
        self.frames.push(CallFrame {
            closure,
            ip: 0,
            base: 0,
            dest_reg: 0,
        });
        Ok(())
    }
}

fn required_global_slots(program: &CompiledProgram, existing: usize) -> usize {
    std::iter::once(&program.main)
        .chain(program.functions.iter())
        .flat_map(|function| function.chunk.iter().copied())
        .filter_map(|instruction| {
            let opcode = OpCode::from_u8(decode_opcode(instruction))?;
            matches!(
                opcode,
                OpCode::DefGlobalVar | OpCode::DefGlobalLet | OpCode::GetGlobal | OpCode::SetGlobal
            )
            .then(|| decode_bx(instruction) as usize + 1)
        })
        .fold(existing, usize::max)
}

fn remap_constants(
    constants: &mut [Value],
    field_map: &[u32],
    bigint_map: &[u32],
) -> Result<(), LoaderError> {
    for value in constants {
        if value.is_field() {
            let index = required_handle(*value, "Field")? as usize;
            let handle = field_map.get(index).ok_or_else(|| {
                LoaderError::Format(format!("Field handle out of range: {index}"))
            })?;
            *value = Value::field(*handle);
        } else if value.is_bigint() {
            let index = required_handle(*value, "BigInt")? as usize;
            let handle = bigint_map.get(index).ok_or_else(|| {
                LoaderError::Format(format!("BigInt handle out of range: {index}"))
            })?;
            *value = Value::bigint(*handle);
        }
    }
    Ok(())
}
