use crate::{RuntimeError, VM};

use super::super::{context::run_status_helper, RuntimeContext, STATUS_INTERNAL_ERROR};

#[test]
fn runtime_helper_contains_panics_at_the_abi_boundary() {
    let mut vm = VM::new();
    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let status = run_status_helper(context.as_opaque(), |_| -> Result<_, RuntimeError> {
            panic!("expected test panic")
        });

        assert_eq!(status, STATUS_INTERNAL_ERROR);
        context.finish(status).unwrap_err()
    };

    assert!(error
        .to_string()
        .contains("compiled runtime helper panicked"));
}
