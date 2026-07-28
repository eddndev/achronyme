use akron::opcode::instruction::{decode_bx, decode_opcode};
use akron::{CompiledProgram, OpCode};

use super::verify::Verification;
use super::LoweringError;

pub(super) struct BlockPlan {
    starts: Vec<bool>,
    ends: Vec<usize>,
}

impl BlockPlan {
    pub(super) fn build(
        program: &CompiledProgram,
        verification: &Verification,
    ) -> Result<Self, LoweringError> {
        let chunk = &program.main.chunk;
        let mut starts = vec![false; chunk.len()];
        if chunk.is_empty() {
            return Ok(Self {
                starts,
                ends: Vec::new(),
            });
        }
        starts[0] = true;

        for (ip, &instruction) in chunk.iter().enumerate() {
            if !verification.is_native(ip) {
                starts[ip] = true;
                mark_fallthrough(&mut starts, ip);
                continue;
            }

            let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
                LoweringError::Internal(format!("unknown opcode in block plan at {ip}"))
            })?;
            match opcode {
                OpCode::Jump => mark_target(&mut starts, decode_bx(instruction), ip)?,
                OpCode::JumpIfFalse => {
                    mark_target(&mut starts, decode_bx(instruction), ip)?;
                    mark_fallthrough(&mut starts, ip);
                }
                OpCode::Return => mark_fallthrough(&mut starts, ip),
                _ => {}
            }
        }

        let mut ends = vec![chunk.len(); chunk.len()];
        let mut start = 0;
        for next in 1..=chunk.len() {
            if next == chunk.len() || starts[next] {
                ends[start..next].fill(next);
                start = next;
            }
        }
        Ok(Self { starts, ends })
    }

    pub(super) fn is_start(&self, instruction: usize) -> bool {
        self.starts.get(instruction).copied().unwrap_or(false)
    }

    pub(super) fn end(&self, instruction: usize) -> usize {
        self.ends
            .get(instruction)
            .copied()
            .unwrap_or(self.ends.len())
    }
}

fn mark_fallthrough(starts: &mut [bool], instruction: usize) {
    if let Some(start) = starts.get_mut(instruction + 1) {
        *start = true;
    }
}

fn mark_target(starts: &mut [bool], target: u16, instruction: usize) -> Result<(), LoweringError> {
    let target = target as usize;
    let start = starts.get_mut(target).ok_or_else(|| {
        LoweringError::Internal(format!(
            "validated jump at {instruction} has invalid target {target}"
        ))
    })?;
    *start = true;
    Ok(())
}
