use std::fmt::Write;

use akron::opcode::instruction::{decode_a, decode_b, decode_bx, decode_opcode};
use akron::OpCode;
use memory::Function;

use super::blocks::BlockPlan;
use super::module_ir::module_header;
use super::verify::{FunctionVerification, InstructionMode, Verification};
use super::LoweringError;

const NIL_BITS: u64 = 1u64 << 60;
const FALSE_BITS: u64 = 2u64 << 60;
const TRUE_BITS: u64 = 3u64 << 60;
mod poll;
mod specialize;

pub(super) fn emit(
    program: &akron::CompiledProgram,
    verification: &Verification,
    options: crate::LlvmTierOptions,
) -> Result<String, LoweringError> {
    let mut output = String::with_capacity(program.instruction_count() * 500 + 4096);
    module_header(&mut output);
    emit_function(
        &mut output,
        &program.main,
        verification.main(),
        "akron_compiled_main",
        program.functions.len(),
        options,
    )?;
    for (index, function) in program.functions.iter().enumerate() {
        emit_function(
            &mut output,
            function,
            verification.function(index),
            &format!("akron_compiled_fn_{index}"),
            program.functions.len(),
            options,
        )?;
    }
    Ok(output)
}

fn emit_function(
    output: &mut String,
    function: &Function,
    verification: &FunctionVerification,
    symbol: &str,
    prototype_count: usize,
    options: crate::LlvmTierOptions,
) -> Result<(), LoweringError> {
    let blocks = BlockPlan::build(function, verification)?;
    let mut emitter = Emitter::new(
        output,
        function.chunk.len(),
        function.max_slots,
        blocks,
        prototype_count,
        options,
    );
    emitter.header(symbol);
    if function.chunk.is_empty() {
        emitter.line("  br label %finish");
    } else {
        emitter.line("  br label %ip0");
    }

    for (ip, &instruction) in function.chunk.iter().enumerate() {
        match verification.mode(ip) {
            InstructionMode::Bailout => {
                emitter.bailout(ip);
                continue;
            }
            InstructionMode::Runtime => {
                emitter.begin_instruction(ip, false);
                emitter.runtime_instruction(ip);
                continue;
            }
            InstructionMode::Call(expected_prototype) => {
                emitter.begin_instruction(ip, false);
                emitter.call_instruction(
                    ip,
                    instruction,
                    options.known_calls.then_some(expected_prototype).flatten(),
                );
                continue;
            }
            InstructionMode::SpecializationPreamble => {
                emitter.begin_instruction(ip, false);
                if options.list_specializations {
                    emitter.next(ip);
                } else {
                    emitter.runtime_instruction(ip);
                }
                continue;
            }
            InstructionMode::ListPush => {
                emitter.begin_instruction(ip, false);
                if options.list_specializations {
                    emitter.list_push_instruction(ip, instruction, ip - 1);
                } else {
                    emitter.runtime_instruction(ip);
                }
                continue;
            }
            InstructionMode::ListIndex => {
                emitter.begin_instruction(ip, false);
                if options.list_specializations {
                    emitter.list_index_instruction(ip, instruction);
                } else {
                    emitter.runtime_instruction(ip);
                }
                continue;
            }
            InstructionMode::Direct => {}
        }

        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoweringError::Internal(format!("unknown opcode reached emitter at {ip}"))
        })?;
        emitter.begin_instruction(ip, true);
        match opcode {
            OpCode::LoadConst => {
                let value = function.constants[decode_bx(instruction) as usize];
                emitter.store(decode_a(instruction), &value.to_abi_bits().to_string());
                emitter.next(ip);
            }
            OpCode::LoadTrue => {
                emitter.store(decode_a(instruction), &TRUE_BITS.to_string());
                emitter.next(ip);
            }
            OpCode::LoadFalse => {
                emitter.store(decode_a(instruction), &FALSE_BITS.to_string());
                emitter.next(ip);
            }
            OpCode::LoadNil => {
                emitter.store(decode_a(instruction), &NIL_BITS.to_string());
                emitter.next(ip);
            }
            OpCode::Move => {
                let value = emitter.load(decode_b(instruction));
                emitter.store(decode_a(instruction), &value);
                emitter.next(ip);
            }
            OpCode::DefGlobalVar | OpCode::DefGlobalLet => emitter.define_global(
                ip,
                decode_a(instruction),
                decode_bx(instruction),
                opcode == OpCode::DefGlobalVar,
            ),
            OpCode::GetGlobal => {
                emitter.get_global(ip, decode_a(instruction), decode_bx(instruction));
            }
            OpCode::SetGlobal => {
                emitter.set_global(ip, decode_a(instruction), decode_bx(instruction));
            }
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => {
                emitter.binary_integer(ip, instruction, opcode);
            }
            OpCode::Neg => emitter.negate_integer(ip, instruction),
            OpCode::Eq | OpCode::NotEq | OpCode::Lt | OpCode::Gt | OpCode::Le | OpCode::Ge => {
                emitter.comparison(ip, instruction, opcode);
            }
            OpCode::LogNot => emitter.logical_not(ip, instruction),
            OpCode::Jump => {
                emitter.line(&format!("  br label %ip{}", decode_bx(instruction)));
            }
            OpCode::JumpIfFalse => emitter.jump_if_false(ip, instruction),
            OpCode::Return => emitter.return_value(ip, instruction),
            OpCode::Nop => emitter.next(ip),
            _ => {
                return Err(LoweringError::Internal(format!(
                    "unsupported opcode {} reached direct emitter",
                    opcode.name()
                )));
            }
        }
    }

    emitter.line("finish:");
    if function.chunk.is_empty() {
        emitter.line(&format!("  store i64 {NIL_BITS}, ptr %result_out, align 8"));
        emitter.line("  ret i32 0");
    } else {
        emitter.finish_with_register_zero();
    }
    emitter.line("}\n");
    Ok(())
}

pub(super) struct Emitter<'output> {
    output: &'output mut String,
    instruction_count: usize,
    register_count: u16,
    pub(super) blocks: BlockPlan,
    pub(super) prototype_count: usize,
    pub(super) options: crate::LlvmTierOptions,
}

impl<'output> Emitter<'output> {
    fn new(
        output: &'output mut String,
        instruction_count: usize,
        register_count: u16,
        blocks: BlockPlan,
        prototype_count: usize,
        options: crate::LlvmTierOptions,
    ) -> Self {
        Self {
            output,
            instruction_count,
            register_count,
            blocks,
            prototype_count,
            options,
        }
    }

    pub(super) fn line(&mut self, line: &str) {
        writeln!(self.output, "{line}").expect("write to String");
    }

    fn header(&mut self, symbol: &str) {
        self.line(&format!(
            "define i32 @{symbol}(ptr %api, ptr %context, i32 %frame_index, i32 %base, ptr %result_out) {{"
        ));
        self.line("entry:");
        self.line("  %window.out = alloca ptr, align 8");
        self.line("  %globals.out = alloca ptr, align 8");
        self.line("  %globals.len.out = alloca i32, align 4");
        self.line("  %call.frame.out = alloca i32, align 4");
        self.line("  %call.base.out = alloca i32, align 4");
        self.line("  %call.prototype.out = alloca i32, align 4");
        self.line("  %call.result.out = alloca i64, align 8");
        for register in 0..self.register_count {
            self.line(&format!("  %reg.{register} = alloca i64, align 8"));
        }
        for (field, name) in [
            (7, "raise"),
            (8, "bailout"),
            (10, "poll_block"),
            (11, "refund_block"),
            (12, "execute_instruction"),
            (13, "prepare_call"),
            (14, "finish_call"),
            (15, "poll_tier1_block"),
            (16, "execution_window"),
        ] {
            self.load_api_function(field, name);
        }
        if self.options.fast_poll {
            self.load_api_function(17, "poll_fast_block");
        }
        if self.options.known_calls {
            self.load_api_function(18, "prepare_known_call");
        }
        if self.options.list_specializations {
            self.load_api_function(19, "list_push");
            self.load_api_function(20, "list_index");
        }
        self.line(&format!(
            "  %window.status = call i32 %execution_window_fn(ptr %context, i32 %base, i32 {}, ptr %window.out, ptr %globals.out, ptr %globals.len.out)",
            self.register_count
        ));
        self.line("  %window.ok = icmp eq i32 %window.status, 0");
        self.line("  br i1 %window.ok, label %window.ready, label %window.error");
        self.line("window.error:");
        self.line("  ret i32 %window.status");
        self.line("window.ready:");
        self.line("  %window = load ptr, ptr %window.out, align 8");
        self.line("  %globals = load ptr, ptr %globals.out, align 8");
        self.line("  %globals.len = load i32, ptr %globals.len.out, align 4");
        self.reload("initial");
    }

    fn load_api_function(&mut self, field: u8, name: &str) {
        self.line(&format!(
            "  %{name}_slot = getelementptr %RuntimeApi, ptr %api, i32 0, i32 {field}"
        ));
        self.line(&format!(
            "  %{name}_fn = load ptr, ptr %{name}_slot, align 8"
        ));
    }

    fn bailout(&mut self, ip: usize) {
        self.line(&format!("ip{ip}:"));
        self.spill(&format!("bailout.{ip}"));
        self.line(&format!(
            "  %bailout.status.{ip} = call i32 %bailout_fn(ptr %context, i32 %frame_index, i32 {ip}, ptr %result_out)"
        ));
        self.line(&format!("  ret i32 %bailout.status.{ip}"));
    }

    pub(super) fn load(&mut self, register: u8) -> String {
        let value = format!("%value.{}", self.output.len());
        self.line(&format!(
            "  {value} = load i64, ptr %reg.{register}, align 8"
        ));
        value
    }

    pub(super) fn store(&mut self, register: u8, value: &str) {
        self.line(&format!(
            "  store i64 {value}, ptr %reg.{register}, align 8"
        ));
    }

    pub(super) fn next(&mut self, ip: usize) {
        self.line(&format!("  br label %{}", self.next_label(ip)));
    }

    pub(super) fn next_label(&self, ip: usize) -> String {
        let next = ip + 1;
        if next >= self.instruction_count {
            "finish".to_string()
        } else if self.blocks.is_start(next) {
            format!("ip{next}")
        } else {
            format!("op{next}")
        }
    }

    pub(super) fn spill(&mut self, site: &str) {
        for register in 0..self.register_count {
            self.line(&format!(
                "  %spill.value.{site}.{register} = load i64, ptr %reg.{register}, align 8"
            ));
            self.line(&format!(
                "  %spill.slot.{site}.{register} = getelementptr i64, ptr %window, i32 {register}"
            ));
            self.line(&format!(
                "  store i64 %spill.value.{site}.{register}, ptr %spill.slot.{site}.{register}, align 8"
            ));
        }
    }

    pub(super) fn reload(&mut self, site: &str) {
        for register in 0..self.register_count {
            self.line(&format!(
                "  %reload.slot.{site}.{register} = getelementptr i64, ptr %window, i32 {register}"
            ));
            self.line(&format!(
                "  %reload.value.{site}.{register} = load i64, ptr %reload.slot.{site}.{register}, align 8"
            ));
            self.line(&format!(
                "  store i64 %reload.value.{site}.{register}, ptr %reg.{register}, align 8"
            ));
        }
    }

    pub(super) fn raise_block(&mut self, label: &str, ip: usize, code: u32) {
        let clean = label.replace('.', "_");
        let refund = self.blocks.end(ip).saturating_sub(ip + 1);
        self.line(&format!("{label}:"));
        self.spill(&format!("error.{clean}"));
        self.refund(&clean, ip + 1, refund);
        self.line(&format!("raise.{clean}:"));
        self.line(&format!(
            "  %raise.status.{clean} = call i32 %raise_fn(ptr %context, i32 {code})"
        ));
        self.line(&format!("  ret i32 %raise.status.{clean}"));
    }

    pub(super) fn refund(&mut self, site: &str, next_ip: usize, count: usize) {
        self.line(&format!(
            "  %refund.status.{site} = call i32 %refund_block_fn(ptr %context, i32 %frame_index, i32 {next_ip}, i32 {count})"
        ));
        self.line(&format!(
            "  %refund.ok.{site} = icmp eq i32 %refund.status.{site}, 0"
        ));
        self.line(&format!(
            "  br i1 %refund.ok.{site}, label %raise.{site}, label %refund.error.{site}"
        ));
        self.line(&format!("refund.error.{site}:"));
        self.line(&format!("  ret i32 %refund.status.{site}"));
    }

    fn return_value(&mut self, ip: usize, instruction: u32) {
        let value = if decode_b(instruction) == 1 {
            self.load(decode_a(instruction))
        } else {
            NIL_BITS.to_string()
        };
        self.spill(&format!("return.{ip}"));
        self.line(&format!("  store i64 {value}, ptr %result_out, align 8"));
        self.line("  ret i32 0");
    }

    fn finish_with_register_zero(&mut self) {
        let value = self.load(0);
        self.spill("finish");
        self.line(&format!("  store i64 {value}, ptr %result_out, align 8"));
        self.line("  ret i32 0");
    }
}
