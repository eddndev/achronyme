use memory::Function;

use crate::loader::LoaderError;
use crate::opcode::instruction::{decode_bx, decode_opcode};
use crate::opcode::OpCode;

pub(super) fn validate_reachable_returns(
    function: &Function,
    context: &str,
) -> Result<(), LoaderError> {
    let mut pending = vec![0usize];
    let mut visited = vec![false; function.chunk.len()];

    while let Some(ip) = pending.pop() {
        if ip >= function.chunk.len() {
            return Err(LoaderError::Format(format!(
                "{context} reachable path exits without Return at ip {ip}"
            )));
        }
        if visited[ip] {
            continue;
        }
        visited[ip] = true;

        let instruction = function.chunk[ip];
        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoaderError::Format(format!("{context} has invalid control flow at ip {ip}"))
        })?;
        match opcode {
            OpCode::Return => {}
            OpCode::Jump => pending.push(decode_bx(instruction) as usize),
            OpCode::JumpIfFalse | OpCode::ForIter => {
                pending.push(decode_bx(instruction) as usize);
                pending.push(ip + 1);
            }
            _ => pending.push(ip + 1),
        }
    }

    Ok(())
}
