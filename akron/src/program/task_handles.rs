use std::collections::VecDeque;

use memory::Function;

use crate::loader::LoaderError;
use crate::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c, decode_opcode};
use crate::opcode::OpCode;

pub(super) fn validate_non_escaping(
    function: &Function,
    prototypes: &[Function],
    context: &str,
) -> Result<(), LoaderError> {
    if function.chunk.is_empty() {
        return Ok(());
    }
    let slots = function.max_slots as usize;
    let mut incoming = vec![None::<Vec<bool>>; function.chunk.len()];
    incoming[0] = Some(vec![false; slots]);
    let mut pending = VecDeque::from([0usize]);

    while let Some(ip) = pending.pop_front() {
        let mut tasks = incoming[ip]
            .clone()
            .expect("queued task-handle state must exist");
        let instruction = function.chunk[ip];
        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoaderError::Format(format!("{context} has invalid task dataflow at ip {ip}"))
        })?;
        transfer(opcode, instruction, &mut tasks, prototypes, context, ip)?;

        for successor in successors(opcode, instruction, ip) {
            if successor >= function.chunk.len() {
                continue;
            }
            let changed = match &mut incoming[successor] {
                Some(previous) => {
                    let mut changed = false;
                    for (old, new) in previous.iter_mut().zip(&tasks) {
                        if *new && !*old {
                            *old = true;
                            changed = true;
                        }
                    }
                    changed
                }
                slot @ None => {
                    *slot = Some(tasks.clone());
                    true
                }
            };
            if changed {
                pending.push_back(successor);
            }
        }
    }
    Ok(())
}

fn transfer(
    opcode: OpCode,
    instruction: u32,
    tasks: &mut [bool],
    prototypes: &[Function],
    context: &str,
    ip: usize,
) -> Result<(), LoaderError> {
    let a = decode_a(instruction) as usize;
    let b = decode_b(instruction) as usize;
    let c = decode_c(instruction) as usize;
    let escape = |operation: &str| {
        LoaderError::Format(format!(
            "{context} task handle escapes through {operation} at ip {ip}"
        ))
    };

    match opcode {
        OpCode::Spawn => {
            reject_task_range(tasks, b, c + 1, "Spawn argument", context, ip)?;
            tasks[a] = true;
        }
        OpCode::Await | OpCode::AwaitOutcome => {
            if !tasks[b] {
                return Err(LoaderError::Format(format!(
                    "{context} {opcode} reads a non-task register R{b} at ip {ip}"
                )));
            }
            tasks[b] = false;
            tasks[a] = false;
        }
        OpCode::TaskForget => {
            if !tasks[a] {
                return Err(LoaderError::Format(format!(
                    "{context} TaskForget reads a non-task register R{a} at ip {ip}"
                )));
            }
            tasks[a] = false;
        }
        OpCode::AwaitRace => {
            if c < 2
                || tasks
                    .get(b..b.saturating_add(c))
                    .is_none_or(|range| range.iter().any(|is_task| !*is_task))
            {
                return Err(LoaderError::Format(format!(
                    "{context} AwaitRace reads a non-task register at ip {ip}"
                )));
            }
            for task in &mut tasks[b..b + c] {
                *task = false;
            }
            tasks[a] = false;
        }
        OpCode::Move => tasks[a] = tasks[b],
        OpCode::Return if decode_b(instruction) == 1 && tasks[a] => {
            return Err(escape("Return"));
        }
        OpCode::DefGlobalVar | OpCode::DefGlobalLet | OpCode::SetGlobal if tasks[a] => {
            return Err(escape(opcode.name()));
        }
        OpCode::SetUpvalue if tasks[a] => return Err(escape("SetUpvalue")),
        OpCode::BuildList => {
            reject_task_range(tasks, b, c, "BuildList", context, ip)?;
            tasks[a] = false;
        }
        OpCode::BuildMap => {
            reject_task_range(tasks, b, c * 2, "BuildMap", context, ip)?;
            tasks[a] = false;
        }
        OpCode::SetIndex if tasks[c] => return Err(escape("SetIndex")),
        OpCode::Call | OpCode::MethodCall => {
            reject_task_range(tasks, b, c + 1, opcode.name(), context, ip)?;
            tasks[a] = false;
        }
        OpCode::Closure => {
            let prototype = prototypes
                .get(decode_bx(instruction) as usize)
                .ok_or_else(|| {
                    LoaderError::Format(format!("{context} has invalid Closure at ip {ip}"))
                })?;
            for capture in prototype.upvalue_info.chunks_exact(2) {
                if capture[0] == 1 {
                    let register = capture[1] as usize;
                    if register >= tasks.len() {
                        return Err(LoaderError::Format(format!(
                            "{context} Closure captures out-of-range R{register} at ip {ip}"
                        )));
                    }
                    if tasks[register] {
                        return Err(escape("Closure capture"));
                    }
                }
            }
            tasks[a] = false;
        }
        OpCode::LoadConst
        | OpCode::LoadTrue
        | OpCode::LoadFalse
        | OpCode::LoadNil
        | OpCode::Add
        | OpCode::Sub
        | OpCode::Mul
        | OpCode::Div
        | OpCode::Mod
        | OpCode::Pow
        | OpCode::Neg
        | OpCode::Eq
        | OpCode::Lt
        | OpCode::Gt
        | OpCode::NotEq
        | OpCode::Le
        | OpCode::Ge
        | OpCode::LogNot
        | OpCode::GetUpvalue
        | OpCode::GetGlobal
        | OpCode::GetIndex
        | OpCode::GetIter
        | OpCode::Prove
        | OpCode::CallCircomTemplate => tasks[a] = false,
        OpCode::ForIter => {
            tasks[a] = false;
            if a + 1 < tasks.len() {
                tasks[a + 1] = false;
            }
        }
        OpCode::Return
        | OpCode::SetUpvalue
        | OpCode::CloseUpvalue
        | OpCode::Jump
        | OpCode::JumpIfFalse
        | OpCode::ScopeEnter
        | OpCode::ScopeExit
        | OpCode::CancelCheck
        | OpCode::DefGlobalVar
        | OpCode::DefGlobalLet
        | OpCode::SetGlobal
        | OpCode::Print
        | OpCode::SetIndex
        | OpCode::Nop => {}
    }
    Ok(())
}

fn reject_task_range(
    tasks: &[bool],
    start: usize,
    count: usize,
    operation: &str,
    context: &str,
    ip: usize,
) -> Result<(), LoaderError> {
    if tasks
        .get(start..start.saturating_add(count))
        .is_some_and(|registers| registers.iter().any(|is_task| *is_task))
    {
        Err(LoaderError::Format(format!(
            "{context} task handle escapes through {operation} at ip {ip}"
        )))
    } else {
        Ok(())
    }
}

fn successors(opcode: OpCode, instruction: u32, ip: usize) -> Vec<usize> {
    match opcode {
        OpCode::Return => Vec::new(),
        OpCode::Jump => vec![decode_bx(instruction) as usize],
        OpCode::JumpIfFalse | OpCode::ForIter => {
            vec![decode_bx(instruction) as usize, ip + 1]
        }
        _ => vec![ip + 1],
    }
}
