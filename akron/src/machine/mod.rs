//! Machine module - VM implementation
//!
//! This module contains the Virtual Machine implementation segmented into
//! focused submodules for maintainability and scalability.

mod arithmetic;
mod blocking_pool;
mod channel_hub;
pub mod circom;
mod closure;
mod comparison;
mod compiled_call;
mod control;
mod data;
mod debug;
mod frame;
mod gc;
mod globals;
mod interpreter;
mod iterator;
mod limits;
mod method_call;
pub mod methods;
mod native;
mod network_reactor;
mod promotion;
pub mod prototype;
pub mod prove;
mod specialization;
mod stack;
mod task;
mod timer_reactor;
mod upvalue;
pub mod value_ops;
mod vm;

// Public API
pub use circom::{CircomCallError, CircomCallResult, CircomOutputValue, CircomWitnessHandler};
pub use frame::CallFrame;
pub use limits::RuntimeLimits;
pub use prove::{ProveError, ProveHandler, ProveResult, VerifyHandler};
pub use task::{
    TaskAwaitMode, TaskDiagnostic, TaskDiagnosticState, TaskFailureDiagnostic, TaskWaitReason,
};
pub use vm::{MAX_FRAMES, VM};

pub(crate) enum CompiledCallTarget {
    NativeComplete,
    Prototype {
        frame_index: u32,
        base: u32,
        prototype_index: u32,
    },
    InterpreterRequired {
        frame_index: u32,
        base: u32,
    },
}
