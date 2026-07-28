use akron::opcode::instruction::{decode_a, decode_b, decode_bx, decode_c};
use akron::OpCode;

use super::emit::Emitter;

const I60_MIN: i64 = -(1i64 << 59);
const I60_MAX: i64 = (1i64 << 59) - 1;
const PAYLOAD_MASK: u64 = (1u64 << 60) - 1;
const NIL_BITS: u64 = 1u64 << 60;
const FALSE_BITS: u64 = 2u64 << 60;
const TRUE_BITS: u64 = 3u64 << 60;

impl Emitter<'_> {
    pub(super) fn binary_integer(&mut self, ip: usize, instruction: u32, opcode: OpCode) {
        let left_raw = self.load(decode_b(instruction));
        self.require_integer(ip, "left", &left_raw);
        let left = self.decode_integer(ip, "left", &left_raw);
        let right_raw = self.load(decode_c(instruction));
        self.require_integer(ip, "right", &right_raw);
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
                self.raise_block(&format!("division.zero.{ip}"), ip, 2);
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

    pub(super) fn negate_integer(&mut self, ip: usize, instruction: u32) {
        let raw = self.load(decode_b(instruction));
        self.require_integer(ip, "neg", &raw);
        let integer = self.decode_integer(ip, "neg", &raw);
        self.line(&format!("  %result.{ip} = sub i64 0, {integer}"));
        self.check_range_and_store(ip, decode_a(instruction), &format!("%result.{ip}"), "false");
    }

    pub(super) fn comparison(&mut self, ip: usize, instruction: u32, opcode: OpCode) {
        let left_raw = self.load(decode_b(instruction));
        let right_raw = self.load(decode_c(instruction));
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
            self.require_integer(ip, "compare_left", &left_raw);
            self.require_integer(ip, "compare_right", &right_raw);
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
        self.store(decode_a(instruction), &format!("%bool.{ip}"));
        self.next(ip);
    }

    pub(super) fn logical_not(&mut self, ip: usize, instruction: u32) {
        let raw = self.load(decode_b(instruction));
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
        self.store(decode_a(instruction), &format!("%bool.{ip}"));
        self.next(ip);
    }

    pub(super) fn jump_if_false(&mut self, ip: usize, instruction: u32) {
        let raw = self.load(decode_a(instruction));
        self.line(&format!(
            "  %condition.nil.{ip} = icmp eq i64 {raw}, {NIL_BITS}"
        ));
        self.line(&format!(
            "  %condition.false.{ip} = icmp eq i64 {raw}, {FALSE_BITS}"
        ));
        self.line(&format!(
            "  %condition.falsey.{ip} = or i1 %condition.nil.{ip}, %condition.false.{ip}"
        ));
        let fallthrough = self.next_label(ip);
        self.line(&format!(
            "  br i1 %condition.falsey.{ip}, label %ip{}, label %{fallthrough}",
            decode_bx(instruction)
        ));
    }

    fn decode_integer(&mut self, ip: usize, name: &str, value: &str) -> String {
        let shifted = format!("%{name}.shifted.{ip}");
        let integer = format!("%{name}.int.{ip}");
        self.line(&format!("  {shifted} = shl i64 {value}, 4"));
        self.line(&format!("  {integer} = ashr i64 {shifted}, 4"));
        integer
    }

    fn require_integer(&mut self, ip: usize, name: &str, value: &str) {
        self.line(&format!("  %{name}.tag.{ip} = lshr i64 {value}, 60"));
        self.line(&format!(
            "  %{name}.is_int.{ip} = icmp eq i64 %{name}.tag.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %{name}.is_int.{ip}, label %{name}.int.ok.{ip}, label %{name}.deopt.{ip}"
        ));
        self.deoptimize(&format!("{name}.deopt.{ip}"), ip);
        self.line(&format!("{name}.int.ok.{ip}:"));
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
        self.raise_block(&format!("overflow.{ip}"), ip, 3);
        self.line(&format!("arith.ok.{ip}:"));
        self.line(&format!(
            "  %encoded.{ip} = and i64 {result}, {PAYLOAD_MASK}"
        ));
        self.store(destination, &format!("%encoded.{ip}"));
        self.next(ip);
    }
}
