use akron::compiled::{STATUS_BAILOUT_REQUIRED, STATUS_SLOW_PATH_REQUIRED};

use super::Emitter;

impl Emitter<'_> {
    pub(super) fn begin_instruction(&mut self, ip: usize, direct: bool) {
        if !self.blocks.is_start(ip) {
            self.line(&format!("op{ip}:"));
            return;
        }

        let instruction_count = self.blocks.end(ip) - ip;
        let direct_instruction_count = if direct { instruction_count } else { 0 };
        self.line(&format!("ip{ip}:"));
        self.line(&format!(
            "  %poll.fast.status.{ip} = call i32 %poll_fast_block_fn(ptr %context, i32 %frame_index, i32 {ip}, i32 {instruction_count}, i32 {direct_instruction_count})"
        ));
        self.line(&format!(
            "  %poll.fast.ok.{ip} = icmp eq i32 %poll.fast.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %poll.fast.ok.{ip}, label %op{ip}, label %poll.fast.not_ok.{ip}"
        ));
        self.line(&format!("poll.fast.not_ok.{ip}:"));
        self.line(&format!(
            "  %poll.protected.required.{ip} = icmp eq i32 %poll.fast.status.{ip}, {STATUS_SLOW_PATH_REQUIRED}"
        ));
        self.line(&format!(
            "  br i1 %poll.protected.required.{ip}, label %poll.protected.{ip}, label %poll.fast.error.{ip}"
        ));
        self.line(&format!("poll.fast.error.{ip}:"));
        self.line(&format!("  ret i32 %poll.fast.status.{ip}"));
        self.protected_poll(ip, instruction_count, direct_instruction_count);
        self.line(&format!("op{ip}:"));
    }

    fn protected_poll(
        &mut self,
        ip: usize,
        instruction_count: usize,
        direct_instruction_count: usize,
    ) {
        self.line(&format!("poll.protected.{ip}:"));
        self.line(&format!(
            "  %poll.status.{ip} = call i32 %poll_tier1_block_fn(ptr %context, i32 %frame_index, i32 {ip}, i32 {instruction_count}, i32 {direct_instruction_count})"
        ));
        self.line(&format!(
            "  %poll.ok.{ip} = icmp eq i32 %poll.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %poll.ok.{ip}, label %op{ip}, label %poll.not_ok.{ip}"
        ));
        self.line(&format!("poll.not_ok.{ip}:"));
        self.line(&format!(
            "  %poll.bailout.required.{ip} = icmp eq i32 %poll.status.{ip}, {STATUS_BAILOUT_REQUIRED}"
        ));
        self.line(&format!(
            "  br i1 %poll.bailout.required.{ip}, label %poll.bailout.{ip}, label %poll.error.{ip}"
        ));
        self.line(&format!("poll.error.{ip}:"));
        self.line(&format!("  ret i32 %poll.status.{ip}"));
        self.line(&format!("poll.bailout.{ip}:"));
        self.spill(&format!("poll.bailout.{ip}"));
        self.line(&format!(
            "  %poll.bailout.status.{ip} = call i32 %bailout_fn(ptr %context, i32 %frame_index, i32 {ip}, ptr %result_out)"
        ));
        self.line(&format!("  ret i32 %poll.bailout.status.{ip}"));
    }
}
