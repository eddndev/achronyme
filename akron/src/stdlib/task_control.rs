use crate::error::RuntimeError;
use crate::machine::VM;
use ach_macros::{ach_module, ach_native};
use memory::Value;

#[ach_module(name = "task_control")]
pub mod task_control_impl {
    use super::*;

    #[ach_native(
        name = "cancel_check",
        arity = 0,
        effects = "task",
        cancellation = "cooperative"
    )]
    pub fn native_cancel_check(vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        vm.check_task_cancellation()?;
        Ok(Value::nil())
    }
}
