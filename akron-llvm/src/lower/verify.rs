use std::collections::VecDeque;

use akron::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c, decode_opcode};
use akron::{CompiledProgram, OpCode};
use memory::Function;

use super::LoweringError;
use crate::LlvmTierOptions;
use instruction::{transition, ListPushSite, ValueKind};

mod instruction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstructionMode {
    Direct,
    Runtime,
    Call(Option<u32>),
    SpecializationPreamble,
    ListPush,
    ListIndex,
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

    pub(super) fn runtime_count(&self, options: LlvmTierOptions) -> usize {
        self.all_modes()
            .filter(|mode| {
                **mode == InstructionMode::Runtime
                    || (!options.list_specializations
                        && matches!(
                            mode,
                            InstructionMode::SpecializationPreamble
                                | InstructionMode::ListPush
                                | InstructionMode::ListIndex
                        ))
            })
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
    let main = verify_function(&program.main, 0, None, &program.strings)?;
    let functions = program
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            verify_function(
                function,
                function.arity as usize,
                Some(index as u32),
                &program.strings,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Verification { main, functions })
}

fn verify_function(
    function: &Function,
    parameter_count: usize,
    self_prototype: Option<u32>,
    strings: &[String],
) -> Result<FunctionVerification, LoweringError> {
    let chunk = &function.chunk;
    let push_fusion = PushFusion::build(function, strings);
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
            push_fusion.site(ip),
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

struct PushFusion {
    preambles: Vec<bool>,
    methods: Vec<bool>,
}

impl PushFusion {
    fn site(&self, ip: usize) -> ListPushSite {
        if self.preambles[ip] {
            ListPushSite::Preamble
        } else if self.methods[ip] {
            ListPushSite::Method
        } else {
            ListPushSite::None
        }
    }

    fn build(function: &Function, strings: &[String]) -> Self {
        let mut plan = Self {
            preambles: vec![false; function.chunk.len()],
            methods: vec![false; function.chunk.len()],
        };
        let mut branch_targets = vec![false; function.chunk.len()];
        for &instruction in &function.chunk {
            let Some(opcode) = OpCode::from_u8(decode_opcode(instruction)) else {
                continue;
            };
            if matches!(opcode, OpCode::Jump | OpCode::JumpIfFalse) {
                if let Some(target) = branch_targets.get_mut(decode_bx(instruction) as usize) {
                    *target = true;
                }
            }
        }

        for (method_ip, targeted) in branch_targets.iter().copied().enumerate().skip(1) {
            let method = function.chunk[method_ip];
            if targeted
                || OpCode::from_u8(decode_opcode(method)) != Some(OpCode::MethodCall)
                || decode_c(method) != 1
                || decode_b(method) == 0
            {
                continue;
            }
            let preamble_ip = method_ip - 1;
            let preamble = function.chunk[preamble_ip];
            if OpCode::from_u8(decode_opcode(preamble)) != Some(OpCode::LoadConst)
                || decode_a(preamble) != decode_b(method) - 1
            {
                continue;
            }
            let Some(constant) = function.constants.get(decode_bx(preamble) as usize) else {
                continue;
            };
            let Some(string_index) = constant.as_handle() else {
                continue;
            };
            if !constant.is_string()
                || strings.get(string_index as usize).map(String::as_str) != Some("push")
            {
                continue;
            }
            plan.preambles[preamble_ip] = true;
            plan.methods[method_ip] = true;
        }
        plan
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
