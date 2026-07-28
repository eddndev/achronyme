mod abi;
mod context;

pub use abi::{
    runtime_api, CompiledEntry, InterpreterBailoutFn, RuntimeAbiError, RuntimeApi,
    RuntimeCapabilities, RuntimeStatus, ERROR_ASSERTION_FAILED, ERROR_DIVISION_BY_ZERO,
    ERROR_INTEGER_OVERFLOW, ERROR_INVALID_OPERAND, ERROR_STACK_OVERFLOW, RUNTIME_ABI_VERSION,
    STATUS_INTERNAL_ERROR, STATUS_INVALID_ARGUMENT, STATUS_OK, STATUS_RUNTIME_ERROR,
};
pub use context::RuntimeContext;

#[cfg(test)]
mod tests;
