//! Runtime boundary shared by compiled Akron backends.
//!
//! Expected execution failures are represented by runtime status values and a
//! pending [`crate::RuntimeError`]. Unexpected internal panics are fatal. ABI
//! helpers do not recover panics, and no unwind may cross an `extern "C"`
//! boundary.

mod abi;
mod calls;
mod context;
mod specializations;
mod stats;

pub use abi::{
    runtime_api, CompiledEntry, ExecuteInstructionFn, ExecutionWindowFn, FinishCallFn,
    InterpreterBailoutFn, PollBlockFn, PollFastBlockFn, PollTier1BlockFn, PrepareCallFn,
    PrepareKnownCallFn, RefundBlockFn, RegisterWindowFn, RuntimeAbiError, RuntimeApi,
    RuntimeCapabilities, RuntimeStatus, SpecializeInstructionFn, TaskSignalFn, TaskTransitionFn,
    ERROR_ASSERTION_FAILED, ERROR_DIVISION_BY_ZERO, ERROR_INTEGER_OVERFLOW, ERROR_INVALID_OPERAND,
    ERROR_STACK_OVERFLOW, RUNTIME_ABI_V1_SIZE, RUNTIME_ABI_V2_SIZE, RUNTIME_ABI_V3_SIZE,
    RUNTIME_ABI_V4_SIZE, RUNTIME_ABI_V5_SIZE, RUNTIME_ABI_V6_SIZE, RUNTIME_ABI_VERSION,
    STATUS_BAILOUT_REQUIRED, STATUS_CALL_INTERPRETER_REQUIRED, STATUS_INTERNAL_ERROR,
    STATUS_INTERPRETER_COMPLETED, STATUS_INVALID_ARGUMENT, STATUS_KNOWN_CALL_MISS,
    STATUS_NATIVE_CALL_COMPLETE, STATUS_OK, STATUS_RUNTIME_ERROR, STATUS_SLOW_PATH_REQUIRED,
    STATUS_SPECIALIZATION_MISS,
};
pub use context::RuntimeContext;
pub use stats::ExecutionStats;

#[cfg(test)]
mod tests;
