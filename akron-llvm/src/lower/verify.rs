use std::collections::VecDeque;

use akron::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c, decode_opcode};
use akron::{CompiledProgram, OpCode};
use memory::Value;

use super::LoweringError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueKind {
    Uninitialized,
    Int,
    Bool,
    Nil,
    Scalar,
}

pub(super) struct Verification {
    native: Vec<bool>,
}

impl Verification {
    pub(super) fn is_native(&self, instruction: usize) -> bool {
        self.native.get(instruction).copied().unwrap_or(false)
    }

    pub(super) fn native_count(&self) -> usize {
        self.native.iter().filter(|&&is_native| is_native).count()
    }
}

pub(super) fn verify(program: &CompiledProgram) -> Result<Verification, LoweringError> {
    let mut verification = Verification {
        native: vec![false; program.main.chunk.len()],
    };

    let chunk = &program.main.chunk;
    if chunk.is_empty() {
        return Ok(verification);
    }
    let register_count = program.main.max_slots as usize;
    let mut inputs: Vec<Option<Vec<ValueKind>>> = vec![None; chunk.len()];
    inputs[0] = Some(vec![ValueKind::Uninitialized; register_count]);
    let mut worklist = VecDeque::from([0usize]);

    while let Some(ip) = worklist.pop_front() {
        let mut state = inputs[ip].clone().ok_or_else(|| {
            LoweringError::Internal(format!("missing verifier state for instruction {ip}"))
        })?;
        let instruction = chunk[ip];
        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoweringError::unsupported(ip, "unknown opcode after program validation")
        })?;
        let transition = (|| -> Result<Vec<usize>, LoweringError> {
            let mut successors = Vec::with_capacity(2);
            match opcode {
                OpCode::LoadConst => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    let constant = program
                        .main
                        .constants
                        .get(decode_bx(instruction) as usize)
                        .ok_or_else(|| {
                            LoweringError::unsupported(ip, "constant is out of range")
                        })?;
                    state[destination] = scalar_kind(*constant).ok_or_else(|| {
                        LoweringError::unsupported(ip, "heap-backed constant is unsupported")
                    })?;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::LoadTrue | OpCode::LoadFalse => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    state[destination] = ValueKind::Bool;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::LoadNil => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    state[destination] = ValueKind::Nil;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Move => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    let source = initialized(&state, decode_b(instruction), ip)?;
                    state[destination] = state[source];
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    require_int(&state, decode_b(instruction), ip)?;
                    require_int(&state, decode_c(instruction), ip)?;
                    state[destination] = ValueKind::Int;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Neg => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    require_int(&state, decode_b(instruction), ip)?;
                    state[destination] = ValueKind::Int;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Eq | OpCode::NotEq => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    initialized(&state, decode_b(instruction), ip)?;
                    initialized(&state, decode_c(instruction), ip)?;
                    state[destination] = ValueKind::Bool;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Lt | OpCode::Gt | OpCode::Le | OpCode::Ge => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    require_int(&state, decode_b(instruction), ip)?;
                    require_int(&state, decode_c(instruction), ip)?;
                    state[destination] = ValueKind::Bool;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::LogNot => {
                    let destination = register(&state, decode_a(instruction), ip)?;
                    initialized(&state, decode_b(instruction), ip)?;
                    state[destination] = ValueKind::Bool;
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Jump => successors.push(decode_bx(instruction) as usize),
                OpCode::JumpIfFalse => {
                    initialized(&state, decode_a(instruction), ip)?;
                    successors.push(decode_bx(instruction) as usize);
                    fallthrough(ip, chunk.len(), &mut successors);
                }
                OpCode::Return => match decode_b(instruction) {
                    0 => {}
                    1 => {
                        initialized(&state, decode_a(instruction), ip)?;
                    }
                    flag => {
                        return Err(LoweringError::unsupported(
                            ip,
                            format!("invalid return-value flag {flag}"),
                        ));
                    }
                },
                OpCode::Nop => fallthrough(ip, chunk.len(), &mut successors),
                _ => {
                    return Err(LoweringError::unsupported(
                        ip,
                        format!("opcode {} requires interpreter fallback", opcode.name()),
                    ));
                }
            }
            Ok(successors)
        })();

        let successors = match transition {
            Ok(successors) => {
                verification.native[ip] = true;
                successors
            }
            Err(LoweringError::Unsupported { .. }) => continue,
            Err(error) => return Err(error),
        };

        for successor in successors {
            if successor >= chunk.len() {
                return Err(LoweringError::unsupported(
                    ip,
                    format!("successor {successor} is outside the function"),
                ));
            }
            if merge(&mut inputs[successor], &state) {
                worklist.push_back(successor);
            }
        }
    }
    Ok(verification)
}

fn scalar_kind(value: Value) -> Option<ValueKind> {
    if value.is_int() {
        Some(ValueKind::Int)
    } else if value.is_bool() {
        Some(ValueKind::Bool)
    } else if value.is_nil() {
        Some(ValueKind::Nil)
    } else {
        None
    }
}

fn register(state: &[ValueKind], register: u8, ip: usize) -> Result<usize, LoweringError> {
    let index = register as usize;
    if index >= state.len() {
        Err(LoweringError::unsupported(
            ip,
            format!("register R{register} exceeds max_slots {}", state.len()),
        ))
    } else {
        Ok(index)
    }
}

fn initialized(state: &[ValueKind], register: u8, ip: usize) -> Result<usize, LoweringError> {
    let index = self::register(state, register, ip)?;
    if state[index] == ValueKind::Uninitialized {
        Err(LoweringError::unsupported(
            ip,
            format!("R{register} is not initialized on every control-flow path"),
        ))
    } else {
        Ok(index)
    }
}

fn require_int(state: &[ValueKind], register: u8, ip: usize) -> Result<(), LoweringError> {
    let index = self::register(state, register, ip)?;
    if state[index] == ValueKind::Int {
        Ok(())
    } else {
        Err(LoweringError::unsupported(
            ip,
            format!("R{register} is not known to be an integer"),
        ))
    }
}

fn fallthrough(ip: usize, length: usize, successors: &mut Vec<usize>) {
    if ip + 1 < length {
        successors.push(ip + 1);
    }
}

fn merge(target: &mut Option<Vec<ValueKind>>, incoming: &[ValueKind]) -> bool {
    let Some(current) = target else {
        *target = Some(incoming.to_vec());
        return true;
    };
    let mut changed = false;
    for (current, incoming) in current.iter_mut().zip(incoming) {
        let merged = if *current == *incoming {
            *current
        } else if *current == ValueKind::Uninitialized || *incoming == ValueKind::Uninitialized {
            ValueKind::Uninitialized
        } else {
            ValueKind::Scalar
        };
        if merged != *current {
            *current = merged;
            changed = true;
        }
    }
    changed
}
