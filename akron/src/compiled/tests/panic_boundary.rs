use crate::{RuntimeError, VM};

use super::super::{context::run_status_helper, RuntimeContext};

#[test]
#[should_panic(expected = "expected fatal runtime panic")]
fn runtime_helper_does_not_recover_internal_panics() {
    let mut vm = VM::new();
    let mut context = RuntimeContext::new(&mut vm);
    run_status_helper(context.as_opaque(), |_| -> Result<_, RuntimeError> {
        panic!("expected fatal runtime panic")
    });
}
