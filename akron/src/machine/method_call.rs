use crate::opcode::instruction::{decode_a, decode_b, decode_c};
use crate::RuntimeError;

use super::stack::StackOps;

pub trait MethodCallOps {
    fn handle_method_call(&mut self, instruction: u32, base: usize) -> Result<(), RuntimeError>;
}

impl MethodCallOps for super::vm::VM {
    fn handle_method_call(&mut self, instruction: u32, base: usize) -> Result<(), RuntimeError> {
        let destination = decode_a(instruction) as usize;
        let receiver_register = decode_b(instruction) as usize;
        let argument_count = decode_c(instruction) as usize;

        let name_value = self.get_reg(base, receiver_register.wrapping_sub(1))?;
        let name_handle = name_value.as_handle().ok_or_else(|| {
            RuntimeError::type_mismatch("MethodCall: method name register is not a string")
        })?;
        let method_name = self
            .heap
            .get_string(name_handle)
            .ok_or(RuntimeError::stale_heap("String", "MethodCall name"))?
            .clone();
        let receiver = self.get_reg(base, receiver_register)?;
        let tag = receiver.tag();
        let mut arguments = Vec::with_capacity(argument_count);
        for offset in 1..=argument_count {
            arguments.push(self.get_reg(base, receiver_register + offset)?);
        }

        if let Some(method) = self.prototype_registry.lookup(tag, &method_name) {
            let result = method(self, receiver, &arguments)?;
            self.set_reg(base, destination, result)?;
            return Ok(());
        }
        if receiver.is_map() {
            let map_handle = receiver
                .as_handle()
                .ok_or_else(|| RuntimeError::type_mismatch("bad map handle"))?;
            let callee = self
                .heap
                .get_map(map_handle)
                .and_then(|map| map.get(&method_name).copied())
                .ok_or_else(|| {
                    RuntimeError::type_mismatch(format!("Map has no method or key '{method_name}'"))
                })?;
            if !callee.is_closure() && !callee.is_native() {
                return Err(RuntimeError::type_mismatch(format!(
                    "Map key '{method_name}' is not callable"
                )));
            }
            let result = self.call_value(callee, &arguments)?;
            self.set_reg(base, destination, result)?;
            return Ok(());
        }

        let type_name = match tag {
            0 => "Int",
            1 => "Nil",
            2 | 3 => "Bool",
            4 => "String",
            5 => "List",
            6 => "Map",
            7 => "Function",
            8 => "Field",
            9 => "Proof",
            10 => "Native",
            11 => "Function",
            12 => "Iterator",
            13 => "BigInt",
            14 => "Bytes",
            15 => "CircomHandle",
            _ => "Unknown",
        };
        Err(RuntimeError::type_mismatch(format!(
            "{type_name} has no method '{method_name}'"
        )))
    }
}
