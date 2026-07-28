use std::collections::VecDeque;

use akron::opcode::instruction::decode_opcode;
use akron::{CompiledProgram, OpCode};
use memory::Function;

use super::LoweringError;
use instruction::{transition, ValueKind};

mod instruction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstructionMode {
    Direct,
    Runtime,
    Call(Option<u32>),
    Bailout,
}

pub(super) struct FunctionVerification {
    modes: Vec<InstructionMode>,
}

impl FunctionVerification {
    pub(super) fn mode(&self, instruction: usize) -> InstructionMode {
        self.modes
            .get(instruction)
            .copied()
            .unwrap_or(InstructionMode::Bailout)
    }

    pub(super) fn is_direct(&self, instruction: usize) -> bool {
        self.mode(instruction) == InstructionMode::Direct
    }
}

pub(super) struct Verification {
    main: FunctionVerification,
    functions: Vec<FunctionVerification>,
}

impl Verification {
    pub(super) fn main(&self) -> &FunctionVerification {
        &self.main
    }

    pub(super) fn function(&self, index: usize) -> &FunctionVerification {
        &self.functions[index]
    }

    pub(super) fn native_count(&self) -> usize {
        self.all_modes()
            .filter(|mode| **mode != InstructionMode::Bailout)
            .count()
    }

    pub(super) fn direct_count(&self) -> usize {
        self.all_modes()
            .filter(|mode| **mode == InstructionMode::Direct)
            .count()
    }

    pub(super) fn runtime_count(&self) -> usize {
        self.all_modes()
            .filter(|mode| **mode == InstructionMode::Runtime)
            .count()
    }

    pub(super) fn call_count(&self) -> usize {
        self.all_modes()
            .filter(|mode| matches!(mode, InstructionMode::Call(_)))
            .count()
    }

    fn all_modes(&self) -> impl Iterator<Item = &InstructionMode> {
        self.main.modes.iter().chain(
            self.functions
                .iter()
                .flat_map(|function| function.modes.iter()),
        )
    }
}

pub(super) fn verify(program: &CompiledProgram) -> Result<Verification, LoweringError> {
    let main = verify_function(&program.main, 0, None)?;
    let functions = program
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            verify_function(function, function.arity as usize, Some(index as u32))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Verification { main, functions })
}

fn verify_function(
    function: &Function,
    parameter_count: usize,
    self_prototype: Option<u32>,
) -> Result<FunctionVerification, LoweringError> {
    let chunk = &function.chunk;
    let mut verification = FunctionVerification {
        modes: vec![InstructionMode::Bailout; chunk.len()],
    };
    if chunk.is_empty() {
        return Ok(verification);
    }

    let mut initial = vec![ValueKind::Uninitialized; function.max_slots as usize];
    for register in initial.iter_mut().take(parameter_count) {
        *register = ValueKind::Scalar;
    }
    let mut inputs: Vec<Option<Vec<ValueKind>>> = vec![None; chunk.len()];
    inputs[0] = Some(initial);
    let mut worklist = VecDeque::from([0usize]);

    while let Some(ip) = worklist.pop_front() {
        let mut state = inputs[ip].clone().ok_or_else(|| {
            LoweringError::Internal(format!("missing verifier state for instruction {ip}"))
        })?;
        let instruction = chunk[ip];
        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoweringError::unsupported(ip, "unknown opcode after program validation")
        })?;
        let transition = transition(
            function,
            ip,
            instruction,
            opcode,
            &mut state,
            self_prototype,
        );
        let (mode, successors) = match transition {
            Ok(result) => result,
            Err(LoweringError::Unsupported { .. }) => continue,
            Err(error) => return Err(error),
        };
        verification.modes[ip] = mode;

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
