extern crate self as akron;

pub mod compiled;
pub mod error;
pub mod globals;
pub mod host;
pub mod machine;
pub mod opcode;
pub mod program;
pub mod resource;

pub use error::RuntimeError;
pub use globals::GlobalEntry;
pub use host::HostPolicy;
pub use machine::prototype::known_method_names;
pub use machine::prove::ProveError;
pub use machine::value_ops::ValueOps;
pub use machine::{
    CallFrame, CircomCallError, CircomCallResult, CircomOutputValue, CircomWitnessHandler,
    ProveHandler, ProveResult, RuntimeLimits, TaskAwaitMode, TaskDiagnostic, TaskDiagnosticState,
    TaskFailureDiagnostic, TaskWaitReason, VerifyHandler, MAX_FRAMES, VM,
};
pub use opcode::OpCode;
pub use program::{
    CompiledProgram, ProgramCapabilities, ProgramNativeMetadata, BYTECODE_VERSION,
    EXECUTABLE_FORMAT_VERSION, LEGACY_BYTECODE_VERSION, LEGACY_CANONICAL_FORMAT_VERSION,
};
pub mod module;
pub mod native;
pub use module::{NativeDef, NativeModule};
pub use native::{
    MethodFn, NativeAsyncOutput, NativeAsyncRequest, NativeAsyncStart, NativeAsyncWork,
    NativeChannelRequest, NativeFn, NativeJob, NativeNetworkRequest, NativeObj,
};
pub use resource::SharedFileResource;
pub mod loader;
pub mod specs;
pub mod stdlib;
pub use loader::LoaderError;

// Re-export proc-macros so downstream crates use `akron::ach_native` / `akron::ach_module`
pub use ach_macros::{ach_module, ach_native};
