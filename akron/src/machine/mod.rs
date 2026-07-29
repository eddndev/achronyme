//! Machine module - VM implementation
//!
//! This module contains the Virtual Machine implementation segmented into
//! focused submodules for maintainability and scalability.

mod arithmetic;
pub mod circom;
mod closure;
mod comparison;
mod compiled_call;
mod control;
mod data;
mod frame;
mod gc;
mod globals;
mod interpreter;
mod iterator;
mod method_call;
pub mod methods;
mod native;
mod promotion;
pub mod prototype;
pub mod prove;
mod specialization;
mod stack;
mod upvalue;
pub mod value_ops;
mod vm;

// Public API
pub use circom::{CircomCallError, CircomCallResult, CircomOutputValue, CircomWitnessHandler};
pub use frame::CallFrame;
pub use prove::{ProveError, ProveHandler, ProveResult, VerifyHandler};
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
