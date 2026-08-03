use crate::symbol::{Arity, Availability};
use crate::{CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, ResourceEffect};

use super::{BuiltinAuditError, ProveIrLowerHandle, VmFnHandle};

/// One entry in the [`BuiltinRegistry`].
///
/// Declares everything the compilers need to know about a builtin: its
/// name (for diagnostics and for the resolver pass's name lookup), its
/// arity (for early validation), its availability (for audit), and the
/// pointers to its implementation(s).
#[derive(Debug, Clone)]
pub struct BuiltinEntry {
    /// The canonical name users write in source code. `poseidon`,
    /// `assert_eq`, `mux`, etc. Unique within the registry.
    pub name: &'static str,
    /// Expected argument count. Checked by the resolver pass before
    /// dispatch; individual impls may validate further.
    pub arity: Arity,
    /// Which contexts can call this builtin. Enforced structurally by
    /// [`BuiltinRegistry::audit`].
    pub availability: Availability,
    /// VM runtime implementation handle. Required for
    /// [`Availability::Vm`] and [`Availability::Both`]; must be `None`
    /// for [`Availability::ProveIr`].
    pub vm_fn: Option<VmFnHandle>,
    /// ProveIR lowering callback handle. Required for
    /// [`Availability::ProveIr`] and [`Availability::Both`]; must be
    /// `None` for [`Availability::Vm`].
    pub prove_ir_lower: Option<ProveIrLowerHandle>,
    /// Effects inferred at every call site that resolves to this builtin.
    pub effects: EffectSet,
    /// Host authority required before the builtin may execute.
    pub capabilities: CapabilitySet,
    /// Whether the builtin completes immediately, blocks, or suspends.
    pub behavior: NativeBehavior,
    /// Cancellation behavior once the operation has been invoked.
    pub cancellation: CancellationPolicy,
    /// Owned resource created or consumed by the call.
    pub resource: ResourceEffect,
}

/// Compiler-visible metadata for a VM native supplied by a host module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeMeta {
    /// Canonical source-level name.
    pub name: &'static str,
    /// Fixed arity, or -1 for a variadic function.
    pub arity: isize,
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
}

impl BuiltinEntry {
    /// Audit just this entry against the four invariants documented at
    /// the module level. Returns a specific error describing the
    /// violation, or `Ok(())`.
    pub fn audit(&self) -> Result<(), BuiltinAuditError> {
        match self.availability {
            Availability::Both => {
                if self.vm_fn.is_none() {
                    return Err(BuiltinAuditError::BothMissingVm { name: self.name });
                }
                if self.prove_ir_lower.is_none() {
                    return Err(BuiltinAuditError::BothMissingProveIr { name: self.name });
                }
            }
            Availability::Vm => {
                if self.vm_fn.is_none() {
                    return Err(BuiltinAuditError::VmMissingImpl { name: self.name });
                }
                if self.prove_ir_lower.is_some() {
                    return Err(BuiltinAuditError::VmDeclaresProveIr { name: self.name });
                }
            }
            Availability::ProveIr => {
                if self.prove_ir_lower.is_none() {
                    return Err(BuiltinAuditError::ProveIrMissingImpl { name: self.name });
                }
                if self.vm_fn.is_some() {
                    return Err(BuiltinAuditError::ProveIrDeclaresVm { name: self.name });
                }
            }
        }
        let required_effects = self.capabilities.required_effects();
        if !self.effects.contains(required_effects) {
            return Err(BuiltinAuditError::CapabilityEffectMismatch {
                name: self.name,
                declared: self.effects,
                required: required_effects,
            });
        }
        if self.behavior == NativeBehavior::Suspending && !self.effects.contains(EffectSet::TASK) {
            return Err(BuiltinAuditError::SuspendingWithoutTask { name: self.name });
        }
        if self.availability.includes_prove_ir()
            && self.effects.intersects(EffectSet::FORBIDDEN_IN_CIRCUIT)
        {
            return Err(BuiltinAuditError::ProveIrHasHostEffect {
                name: self.name,
                effects: self.effects,
            });
        }
        Ok(())
    }
}
