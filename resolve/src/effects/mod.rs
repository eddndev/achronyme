//! Effect and host-authority metadata shared by all Achronyme backends.

mod capability_set;
mod effect_set;
mod metadata;

pub use capability_set::{CapabilitySet, CAPABILITY_CATALOG};
pub use effect_set::{EffectSet, EFFECT_CATALOG};
pub use metadata::{CancellationPolicy, NativeBehavior, ResourceEffect, ResourceKind};

#[cfg(test)]
mod tests;
