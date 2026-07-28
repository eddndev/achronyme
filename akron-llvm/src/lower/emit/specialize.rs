use super::Emitter;

const STATUS_SPECIALIZATION_MISS: u32 = 10;

impl Emitter<'_> {
    pub(super) fn list_push_instruction(
        &mut self,
        ip: usize,
        instruction: u32,
        preamble_ip: usize,
    ) {
        self.spill(&format!("specialization.push.{ip}"));
        self.line(&format!(
            "  %specialization.push.status.{ip} = call i32 %list_push_fn(ptr %context, i32 %base, i32 {instruction})"
        ));
        self.line(&format!(
            "  %specialization.push.hit.{ip} = icmp eq i32 %specialization.push.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %specialization.push.hit.{ip}, label %specialization.push.done.{ip}, label %specialization.push.not_hit.{ip}"
        ));
        self.line(&format!("specialization.push.not_hit.{ip}:"));
        self.line(&format!(
            "  %specialization.push.miss.{ip} = icmp eq i32 %specialization.push.status.{ip}, {STATUS_SPECIALIZATION_MISS}"
        ));
        self.line(&format!(
            "  br i1 %specialization.push.miss.{ip}, label %specialization.push.fallback.load.{ip}, label %specialization.push.error.{ip}"
        ));
        self.line(&format!("specialization.push.error.{ip}:"));
        self.line(&format!("  ret i32 %specialization.push.status.{ip}"));
        self.line(&format!("specialization.push.fallback.load.{ip}:"));
        self.line(&format!(
            "  %specialization.push.load.status.{ip} = call i32 %execute_instruction_fn(ptr %context, i32 %frame_index, i32 {preamble_ip})"
        ));
        self.line(&format!(
            "  %specialization.push.load.ok.{ip} = icmp eq i32 %specialization.push.load.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %specialization.push.load.ok.{ip}, label %specialization.push.fallback.method.{ip}, label %specialization.push.load.error.{ip}"
        ));
        self.line(&format!("specialization.push.load.error.{ip}:"));
        self.line(&format!("  ret i32 %specialization.push.load.status.{ip}"));
        self.line(&format!("specialization.push.fallback.method.{ip}:"));
        self.line(&format!(
            "  %specialization.push.method.status.{ip} = call i32 %execute_instruction_fn(ptr %context, i32 %frame_index, i32 {ip})"
        ));
        self.line(&format!(
            "  %specialization.push.method.ok.{ip} = icmp eq i32 %specialization.push.method.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %specialization.push.method.ok.{ip}, label %specialization.push.done.{ip}, label %specialization.push.method.error.{ip}"
        ));
        self.line(&format!("specialization.push.method.error.{ip}:"));
        self.line(&format!(
            "  ret i32 %specialization.push.method.status.{ip}"
        ));
        self.line(&format!("specialization.push.done.{ip}:"));
        self.reload(&format!("specialization.push.{ip}"));
        self.next(ip);
    }

    pub(super) fn list_index_instruction(&mut self, ip: usize, instruction: u32) {
        self.spill(&format!("specialization.index.{ip}"));
        self.line(&format!(
            "  %specialization.index.status.{ip} = call i32 %list_index_fn(ptr %context, i32 %base, i32 {instruction})"
        ));
        self.line(&format!(
            "  %specialization.index.hit.{ip} = icmp eq i32 %specialization.index.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %specialization.index.hit.{ip}, label %specialization.index.done.{ip}, label %specialization.index.not_hit.{ip}"
        ));
        self.line(&format!("specialization.index.not_hit.{ip}:"));
        self.line(&format!(
            "  %specialization.index.miss.{ip} = icmp eq i32 %specialization.index.status.{ip}, {STATUS_SPECIALIZATION_MISS}"
        ));
        self.line(&format!(
            "  br i1 %specialization.index.miss.{ip}, label %specialization.index.fallback.{ip}, label %specialization.index.error.{ip}"
        ));
        self.line(&format!("specialization.index.error.{ip}:"));
        self.line(&format!("  ret i32 %specialization.index.status.{ip}"));
        self.line(&format!("specialization.index.fallback.{ip}:"));
        self.line(&format!(
            "  %specialization.index.fallback.status.{ip} = call i32 %execute_instruction_fn(ptr %context, i32 %frame_index, i32 {ip})"
        ));
        self.line(&format!(
            "  %specialization.index.fallback.ok.{ip} = icmp eq i32 %specialization.index.fallback.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %specialization.index.fallback.ok.{ip}, label %specialization.index.done.{ip}, label %specialization.index.fallback.error.{ip}"
        ));
        self.line(&format!("specialization.index.fallback.error.{ip}:"));
        self.line(&format!(
            "  ret i32 %specialization.index.fallback.status.{ip}"
        ));
        self.line(&format!("specialization.index.done.{ip}:"));
        self.reload(&format!("specialization.index.{ip}"));
        self.next(ip);
    }
}
