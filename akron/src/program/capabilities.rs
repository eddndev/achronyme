use std::ops::{BitOr, BitOrAssign};

use crate::specs::{
    CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, NativeMeta, ResourceEffect,
};

/// Runtime services required by a program image.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ProgramCapabilities(u64);

impl ProgramCapabilities {
    pub const PROVE: Self = Self(1 << 0);
    pub const CIRCOM: Self = Self(1 << 1);
    pub const NATIVE_CALLS: Self = Self(1 << 2);
    pub const FILE_IO: Self = Self(1 << 3);
    pub const VERIFY: Self = Self(1 << 4);
    pub const TASKS: Self = Self(1 << 5);
    pub const NETWORK_IO: Self = Self(1 << 6);
    pub const CONSOLE_IO: Self = Self(1 << 7);
    pub const CLOCK: Self = Self(1 << 8);
    pub const RANDOMNESS: Self = Self(1 << 9);
    pub const UNKNOWN_HOST: Self = Self(1 << 10);

    const KNOWN_BITS: u64 = Self::PROVE.0
        | Self::CIRCOM.0
        | Self::NATIVE_CALLS.0
        | Self::FILE_IO.0
        | Self::VERIFY.0
        | Self::TASKS.0
        | Self::NETWORK_IO.0
        | Self::CONSOLE_IO.0
        | Self::CLOCK.0
        | Self::RANDOMNESS.0
        | Self::UNKNOWN_HOST.0;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn from_bits(bits: u64) -> Option<Self> {
        (bits & !Self::KNOWN_BITS == 0).then_some(Self(bits))
    }

    pub(super) fn from_effects(effects: EffectSet) -> Self {
        let mut capabilities = Self::empty();
        if effects.contains(EffectSet::TASK) {
            capabilities |= Self::TASKS;
        }
        if effects.contains(EffectSet::IO_FILE) {
            capabilities |= Self::FILE_IO;
        }
        if effects.contains(EffectSet::IO_NETWORK) {
            capabilities |= Self::NETWORK_IO;
        }
        if effects.contains(EffectSet::IO_CONSOLE) {
            capabilities |= Self::CONSOLE_IO;
        }
        if effects.contains(EffectSet::IO_CLOCK) {
            capabilities |= Self::CLOCK;
        }
        if effects.contains(EffectSet::IO_RANDOM) {
            capabilities |= Self::RANDOMNESS;
        }
        if effects.contains(EffectSet::PROVE) {
            capabilities |= Self::PROVE;
        }
        if effects.contains(EffectSet::VERIFY) {
            capabilities |= Self::VERIFY;
        }
        if effects.contains(EffectSet::CIRCUIT) {
            capabilities |= Self::CIRCOM;
        }
        if effects.contains(EffectSet::UNKNOWN_HOST) {
            capabilities |= Self::UNKNOWN_HOST;
        }
        capabilities
    }

    pub(super) fn inferred_effects(self) -> EffectSet {
        let mut effects = EffectSet::empty();
        if self.contains(Self::TASKS) {
            effects |= EffectSet::TASK;
        }
        if self.contains(Self::FILE_IO) {
            effects |= EffectSet::IO_FILE;
        }
        if self.contains(Self::NETWORK_IO) {
            effects |= EffectSet::IO_NETWORK;
        }
        if self.contains(Self::CONSOLE_IO) {
            effects |= EffectSet::IO_CONSOLE;
        }
        if self.contains(Self::CLOCK) {
            effects |= EffectSet::IO_CLOCK;
        }
        if self.contains(Self::RANDOMNESS) {
            effects |= EffectSet::IO_RANDOM;
        }
        if self.contains(Self::PROVE) {
            effects |= EffectSet::PROVE;
        }
        if self.contains(Self::VERIFY) {
            effects |= EffectSet::VERIFY;
        }
        if self.contains(Self::CIRCOM) {
            effects |= EffectSet::CIRCUIT;
        }
        if self.contains(Self::UNKNOWN_HOST) {
            effects |= EffectSet::UNKNOWN_HOST;
        }
        effects
    }
}

impl BitOr for ProgramCapabilities {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ProgramCapabilities {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::fmt::Display for ProgramCapabilities {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let known = [
            (Self::PROVE, "prove"),
            (Self::CIRCOM, "circom"),
            (Self::NATIVE_CALLS, "native"),
            (Self::FILE_IO, "file"),
            (Self::VERIFY, "verify"),
            (Self::TASKS, "task"),
            (Self::NETWORK_IO, "network"),
            (Self::CONSOLE_IO, "console"),
            (Self::CLOCK, "clock"),
            (Self::RANDOMNESS, "random"),
            (Self::UNKNOWN_HOST, "host.unknown"),
        ];
        let mut first = true;
        for (capability, name) in known {
            if self.contains(capability) {
                if !first {
                    formatter.write_str(",")?;
                }
                formatter.write_str(name)?;
                first = false;
            }
        }
        if first {
            formatter.write_str("none")?;
        }
        Ok(())
    }
}

/// Artifact-safe metadata for a native global referenced by bytecode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramNativeMetadata {
    /// Effects inferred when the native global is referenced.
    pub effects: EffectSet,
    /// Host authority requested by the native.
    pub capabilities: CapabilitySet,
    /// Whether the native completes, blocks, or suspends.
    pub behavior: NativeBehavior,
    /// Cancellation contract.
    pub cancellation: CancellationPolicy,
    /// Owned resource created or consumed by the call.
    pub resource: ResourceEffect,
}

impl ProgramNativeMetadata {
    pub(super) fn from_native_meta(meta: &NativeMeta) -> Self {
        Self {
            effects: meta.effects,
            capabilities: meta.capabilities,
            behavior: meta.behavior,
            cancellation: meta.cancellation,
            resource: meta.resource,
        }
    }

    pub(super) fn program_capabilities(self) -> ProgramCapabilities {
        let mut capabilities = ProgramCapabilities::NATIVE_CALLS;
        if self.effects.contains(EffectSet::TASK) {
            capabilities |= ProgramCapabilities::TASKS;
        }
        if self.effects.contains(EffectSet::IO_FILE)
            || self
                .capabilities
                .intersects(CapabilitySet::FILE_READ | CapabilitySet::FILE_WRITE)
        {
            capabilities |= ProgramCapabilities::FILE_IO;
        }
        if self.effects.contains(EffectSet::IO_NETWORK)
            || self
                .capabilities
                .intersects(CapabilitySet::NETWORK_CONNECT | CapabilitySet::NETWORK_LISTEN)
        {
            capabilities |= ProgramCapabilities::NETWORK_IO;
        }
        if self.effects.contains(EffectSet::IO_CONSOLE)
            || self
                .capabilities
                .intersects(CapabilitySet::CONSOLE_READ | CapabilitySet::CONSOLE_WRITE)
        {
            capabilities |= ProgramCapabilities::CONSOLE_IO;
        }
        if self.effects.contains(EffectSet::IO_CLOCK)
            || self.capabilities.contains(CapabilitySet::CLOCK)
        {
            capabilities |= ProgramCapabilities::CLOCK;
        }
        if self.effects.contains(EffectSet::IO_RANDOM)
            || self.capabilities.contains(CapabilitySet::RANDOM)
        {
            capabilities |= ProgramCapabilities::RANDOMNESS;
        }
        if self.effects.contains(EffectSet::VERIFY) {
            capabilities |= ProgramCapabilities::VERIFY;
        }
        if self.effects.contains(EffectSet::UNKNOWN_HOST)
            || self.capabilities.contains(CapabilitySet::UNKNOWN_HOST)
        {
            capabilities |= ProgramCapabilities::UNKNOWN_HOST;
        }
        capabilities
    }
}
