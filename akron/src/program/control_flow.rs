use memory::Function;

use crate::loader::LoaderError;
use crate::opcode::instruction::{decode_bx, decode_opcode};
use crate::opcode::OpCode;

pub(super) fn validate_reachable_returns(
    function: &Function,
    context: &str,
) -> Result<(), LoaderError> {
    let mut pending = vec![(0usize, 0u16)];
    let mut visited = vec![None; function.chunk.len()];

    while let Some((ip, scope_depth)) = pending.pop() {
        if ip >= function.chunk.len() {
            return Err(LoaderError::Format(format!(
                "{context} reachable path exits without Return at ip {ip}"
            )));
        }
        if let Some(previous_depth) = visited[ip] {
            if previous_depth != scope_depth {
                return Err(LoaderError::Format(format!(
                    "{context} reaches ip {ip} with inconsistent task scope depths \
                     {previous_depth} and {scope_depth}"
                )));
            }
            continue;
        }
        visited[ip] = Some(scope_depth);

        let instruction = function.chunk[ip];
        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoaderError::Format(format!("{context} has invalid control flow at ip {ip}"))
        })?;
        match opcode {
            OpCode::Return if scope_depth != 0 => {
                return Err(LoaderError::Format(format!(
                    "{context} returns with {scope_depth} open task scope(s) at ip {ip}"
                )));
            }
            OpCode::Return => {}
            OpCode::ScopeEnter => {
                let next_depth = scope_depth.checked_add(1).ok_or_else(|| {
                    LoaderError::Format(format!("{context} task scope depth overflows at ip {ip}"))
                })?;
                pending.push((ip + 1, next_depth));
            }
            OpCode::ScopeExit => {
                let next_depth = scope_depth.checked_sub(1).ok_or_else(|| {
                    LoaderError::Format(format!(
                        "{context} exits a task scope before entering one at ip {ip}"
                    ))
                })?;
                pending.push((ip + 1, next_depth));
            }
            OpCode::Jump => pending.push((decode_bx(instruction) as usize, scope_depth)),
            OpCode::JumpIfFalse | OpCode::ForIter => {
                pending.push((decode_bx(instruction) as usize, scope_depth));
                pending.push((ip + 1, scope_depth));
            }
            _ => pending.push((ip + 1, scope_depth)),
        }
    }

    Ok(())
}
