#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExecutionStats {
    pub direct_instructions: u64,
    pub runtime_calls: u64,
    pub compiled_function_calls: u64,
    pub native_function_calls: u64,
    pub interpreter_fallbacks: u64,
    pub block_polls: u64,
    pub slow_paths: u64,
    pub fast_poll_hits: u64,
    pub slow_poll_entries: u64,
    pub known_call_fast_hits: u64,
    pub known_call_fast_misses: u64,
    pub specialization_hits: u64,
    pub specialization_misses: u64,
    pub first_fallback_instruction: Option<u32>,
}

impl ExecutionStats {
    pub(super) fn record_block_poll(&mut self) {
        self.block_polls = self.block_polls.saturating_add(1);
    }

    pub(super) fn record_direct_instructions(&mut self, count: u32) {
        self.direct_instructions = self.direct_instructions.saturating_add(u64::from(count));
    }

    pub(super) fn refund_direct_instructions(&mut self, count: u32) {
        self.direct_instructions = self.direct_instructions.saturating_sub(u64::from(count));
    }

    pub(super) fn record_runtime_call(&mut self) {
        self.runtime_calls = self.runtime_calls.saturating_add(1);
    }

    pub(super) fn record_compiled_function_call(&mut self) {
        self.compiled_function_calls = self.compiled_function_calls.saturating_add(1);
    }

    pub(super) fn record_native_function_call(&mut self) {
        self.native_function_calls = self.native_function_calls.saturating_add(1);
    }

    pub(super) fn record_slow_path(&mut self) {
        self.slow_paths = self.slow_paths.saturating_add(1);
    }

    pub(super) fn record_slow_poll_entry(&mut self) {
        self.slow_poll_entries = self.slow_poll_entries.saturating_add(1);
    }

    pub(super) fn record_interpreter_fallback(&mut self, instruction: u32) {
        self.interpreter_fallbacks = self.interpreter_fallbacks.saturating_add(1);
        self.first_fallback_instruction.get_or_insert(instruction);
    }
}
