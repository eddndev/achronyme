#[cfg(test)]
mod tests {
    use super::*;
    use instruction::*;

    #[test]
    fn test_opcode_conversion() {
        assert_eq!(OpCode::Add.as_u8(), 10);
        assert_eq!(OpCode::from_u8(10), Some(OpCode::Add));
        assert_eq!(OpCode::from_u8(255), Some(OpCode::Nop));
        assert_eq!(OpCode::from_u8(205), None); // 205 is not assigned
    }

    #[test]
    fn test_instruction_encoding() {
        let inst = encode_abc(OpCode::Add.as_u8(), 1, 2, 3);
        assert_eq!(decode_opcode(inst), OpCode::Add.as_u8());
        assert_eq!(decode_a(inst), 1);
        assert_eq!(decode_b(inst), 2);
        assert_eq!(decode_c(inst), 3);
    }

    #[test]
    fn test_instruction_encoding_abx() {
        let inst = encode_abx(OpCode::LoadConst.as_u8(), 5, 1000);
        assert_eq!(decode_opcode(inst), OpCode::LoadConst.as_u8());
        assert_eq!(decode_a(inst), 5);
        assert_eq!(decode_bx(inst), 1000);
    }

    #[test]
    fn call_circom_template_opcode_roundtrips() {
        assert_eq!(OpCode::CallCircomTemplate.as_u8(), 162);
        assert_eq!(OpCode::from_u8(162), Some(OpCode::CallCircomTemplate));
        assert_eq!(OpCode::CallCircomTemplate.name(), "CALL_CIRCOM_TEMPLATE");
    }

    #[test]
    fn call_circom_template_encodes_as_abc() {
        // Layout: A=dest, B=first_input_reg, C=input_count
        let inst = encode_abc(OpCode::CallCircomTemplate.as_u8(), 3, 5, 2);
        assert_eq!(decode_opcode(inst), OpCode::CallCircomTemplate.as_u8());
        assert_eq!(decode_a(inst), 3);
        assert_eq!(decode_b(inst), 5);
        assert_eq!(decode_c(inst), 2);
    }

    #[test]
    fn structured_concurrency_opcodes_roundtrip() {
        for (byte, opcode, name) in [
            (70, OpCode::ScopeEnter, "SCOPE_ENTER"),
            (71, OpCode::ScopeExit, "SCOPE_EXIT"),
            (72, OpCode::Spawn, "SPAWN"),
            (73, OpCode::Await, "AWAIT"),
            (74, OpCode::CancelCheck, "CANCEL_CHECK"),
            (75, OpCode::AwaitOutcome, "AWAIT_OUTCOME"),
            (76, OpCode::TaskForget, "TASK_FORGET"),
            (77, OpCode::AwaitRace, "AWAIT_RACE"),
        ] {
            assert_eq!(OpCode::from_u8(byte), Some(opcode));
            assert_eq!(opcode.as_u8(), byte);
            assert_eq!(opcode.name(), name);
        }
    }
}
