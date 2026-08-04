//! NativeModule trait — modular registration of native functions.
//!
//! Each stdlib module (core, string, bigint, collections) implements
//! `NativeModule`, declaring its native functions in one place.
//! External modules (IO, zkML, etc.) implement the same trait to
//! extend the VM without modifying its internals.

use crate::native::{NativeAsyncStart, NativeFn};
use crate::specs::{CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, ResourceEffect};

/// A single native function definition.
pub struct NativeDef {
    /// Canonical source-level name.
    pub name: &'static str,
    /// Runtime implementation.
    pub func: NativeFn,
    /// Fixed arity, or -1 for a variadic function.
    pub arity: isize, // -1 = variadic
    /// Effects inferred at call sites.
    pub effects: EffectSet,
    /// Host authority required to invoke the function.
    pub capabilities: CapabilitySet,
    /// Whether the function completes, blocks, or suspends.
    pub behavior: NativeBehavior,
    /// Cancellation contract.
    pub cancellation: CancellationPolicy,
    /// Owned resource created or consumed by the call.
    pub resource: ResourceEffect,
    /// Optional two-stage adapter for bounded blocking-pool execution.
    pub async_start: Option<NativeAsyncStart>,
}

/// Trait implemented by each group of native functions.
///
/// # Contract
///
/// `natives()` must return definitions in a **stable order** — the
/// position within the returned `Vec` determines the global index
/// assigned to each function (after concatenating all modules).
pub trait NativeModule {
    /// Human-readable module name (e.g. `"core"`, `"string"`).
    fn name(&self) -> &'static str;

    /// The native functions provided by this module.
    fn natives(&self) -> Vec<NativeDef>;
}

/// Returns the built-in modules in registration order.
///
/// The order here **must** match the `VmFnHandle` ordering in
/// `resolve::BuiltinRegistry::default()` — `bootstrap_natives` verifies this.
pub fn builtin_modules() -> Vec<Box<dyn NativeModule>> {
    use crate::stdlib::{bigint::BigintModule, core::CoreModule, task_control::TaskControlModule};

    vec![
        Box::new(CoreModule),
        Box::new(BigintModule),
        Box::new(TaskControlModule),
    ]
}
