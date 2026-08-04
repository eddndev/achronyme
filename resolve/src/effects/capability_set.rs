use std::fmt;
use std::ops::{BitOr, BitOrAssign};

use super::EffectSet;

/// Host authorities requested by native operations.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct CapabilitySet(u32);

/// Stable host-capability names for tooling and launch configuration.
pub const CAPABILITY_CATALOG: &[(&str, CapabilitySet)] = &[
    ("console.read", CapabilitySet::CONSOLE_READ),
    ("console.write", CapabilitySet::CONSOLE_WRITE),
    ("file.read", CapabilitySet::FILE_READ),
    ("file.write", CapabilitySet::FILE_WRITE),
    ("network.connect", CapabilitySet::NETWORK_CONNECT),
    ("network.listen", CapabilitySet::NETWORK_LISTEN),
    ("clock", CapabilitySet::CLOCK),
    ("random", CapabilitySet::RANDOM),
    ("host.unknown", CapabilitySet::UNKNOWN_HOST),
];

impl CapabilitySet {
    /// Read from the process console.
    pub const CONSOLE_READ: Self = Self(1 << 0);
    /// Write to the process console.
    pub const CONSOLE_WRITE: Self = Self(1 << 1);
    /// Read files through granted paths.
    pub const FILE_READ: Self = Self(1 << 2);
    /// Write files through granted paths.
    pub const FILE_WRITE: Self = Self(1 << 3);
    /// Establish outbound network connections.
    pub const NETWORK_CONNECT: Self = Self(1 << 4);
    /// Listen for inbound network connections.
    pub const NETWORK_LISTEN: Self = Self(1 << 5);
    /// Read host clocks.
    pub const CLOCK: Self = Self(1 << 6);
    /// Read host randomness.
    pub const RANDOM: Self = Self(1 << 7);
    /// Unclassified native authority.
    pub const UNKNOWN_HOST: Self = Self(1 << 8);

    const KNOWN_BITS: u32 = Self::CONSOLE_READ.0
        | Self::CONSOLE_WRITE.0
        | Self::FILE_READ.0
        | Self::FILE_WRITE.0
        | Self::NETWORK_CONNECT.0
        | Self::NETWORK_LISTEN.0
        | Self::CLOCK.0
        | Self::RANDOM.0
        | Self::UNKNOWN_HOST.0;

    /// Every currently defined host capability.
    pub const ALL: Self = Self(Self::KNOWN_BITS);

    /// Empty capability set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Raw stable bit representation.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Construct a capability set if every bit is known.
    pub const fn from_bits(bits: u32) -> Option<Self> {
        if bits & !Self::KNOWN_BITS == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Whether the set contains every bit from `other`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether the set shares any bit with `other`.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Remove every capability present in `other`.
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Whether no capabilities are present.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Effects required to describe this authority without under-reporting it.
    pub const fn required_effects(self) -> EffectSet {
        let mut bits = 0;
        if self.0 & (Self::CONSOLE_READ.0 | Self::CONSOLE_WRITE.0) != 0 {
            bits |= EffectSet::IO_CONSOLE.0;
        }
        if self.0 & (Self::FILE_READ.0 | Self::FILE_WRITE.0) != 0 {
            bits |= EffectSet::IO_FILE.0;
        }
        if self.0 & (Self::NETWORK_CONNECT.0 | Self::NETWORK_LISTEN.0) != 0 {
            bits |= EffectSet::IO_NETWORK.0;
        }
        if self.0 & Self::CLOCK.0 != 0 {
            bits |= EffectSet::IO_CLOCK.0;
        }
        if self.0 & Self::RANDOM.0 != 0 {
            bits |= EffectSet::IO_RANDOM.0;
        }
        if self.0 & Self::UNKNOWN_HOST.0 != 0 {
            bits |= EffectSet::UNKNOWN_HOST.0;
        }
        EffectSet(bits)
    }
}

impl BitOr for CapabilitySet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for CapabilitySet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for CapabilitySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for &(name, capability) in CAPABILITY_CATALOG {
            if self.contains(capability) {
                if !first {
                    f.write_str(",")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        if first {
            f.write_str("none")?;
        }
        Ok(())
    }
}
