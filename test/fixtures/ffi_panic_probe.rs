#[inline(never)]
unsafe extern "C" fn fatal_abi_boundary() {
    panic!("expected fatal ABI panic");
}

fn main() {
    unsafe { fatal_abi_boundary() };
}
