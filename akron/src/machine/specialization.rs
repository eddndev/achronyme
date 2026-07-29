use crate::error::RuntimeError;
use crate::opcode::instruction::{decode_a, decode_b, decode_c};

impl super::vm::VM {
    pub(crate) fn execute_list_push_specialization(
        &mut self,
        base: usize,
        instruction: u32,
    ) -> Result<bool, RuntimeError> {
        if decode_c(instruction) != 1 {
            return Ok(false);
        }
        let receiver_index = base
            .checked_add(decode_b(instruction) as usize)
            .ok_or(RuntimeError::StackOverflow)?;
        let item_index = receiver_index
            .checked_add(1)
            .ok_or(RuntimeError::StackOverflow)?;
        let destination = base
            .checked_add(decode_a(instruction) as usize)
            .ok_or(RuntimeError::StackOverflow)?;
        let receiver = self
            .stack
            .get(receiver_index)
            .copied()
            .ok_or(RuntimeError::StackUnderflow)?;
        let item = self
            .stack
            .get(item_index)
            .copied()
            .ok_or(RuntimeError::StackUnderflow)?;
        if !receiver.is_list() {
            return Ok(false);
        }
        let handle = receiver
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("bad list handle"))?;
        let result = super::methods::list::push_value(self, handle, item)?;
        *self
            .stack
            .get_mut(destination)
            .ok_or(RuntimeError::StackOverflow)? = result;
        Ok(true)
    }

    pub(crate) fn execute_list_index_specialization(
        &mut self,
        base: usize,
        instruction: u32,
    ) -> Result<bool, RuntimeError> {
        let target_index = base
            .checked_add(decode_b(instruction) as usize)
            .ok_or(RuntimeError::StackOverflow)?;
        let key_index = base
            .checked_add(decode_c(instruction) as usize)
            .ok_or(RuntimeError::StackOverflow)?;
        let destination = base
            .checked_add(decode_a(instruction) as usize)
            .ok_or(RuntimeError::StackOverflow)?;
        let target = self
            .stack
            .get(target_index)
            .copied()
            .ok_or(RuntimeError::StackUnderflow)?;
        let key = self
            .stack
            .get(key_index)
            .copied()
            .ok_or(RuntimeError::StackUnderflow)?;
        if !target.is_list() {
            return Ok(false);
        }
        let handle = target
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("bad list handle"))?;
        let value = self.list_index_value(handle, key)?;
        *self
            .stack
            .get_mut(destination)
            .ok_or(RuntimeError::StackOverflow)? = value;
        Ok(true)
    }
}
