use super::emit::Emitter;

impl Emitter<'_> {
    pub(super) fn define_global(&mut self, ip: usize, register: u8, index: u16, mutable: bool) {
        let deopt = format!("global.define.deopt.{ip}");
        let ready = format!("global.define.ready.{ip}");
        self.global_bounds(ip, index, &ready, &deopt);
        let entry = self.global_entry(ip, index);
        let value = self.load(register);
        self.line(&format!("  store i64 {value}, ptr {entry}, align 8"));
        self.line(&format!(
            "  %global.define.mutable.ptr.{ip} = getelementptr i8, ptr {entry}, i64 8"
        ));
        self.line(&format!(
            "  store i8 {}, ptr %global.define.mutable.ptr.{ip}, align 1",
            u8::from(mutable)
        ));
        self.line(&format!(
            "  %global.define.defined.ptr.{ip} = getelementptr i8, ptr {entry}, i64 9"
        ));
        self.line(&format!(
            "  store i8 1, ptr %global.define.defined.ptr.{ip}, align 1"
        ));
        self.next(ip);
        self.deoptimize(&deopt, ip);
    }

    pub(super) fn get_global(&mut self, ip: usize, register: u8, index: u16) {
        let deopt = format!("global.get.deopt.{ip}");
        let address = format!("global.get.address.{ip}");
        self.global_bounds(ip, index, &address, &deopt);
        let entry = self.global_entry(ip, index);
        let ready = format!("global.get.ready.{ip}");
        self.global_defined(ip, &entry, &ready, &deopt);
        self.line(&format!(
            "  %global.get.value.{ip} = load i64, ptr {entry}, align 8"
        ));
        self.store(register, &format!("%global.get.value.{ip}"));
        self.next(ip);
        self.deoptimize(&deopt, ip);
    }

    pub(super) fn set_global(&mut self, ip: usize, register: u8, index: u16) {
        let deopt = format!("global.set.deopt.{ip}");
        let address = format!("global.set.address.{ip}");
        self.global_bounds(ip, index, &address, &deopt);
        let entry = self.global_entry(ip, index);
        let defined = format!("global.set.defined.{ip}");
        self.global_defined(ip, &entry, &defined, &deopt);
        self.line(&format!(
            "  %global.set.mutable.ptr.{ip} = getelementptr i8, ptr {entry}, i64 8"
        ));
        self.line(&format!(
            "  %global.set.mutable.{ip} = load i8, ptr %global.set.mutable.ptr.{ip}, align 1"
        ));
        self.line(&format!(
            "  %global.set.is_mutable.{ip} = icmp ne i8 %global.set.mutable.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %global.set.is_mutable.{ip}, label %global.set.ready.{ip}, label %{deopt}"
        ));
        self.line(&format!("global.set.ready.{ip}:"));
        let value = self.load(register);
        self.line(&format!("  store i64 {value}, ptr {entry}, align 8"));
        self.next(ip);
        self.deoptimize(&deopt, ip);
    }

    fn global_bounds(&mut self, ip: usize, index: u16, ready: &str, deopt: &str) {
        self.line(&format!(
            "  %global.inbounds.{ip} = icmp ult i32 {index}, %globals.len"
        ));
        self.line(&format!(
            "  br i1 %global.inbounds.{ip}, label %{ready}, label %{deopt}"
        ));
        self.line(&format!("{ready}:"));
    }

    fn global_entry(&mut self, ip: usize, index: u16) -> String {
        let entry = format!("%global.entry.{ip}");
        let offset = usize::from(index) * 16;
        self.line(&format!(
            "  {entry} = getelementptr i8, ptr %globals, i64 {offset}"
        ));
        entry
    }

    fn global_defined(&mut self, ip: usize, entry: &str, ready: &str, deopt: &str) {
        self.line(&format!(
            "  %global.defined.ptr.{ip} = getelementptr i8, ptr {entry}, i64 9"
        ));
        self.line(&format!(
            "  %global.defined.{ip} = load i8, ptr %global.defined.ptr.{ip}, align 1"
        ));
        self.line(&format!(
            "  %global.is_defined.{ip} = icmp ne i8 %global.defined.{ip}, 0"
        ));
        self.line(&format!(
            "  br i1 %global.is_defined.{ip}, label %{ready}, label %{deopt}"
        ));
        self.line(&format!("{ready}:"));
    }
}
