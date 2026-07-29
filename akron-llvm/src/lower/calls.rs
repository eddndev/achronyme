use super::emit::Emitter;
const STATUS_NATIVE_CALL_COMPLETE: u32 = 5;
const STATUS_INTERPRETER_COMPLETED: u32 = 6;
const STATUS_CALL_INTERPRETER_REQUIRED: u32 = 7;
const STATUS_KNOWN_CALL_MISS: u32 = 9;

impl Emitter<'_> {
    pub(super) fn runtime_instruction(&mut self, ip: usize) {
        self.spill(&format!("runtime.{ip}"));
        self.line(&format!(
            "  %runtime.status.{ip} = call i32 %execute_instruction_fn(ptr %context, i32 %frame_index, i32 {ip})"
        ));
        self.line(&format!(
            "  %runtime.ok.{ip} = icmp eq i32 %runtime.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %runtime.ok.{ip}, label %runtime.done.{ip}, label %runtime.error.{ip}"
        ));
        self.line(&format!("runtime.error.{ip}:"));
        self.line(&format!("  ret i32 %runtime.status.{ip}"));
        self.line(&format!("runtime.done.{ip}:"));
        self.reload(&format!("runtime.{ip}"));
        self.next(ip);
    }

    pub(super) fn call_instruction(
        &mut self,
        ip: usize,
        instruction: u32,
        expected_prototype: Option<u32>,
    ) {
        self.spill(&format!("call.{ip}"));
        if let Some(prototype) = expected_prototype {
            self.line(&format!(
                "  %call.known.status.{ip} = call i32 %prepare_known_call_fn(ptr %context, i32 %frame_index, i32 {instruction}, i32 {prototype}, ptr %call.frame.out, ptr %call.base.out)",
            ));
            self.line(&format!(
                "  %call.known.prepared.{ip} = icmp eq i32 %call.known.status.{ip}, 0"
            ));
            self.line(&format!(
                "  br i1 %call.known.prepared.{ip}, label %call.known.dispatch.{ip}, label %call.known.not_prepared.{ip}"
            ));
            self.line(&format!("call.known.dispatch.{ip}:"));
            self.line(&format!(
                "  store i32 {prototype}, ptr %call.prototype.out, align 4"
            ));
            self.line(&format!("  br label %call.dispatch.{ip}"));
            self.line(&format!("call.known.not_prepared.{ip}:"));
            self.line(&format!(
                "  %call.known.miss.{ip} = icmp eq i32 %call.known.status.{ip}, {STATUS_KNOWN_CALL_MISS}"
            ));
            self.line(&format!(
                "  br i1 %call.known.miss.{ip}, label %call.prepare.{ip}, label %call.known.error.{ip}"
            ));
            self.line(&format!("call.known.error.{ip}:"));
            self.line(&format!("  ret i32 %call.known.status.{ip}"));
            self.line(&format!("call.prepare.{ip}:"));
        }
        self.line(&format!(
            "  %call.prepare.status.{ip} = call i32 %prepare_call_fn(ptr %context, i32 %frame_index, i32 {ip}, ptr %call.frame.out, ptr %call.base.out, ptr %call.prototype.out)"
        ));
        self.line(&format!(
            "  %call.prepared.{ip} = icmp eq i32 %call.prepare.status.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %call.prepared.{ip}, label %call.dispatch.{ip}, label %call.not_prepared.{ip}"
        ));
        self.line(&format!("call.not_prepared.{ip}:"));
        self.line(&format!(
            "  %call.native.{ip} = icmp eq i32 %call.prepare.status.{ip}, {STATUS_NATIVE_CALL_COMPLETE}"
        ));
        self.line(&format!(
            "  br i1 %call.native.{ip}, label %call.continue.{ip}, label %call.maybe_interpret.{ip}"
        ));
        self.line(&format!("call.maybe_interpret.{ip}:"));
        self.line(&format!(
            "  %call.interpret.required.{ip} = icmp eq i32 %call.prepare.status.{ip}, {STATUS_CALL_INTERPRETER_REQUIRED}"
        ));
        self.line(&format!(
            "  br i1 %call.interpret.required.{ip}, label %call.interpret.{ip}, label %call.prepare.error.{ip}"
        ));
        self.line(&format!("call.prepare.error.{ip}:"));
        self.line(&format!("  ret i32 %call.prepare.status.{ip}"));

        self.line(&format!("call.dispatch.{ip}:"));
        self.line(&format!(
            "  %call.frame.{ip} = load i32, ptr %call.frame.out, align 4"
        ));
        self.line(&format!(
            "  %call.base.{ip} = load i32, ptr %call.base.out, align 4"
        ));
        self.line(&format!(
            "  %call.prototype.{ip} = load i32, ptr %call.prototype.out, align 4"
        ));
        self.line(&format!(
            "  switch i32 %call.prototype.{ip}, label %call.interpret.{ip} ["
        ));
        for prototype in 0..self.prototype_count {
            self.line(&format!(
                "    i32 {prototype}, label %call.prototype.{ip}.{prototype}"
            ));
        }
        self.line("  ]");
        for prototype in 0..self.prototype_count {
            self.line(&format!("call.prototype.{ip}.{prototype}:"));
            self.line(&format!(
                "  %call.callee.status.{ip}.{prototype} = call i32 @akron_compiled_fn_{prototype}(ptr %api, ptr %context, i32 %call.frame.{ip}, i32 %call.base.{ip}, ptr %call.result.out)"
            ));
            self.line(&format!("  br label %call.after.{ip}"));
        }
        if self.prototype_count > 0 {
            self.line(&format!("call.after.{ip}:"));
            let incoming = (0..self.prototype_count)
                .map(|prototype| {
                    format!(
                        "[ %call.callee.status.{ip}.{prototype}, %call.prototype.{ip}.{prototype} ]"
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.line(&format!("  %call.callee.status.{ip} = phi i32 {incoming}"));
            self.line(&format!(
                "  %call.callee.ok.{ip} = icmp eq i32 %call.callee.status.{ip}, 0"
            ));
            self.line(&format!(
                "  br i1 %call.callee.ok.{ip}, label %call.finish.{ip}, label %call.callee.not_ok.{ip}"
            ));
            self.line(&format!("call.callee.not_ok.{ip}:"));
            self.line(&format!(
                "  %call.callee.interpreted.{ip} = icmp eq i32 %call.callee.status.{ip}, {STATUS_INTERPRETER_COMPLETED}"
            ));
            self.line(&format!(
                "  br i1 %call.callee.interpreted.{ip}, label %call.propagate.{ip}, label %call.callee.error.{ip}"
            ));
            self.line(&format!("call.callee.error.{ip}:"));
            self.line(&format!("  ret i32 %call.callee.status.{ip}"));
            self.line(&format!("call.finish.{ip}:"));
            self.line(&format!(
                "  %call.result.{ip} = load i64, ptr %call.result.out, align 8"
            ));
            self.line(&format!(
                "  %call.finish.status.{ip} = call i32 %finish_call_fn(ptr %context, i32 %call.frame.{ip}, i64 %call.result.{ip})"
            ));
            self.line(&format!(
                "  %call.finish.ok.{ip} = icmp eq i32 %call.finish.status.{ip}, 0"
            ));
            self.line(&format!(
                "  br i1 %call.finish.ok.{ip}, label %call.continue.{ip}, label %call.finish.error.{ip}"
            ));
            self.line(&format!("call.finish.error.{ip}:"));
            self.line(&format!("  ret i32 %call.finish.status.{ip}"));
        }

        self.line(&format!("call.interpret.{ip}:"));
        self.line(&format!(
            "  %call.interpret.frame.{ip} = load i32, ptr %call.frame.out, align 4"
        ));
        self.line(&format!(
            "  %call.interpret.status.{ip} = call i32 %bailout_fn(ptr %context, i32 %call.interpret.frame.{ip}, i32 0, ptr %call.result.out)"
        ));
        self.line(&format!(
            "  %call.interpret.completed.{ip} = icmp eq i32 %call.interpret.status.{ip}, {STATUS_INTERPRETER_COMPLETED}"
        ));
        self.line(&format!(
            "  br i1 %call.interpret.completed.{ip}, label %call.interpret.done.{ip}, label %call.interpret.error.{ip}"
        ));
        self.line(&format!("call.interpret.error.{ip}:"));
        self.line(&format!("  ret i32 %call.interpret.status.{ip}"));
        self.line(&format!("call.interpret.done.{ip}:"));
        self.line(&format!("  br label %call.propagate.{ip}"));
        self.line(&format!("call.propagate.{ip}:"));
        let status = if self.prototype_count > 0 {
            format!(
                "phi i32 [ %call.callee.status.{ip}, %call.callee.not_ok.{ip} ], [ %call.interpret.status.{ip}, %call.interpret.done.{ip} ]"
            )
        } else {
            format!("phi i32 [ %call.interpret.status.{ip}, %call.interpret.done.{ip} ]")
        };
        self.line(&format!("  %call.propagate.status.{ip} = {status}"));
        self.line(&format!(
            "  %call.propagate.result.{ip} = load i64, ptr %call.result.out, align 8"
        ));
        self.line(&format!(
            "  store i64 %call.propagate.result.{ip}, ptr %result_out, align 8"
        ));
        self.line(&format!("  ret i32 %call.propagate.status.{ip}"));

        self.line(&format!("call.continue.{ip}:"));
        self.reload(&format!("call.{ip}"));
        self.next(ip);
    }

    pub(super) fn deoptimize(&mut self, label: &str, ip: usize) {
        let site = label.replace('.', "_");
        let refund = self.blocks.end(ip).saturating_sub(ip);
        self.line(&format!("{label}:"));
        self.spill(&format!("deopt.{site}"));
        self.line(&format!(
            "  %deopt.refund.status.{site} = call i32 %refund_block_fn(ptr %context, i32 %frame_index, i32 {ip}, i32 {refund})"
        ));
        self.line(&format!(
            "  %deopt.refund.ok.{site} = icmp eq i32 %deopt.refund.status.{site}, 0"
        ));
        self.line(&format!(
            "  br i1 %deopt.refund.ok.{site}, label %deopt.resume.{site}, label %deopt.refund.error.{site}"
        ));
        self.line(&format!("deopt.refund.error.{site}:"));
        self.line(&format!("  ret i32 %deopt.refund.status.{site}"));
        self.line(&format!("deopt.resume.{site}:"));
        self.line(&format!(
            "  %deopt.status.{site} = call i32 %bailout_fn(ptr %context, i32 %frame_index, i32 {ip}, ptr %result_out)"
        ));
        self.line(&format!("  ret i32 %deopt.status.{site}"));
    }
}
