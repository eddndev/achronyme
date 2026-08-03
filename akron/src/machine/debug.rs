use std::collections::HashMap;

use super::VM;

impl VM {
    /// Parse a passive debug-symbol sidecar from raw bytes.
    pub fn load_debug_section(&mut self, bytes: &[u8]) {
        if bytes.len() < 4 {
            return;
        }

        let mut cursor = 0;
        if bytes[cursor] != 0xDB || bytes[cursor + 1] != 0x67 {
            return;
        }
        cursor += 2;

        let count = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;
        let mut map = HashMap::new();

        for _ in 0..count {
            if cursor + 4 > bytes.len() {
                break;
            }
            let global_idx = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
            cursor += 2;
            let name_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            cursor += 2;
            if cursor + name_len > bytes.len() {
                break;
            }
            let name_bytes = &bytes[cursor..cursor + name_len];
            cursor += name_len;
            if let Ok(name) = std::str::from_utf8(name_bytes) {
                map.insert(global_idx, name.to_string());
            }
        }

        self.debug_symbols = Some(map);
    }
}
