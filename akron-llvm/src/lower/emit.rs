use std::fmt::Write;

use akron::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c, decode_opcode};
use akron::{CompiledProgram, OpCode};

use super::verify::Verification;
use super::LoweringError;

const I60_MIN: i64 = -(1i64 << 59);
const I60_MAX: i64 = (1i64 << 59) - 1;
const PAYLOAD_MASK: u64 = (1u64 << 60) - 1;
const NIL_BITS: u64 = 1u64 << 60;
const FALSE_BITS: u64 = 2u64 << 60;
const TRUE_BITS: u64 = 3u64 << 60;

pub(super) fn emit(
    program: &CompiledProgram,
    verification: &Verification,
) -> Result<String, LoweringError> {
    let mut emitter = Emitter::new(program.main.chunk.len());
    emitter.header();
    if program.main.chunk.is_empty() {
        emitter.line("  br label %finish");
    } else {
        emitter.line("  br label %ip0");
    }

    for (ip, &instruction) in program.main.chunk.iter().enumerate() {
        if !verification.is_native(ip) {
            emitter.bailout(ip);
            continue;
        }
        let opcode = OpCode::from_u8(decode_opcode(instruction)).ok_or_else(|| {
            LoweringError::Internal(format!("unknown opcode reached emitter at {ip}"))
        })?;
        emitter.poll(ip);
        match opcode {
            OpCode::LoadConst => {
                let value = program.main.constants[decode_bx(instruction) as usize];
                emitter.store(
                    ip,
                    "constant",
                    decode_a(instruction),
                    &value.to_abi_bits().to_string(),
                );
                emitter.next(ip);
            }
            OpCode::LoadTrue => {
                emitter.store(ip, "true", decode_a(instruction), &TRUE_BITS.to_string());
                emitter.next(ip);
            }
            OpCode::LoadFalse => {
                emitter.store(ip, "false", decode_a(instruction), &FALSE_BITS.to_string());
                emitter.next(ip);
            }
            OpCode::LoadNil => {
                emitter.store(ip, "nil", decode_a(instruction), &NIL_BITS.to_string());
                emitter.next(ip);
            }
            OpCode::Move => {
                let value = emitter.load(ip, "move", decode_b(instruction), 0);
                emitter.store(ip, "move", decode_a(instruction), &value);
                emitter.next(ip);
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
                    "unsupported opcode {} reached emitter",
                    opcode.name()
                )));
            }
        }
    }

    emitter.line("finish:");
    if program.main.chunk.is_empty() {
        emitter.line(&format!("  store i64 {NIL_BITS}, ptr %result_out, align 8"));
        emitter.line("  ret i32 0");
    } else {
        emitter.finish_with_register_zero();
    }
    emitter.line("}");
    Ok(emitter.output)
}

struct Emitter {
    output: String,
    instruction_count: usize,
}

impl Emitter {
    fn new(instruction_count: usize) -> Self {
        Self {
            output: String::with_capacity(instruction_count * 900 + 1024),
            instruction_count,
        }
    }

    fn line(&mut self, line: &str) {
        writeln!(self.output, "{line}").expect("write to String");
    }

    fn header(&mut self) {
        self.line("; Generated from canonical Akron bytecode.");
        self.line("%RuntimeApi = type { [8 x i8], i32, i32, i64, ptr, ptr, ptr, ptr, ptr }");
        self.line("declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)");
        self.line("declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)");
        self.line("declare { i64, i1 } @llvm.smul.with.overflow.i64(i64, i64)");
        self.line("");
        self.line("define i32 @akron_compiled_main(ptr %api, ptr %context, i32 %frame_index, i32 %base, ptr %result_out) {");
        self.line("entry:");
        self.line("  %scratch0 = alloca i64, align 8");
        self.line("  %scratch1 = alloca i64, align 8");
        self.line("  %load_slot = getelementptr %RuntimeApi, ptr %api, i32 0, i32 4");
        self.line("  %load_fn = load ptr, ptr %load_slot, align 8");
        self.line("  %store_slot = getelementptr %RuntimeApi, ptr %api, i32 0, i32 5");
        self.line("  %store_fn = load ptr, ptr %store_slot, align 8");
        self.line("  %poll_slot = getelementptr %RuntimeApi, ptr %api, i32 0, i32 6");
        self.line("  %poll_fn = load ptr, ptr %poll_slot, align 8");
        self.line("  %raise_slot = getelementptr %RuntimeApi, ptr %api, i32 0, i32 7");
        self.line("  %raise_fn = load ptr, ptr %raise_slot, align 8");
        self.line("  %bailout_slot = getelementptr %RuntimeApi, ptr %api, i32 0, i32 8");
        self.line("  %bailout_fn = load ptr, ptr %bailout_slot, align 8");
    }

    fn bailout(&mut self, ip: usize) {
        self.line(&format!("ip{ip}:"));
        self.line(&format!(
            "  %bailout.status.{ip} = call i32 %bailout_fn(ptr %context, i32 %frame_index, i32 {ip}, ptr %result_out)"
        ));
        self.line(&format!("  ret i32 %bailout.status.{ip}"));
    }

    fn finish_with_register_zero(&mut self) {
        self.line("  %finish.load.status = call i32 %load_fn(ptr %context, i32 %base, i32 0, ptr %scratch0)");
        self.line("  %finish.load.ok = icmp eq i32 %finish.load.status, 0");
        self.line("  br i1 %finish.load.ok, label %finish.loaded, label %finish.load.error");
        self.line("finish.load.error:");
        self.line("  ret i32 %finish.load.status");
        self.line("finish.loaded:");
        self.line("  %finish.value = load i64, ptr %scratch0, align 8");
        self.line("  store i64 %finish.value, ptr %result_out, align 8");
        self.line("  ret i32 0");
    }

    fn poll(&mut self, ip: usize) {
        self.line(&format!("ip{ip}:"));
        self.line(&format!(
            "  %poll.status.{ip} = call i32 %poll_fn(ptr %context, i32 %frame_index, i32 {ip})"
        ));
        self.line(&format!(
            "  %poll.ok.{ip} = icmp eq i32 %poll.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %poll.ok.{ip}, label %op{ip}, label %poll.error.{ip}"
        ));
        self.line(&format!("poll.error.{ip}:"));
        self.line(&format!("  ret i32 %poll.status.{ip}"));
        self.line(&format!("op{ip}:"));
    }

    fn load(&mut self, ip: usize, name: &str, register: u8, scratch: u8) -> String {
        let prefix = format!("{ip}.{name}.{scratch}");
        self.line(&format!(
            "  %load.status.{prefix} = call i32 %load_fn(ptr %context, i32 %base, i32 {register}, ptr %scratch{scratch})"
        ));
        self.line(&format!(
            "  %load.ok.{prefix} = icmp eq i32 %load.status.{prefix}, 0"
        ));
        self.line(&format!(
            "  br i1 %load.ok.{prefix}, label %load.done.{prefix}, label %load.error.{prefix}"
        ));
        self.line(&format!("load.error.{prefix}:"));
        self.line(&format!("  ret i32 %load.status.{prefix}"));
        self.line(&format!("load.done.{prefix}:"));
        let value = format!("%value.{prefix}");
        self.line(&format!(
            "  {value} = load i64, ptr %scratch{scratch}, align 8"
        ));
        value
    }

    fn store(&mut self, ip: usize, name: &str, register: u8, value: &str) {
        let prefix = format!("{ip}.{name}");
        self.line(&format!(
            "  %store.status.{prefix} = call i32 %store_fn(ptr %context, i32 %base, i32 {register}, i64 {value})"
        ));
        self.line(&format!(
            "  %store.ok.{prefix} = icmp eq i32 %store.status.{prefix}, 0"
        ));
        self.line(&format!(
            "  br i1 %store.ok.{prefix}, label %store.done.{prefix}, label %store.error.{prefix}"
        ));
        self.line(&format!("store.error.{prefix}:"));
        self.line(&format!("  ret i32 %store.status.{prefix}"));
        self.line(&format!("store.done.{prefix}:"));
    }

    fn next(&mut self, ip: usize) {
        if ip + 1 < self.instruction_count {
            self.line(&format!("  br label %ip{}", ip + 1));
        } else {
            self.line("  br label %finish");
        }
    }

    fn decode_integer(&mut self, ip: usize, name: &str, value: &str) -> String {
        let shifted = format!("%{name}.shifted.{ip}");
        let integer = format!("%{name}.int.{ip}");
        self.line(&format!("  {shifted} = shl i64 {value}, 4"));
        self.line(&format!("  {integer} = ashr i64 {shifted}, 4"));
        integer
    }

    fn check_range_and_store(&mut self, ip: usize, destination: u8, result: &str, overflow: &str) {
        self.line(&format!("  %below.{ip} = icmp slt i64 {result}, {I60_MIN}"));
        self.line(&format!("  %above.{ip} = icmp sgt i64 {result}, {I60_MAX}"));
        self.line(&format!(
            "  %range.bad.{ip} = or i1 %below.{ip}, %above.{ip}"
        ));
        self.line(&format!(
            "  %arith.bad.{ip} = or i1 {overflow}, %range.bad.{ip}"
        ));
        self.line(&format!(
            "  br i1 %arith.bad.{ip}, label %overflow.{ip}, label %arith.ok.{ip}"
        ));
        self.raise_block(&format!("overflow.{ip}"), 3);
        self.line(&format!("arith.ok.{ip}:"));
        self.line(&format!(
            "  %encoded.{ip} = and i64 {result}, {PAYLOAD_MASK}"
        ));
        self.store(ip, "arith", destination, &format!("%encoded.{ip}"));
        self.next(ip);
    }

    fn raise_block(&mut self, label: &str, code: u32) {
        let clean = label.replace('.', "_");
        self.line(&format!("{label}:"));
        self.line(&format!(
            "  %raise.status.{clean} = call i32 %raise_fn(ptr %context, i32 {code})"
        ));
        self.line(&format!("  ret i32 %raise.status.{clean}"));
    }

    fn binary_integer(&mut self, ip: usize, instruction: u32, opcode: OpCode) {
        let left_raw = self.load(ip, "left", decode_b(instruction), 0);
        let left = self.decode_integer(ip, "left", &left_raw);
        let right_raw = self.load(ip, "right", decode_c(instruction), 1);
        let right = self.decode_integer(ip, "right", &right_raw);
        let destination = decode_a(instruction);

        match opcode {
            OpCode::Add | OpCode::Sub | OpCode::Mul => {
                let intrinsic = match opcode {
                    OpCode::Add => "sadd",
                    OpCode::Sub => "ssub",
                    OpCode::Mul => "smul",
                    _ => unreachable!(),
                };
                self.line(&format!(
                    "  %pair.{ip} = call {{ i64, i1 }} @llvm.{intrinsic}.with.overflow.i64(i64 {left}, i64 {right})"
                ));
                self.line(&format!(
                    "  %result.{ip} = extractvalue {{ i64, i1 }} %pair.{ip}, 0"
                ));
                self.line(&format!(
                    "  %overflow.flag.{ip} = extractvalue {{ i64, i1 }} %pair.{ip}, 1"
                ));
                self.check_range_and_store(
                    ip,
                    destination,
                    &format!("%result.{ip}"),
                    &format!("%overflow.flag.{ip}"),
                );
            }
            OpCode::Div | OpCode::Mod => {
                self.line(&format!("  %zero.{ip} = icmp eq i64 {right}, 0"));
                self.line(&format!(
                    "  br i1 %zero.{ip}, label %division.zero.{ip}, label %division.ok.{ip}"
                ));
                self.raise_block(&format!("division.zero.{ip}"), 2);
                self.line(&format!("division.ok.{ip}:"));
                let operation = if opcode == OpCode::Div {
                    "sdiv"
                } else {
                    "srem"
                };
                self.line(&format!("  %result.{ip} = {operation} i64 {left}, {right}"));
                self.check_range_and_store(ip, destination, &format!("%result.{ip}"), "false");
            }
            _ => unreachable!(),
        }
    }

    fn negate_integer(&mut self, ip: usize, instruction: u32) {
        let raw = self.load(ip, "neg", decode_b(instruction), 0);
        let integer = self.decode_integer(ip, "neg", &raw);
        self.line(&format!("  %result.{ip} = sub i64 0, {integer}"));
        self.check_range_and_store(ip, decode_a(instruction), &format!("%result.{ip}"), "false");
    }

    fn comparison(&mut self, ip: usize, instruction: u32, opcode: OpCode) {
        let left_raw = self.load(ip, "compare_left", decode_b(instruction), 0);
        let right_raw = self.load(ip, "compare_right", decode_c(instruction), 1);
        let predicate = match opcode {
            OpCode::Eq => "eq",
            OpCode::NotEq => "ne",
            OpCode::Lt => "slt",
            OpCode::Gt => "sgt",
            OpCode::Le => "sle",
            OpCode::Ge => "sge",
            _ => unreachable!(),
        };
        let (left, right) = if matches!(opcode, OpCode::Eq | OpCode::NotEq) {
            (left_raw, right_raw)
        } else {
            (
                self.decode_integer(ip, "compare_left", &left_raw),
                self.decode_integer(ip, "compare_right", &right_raw),
            )
        };
        self.line(&format!(
            "  %comparison.{ip} = icmp {predicate} i64 {left}, {right}"
        ));
        self.line(&format!(
            "  %bool.{ip} = select i1 %comparison.{ip}, i64 {TRUE_BITS}, i64 {FALSE_BITS}"
        ));
        self.store(
            ip,
            "comparison",
            decode_a(instruction),
            &format!("%bool.{ip}"),
        );
        self.next(ip);
    }

    fn logical_not(&mut self, ip: usize, instruction: u32) {
        let raw = self.load(ip, "not", decode_b(instruction), 0);
        self.line(&format!("  %is.nil.{ip} = icmp eq i64 {raw}, {NIL_BITS}"));
        self.line(&format!(
            "  %is.false.{ip} = icmp eq i64 {raw}, {FALSE_BITS}"
        ));
        self.line(&format!(
            "  %falsey.{ip} = or i1 %is.nil.{ip}, %is.false.{ip}"
        ));
        self.line(&format!(
            "  %bool.{ip} = select i1 %falsey.{ip}, i64 {TRUE_BITS}, i64 {FALSE_BITS}"
        ));
        self.store(ip, "not", decode_a(instruction), &format!("%bool.{ip}"));
        self.next(ip);
    }

    fn jump_if_false(&mut self, ip: usize, instruction: u32) {
        let raw = self.load(ip, "condition", decode_a(instruction), 0);
        self.line(&format!(
            "  %condition.nil.{ip} = icmp eq i64 {raw}, {NIL_BITS}"
        ));
        self.line(&format!(
            "  %condition.false.{ip} = icmp eq i64 {raw}, {FALSE_BITS}"
        ));
        self.line(&format!(
            "  %condition.falsey.{ip} = or i1 %condition.nil.{ip}, %condition.false.{ip}"
        ));
        let fallthrough = if ip + 1 < self.instruction_count {
            format!("ip{}", ip + 1)
        } else {
            "finish".to_string()
        };
        self.line(&format!(
            "  br i1 %condition.falsey.{ip}, label %ip{}, label %{fallthrough}",
            decode_bx(instruction)
        ));
    }

    fn return_value(&mut self, ip: usize, instruction: u32) {
        if decode_b(instruction) == 1 {
            let value = self.load(ip, "return", decode_a(instruction), 0);
            self.line(&format!("  store i64 {value}, ptr %result_out, align 8"));
        } else {
            self.line(&format!("  store i64 {NIL_BITS}, ptr %result_out, align 8"));
        }
        self.line("  ret i32 0");
    }
}
