use akron::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c};
use akron::OpCode;
use memory::{Function, Value};

use super::InstructionMode;
use crate::lower::LoweringError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueKind {
    Uninitialized,
    Int,
    Bool,
    Nil,
    Scalar,
    Prototype(u32),
}

pub(super) fn transition(
    function: &Function,
    ip: usize,
    instruction: u32,
    opcode: OpCode,
    state: &mut [ValueKind],
    self_prototype: Option<u32>,
) -> Result<(InstructionMode, Vec<usize>), LoweringError> {
    let mut successors = Vec::with_capacity(2);
    let mode = match opcode {
        OpCode::LoadConst => {
            let destination = register(state, decode_a(instruction), ip)?;
            let constant = function
                .constants
                .get(decode_bx(instruction) as usize)
                .ok_or_else(|| LoweringError::unsupported(ip, "constant is out of range"))?;
            match scalar_kind(*constant) {
                Some(kind) => {
                    state[destination] = kind;
                    InstructionMode::Direct
                }
                None => {
                    state[destination] = ValueKind::Scalar;
                    InstructionMode::Runtime
                }
            }
        }
        OpCode::LoadTrue | OpCode::LoadFalse => {
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Bool;
            InstructionMode::Direct
        }
        OpCode::LoadNil => {
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Nil;
            InstructionMode::Direct
        }
        OpCode::Move => {
            let destination = register(state, decode_a(instruction), ip)?;
            let source = initialized(state, decode_b(instruction), ip)?;
            state[destination] = state[source];
            InstructionMode::Direct
        }
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => {
            require_int(state, decode_b(instruction), ip)?;
            require_int(state, decode_c(instruction), ip)?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Int;
            InstructionMode::Direct
        }
        OpCode::Neg => {
            require_int(state, decode_b(instruction), ip)?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Int;
            InstructionMode::Direct
        }
        OpCode::Eq | OpCode::NotEq => {
            let left = initialized(state, decode_b(instruction), ip)?;
            let right = initialized(state, decode_c(instruction), ip)?;
            let immediate_equality = is_immediate(state[left]) && is_immediate(state[right]);
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Bool;
            if immediate_equality {
                InstructionMode::Direct
            } else {
                InstructionMode::Runtime
            }
        }
        OpCode::Lt | OpCode::Gt | OpCode::Le | OpCode::Ge => {
            require_int(state, decode_b(instruction), ip)?;
            require_int(state, decode_c(instruction), ip)?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Bool;
            InstructionMode::Direct
        }
        OpCode::LogNot => {
            initialized(state, decode_b(instruction), ip)?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Bool;
            InstructionMode::Direct
        }
        OpCode::Jump => {
            successors.push(decode_bx(instruction) as usize);
            return Ok((InstructionMode::Direct, successors));
        }
        OpCode::JumpIfFalse => {
            initialized(state, decode_a(instruction), ip)?;
            successors.push(decode_bx(instruction) as usize);
            fallthrough(ip, function.chunk.len(), &mut successors);
            return Ok((InstructionMode::Direct, successors));
        }
        OpCode::Return => {
            match decode_b(instruction) {
                0 => {}
                1 => {
                    initialized(state, decode_a(instruction), ip)?;
                }
                flag => {
                    return Err(LoweringError::unsupported(
                        ip,
                        format!("invalid return-value flag {flag}"),
                    ));
                }
            }
            return Ok((InstructionMode::Direct, successors));
        }
        OpCode::Closure => {
            state[register(state, decode_a(instruction), ip)?] =
                ValueKind::Prototype(u32::from(decode_bx(instruction)));
            InstructionMode::Runtime
        }
        OpCode::GetGlobal => {
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Direct
        }
        OpCode::DefGlobalVar | OpCode::DefGlobalLet | OpCode::SetGlobal => {
            initialized(state, decode_a(instruction), ip)?;
            InstructionMode::Direct
        }
        OpCode::Print => {
            initialized(state, decode_a(instruction), ip)?;
            InstructionMode::Runtime
        }
        OpCode::GetUpvalue => {
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Runtime
        }
        OpCode::SetUpvalue | OpCode::CloseUpvalue => {
            initialized(state, decode_a(instruction), ip)?;
            InstructionMode::Runtime
        }
        OpCode::BuildList => {
            initialized_range(
                state,
                decode_b(instruction) as usize,
                decode_c(instruction) as usize,
                ip,
            )?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Runtime
        }
        OpCode::BuildMap => {
            initialized_range(
                state,
                decode_b(instruction) as usize,
                (decode_c(instruction) as usize).saturating_mul(2),
                ip,
            )?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Runtime
        }
        OpCode::GetIndex => {
            initialized(state, decode_b(instruction), ip)?;
            initialized(state, decode_c(instruction), ip)?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Runtime
        }
        OpCode::SetIndex => {
            initialized(state, decode_a(instruction), ip)?;
            initialized(state, decode_b(instruction), ip)?;
            initialized(state, decode_c(instruction), ip)?;
            InstructionMode::Runtime
        }
        OpCode::MethodCall => {
            let receiver = decode_b(instruction);
            initialized(state, receiver.wrapping_sub(1), ip)?;
            initialized_range(
                state,
                receiver as usize,
                decode_c(instruction) as usize + 1,
                ip,
            )?;
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Runtime
        }
        OpCode::Call => {
            let callee = initialized(state, decode_b(instruction), ip)?;
            initialized_range(
                state,
                decode_b(instruction) as usize,
                decode_c(instruction) as usize + 1,
                ip,
            )?;
            let expected_prototype = match state[callee] {
                ValueKind::Prototype(prototype) => Some(prototype),
                _ => self_prototype,
            };
            state[register(state, decode_a(instruction), ip)?] = ValueKind::Scalar;
            InstructionMode::Call(expected_prototype)
        }
        OpCode::Nop => InstructionMode::Direct,
        _ => {
            return Err(LoweringError::unsupported(
                ip,
                format!("opcode {} requires interpreter fallback", opcode.name()),
            ));
        }
    };
    fallthrough(ip, function.chunk.len(), &mut successors);
    Ok((mode, successors))
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

fn is_immediate(kind: ValueKind) -> bool {
    matches!(kind, ValueKind::Int | ValueKind::Bool | ValueKind::Nil)
}

fn register(state: &[ValueKind], register: u8, ip: usize) -> Result<usize, LoweringError> {
    let index = register as usize;
    if index < state.len() {
        Ok(index)
    } else {
        Err(LoweringError::unsupported(
            ip,
            format!("register R{register} exceeds max_slots {}", state.len()),
        ))
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

fn initialized_range(
    state: &[ValueKind],
    start: usize,
    count: usize,
    ip: usize,
) -> Result<(), LoweringError> {
    let end = start
        .checked_add(count)
        .filter(|&end| end <= state.len())
        .ok_or_else(|| LoweringError::unsupported(ip, "register range exceeds max_slots"))?;
    for (register, kind) in state.iter().enumerate().take(end).skip(start) {
        if *kind == ValueKind::Uninitialized {
            return Err(LoweringError::unsupported(
                ip,
                format!("R{register} is not initialized on every control-flow path"),
            ));
        }
    }
    Ok(())
}

fn require_int(state: &[ValueKind], register: u8, ip: usize) -> Result<(), LoweringError> {
    let index = self::register(state, register, ip)?;
    if matches!(state[index], ValueKind::Int | ValueKind::Scalar) {
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
