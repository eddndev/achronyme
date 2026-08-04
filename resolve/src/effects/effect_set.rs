use std::fmt;
use std::ops::{BitOr, BitOrAssign};

/// Closed set of effects inferred for callables and complete programs.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct EffectSet(pub(super) u32);

/// Stable effect names for tooling and artifact inspection.
pub const EFFECT_CATALOG: &[(&str, EffectSet)] = &[
    ("task", EffectSet::TASK),
    ("io.console", EffectSet::IO_CONSOLE),
    ("io.file", EffectSet::IO_FILE),
    ("io.network", EffectSet::IO_NETWORK),
    ("io.clock", EffectSet::IO_CLOCK),
    ("io.random", EffectSet::IO_RANDOM),
    ("prove", EffectSet::PROVE),
    ("verify", EffectSet::VERIFY),
    ("circuit", EffectSet::CIRCUIT),
    ("host.unknown", EffectSet::UNKNOWN_HOST),
];

impl EffectSet {
    /// Task creation, suspension, or structured task management.
    pub const TASK: Self = Self(1 << 0);
    /// Console input or output.
    pub const IO_CONSOLE: Self = Self(1 << 1);
    /// Filesystem input or output.
    pub const IO_FILE: Self = Self(1 << 2);
    /// Network input or output.
    pub const IO_NETWORK: Self = Self(1 << 3);
    /// Wall or monotonic clock access.
    pub const IO_CLOCK: Self = Self(1 << 4);
    /// Host randomness access.
    pub const IO_RANDOM: Self = Self(1 << 5);
    /// Proof construction.
    pub const PROVE: Self = Self(1 << 6);
    /// Proof verification.
    pub const VERIFY: Self = Self(1 << 7);
    /// Circuit-only computation.
    pub const CIRCUIT: Self = Self(1 << 8);
    /// Native authority that could not be classified precisely.
    pub const UNKNOWN_HOST: Self = Self(1 << 9);

    /// Every concrete host I/O effect.
    pub const HOST_IO: Self = Self(
        Self::IO_CONSOLE.0
            | Self::IO_FILE.0
            | Self::IO_NETWORK.0
            | Self::IO_CLOCK.0
            | Self::IO_RANDOM.0,
    );

    /// Effects that are forbidden in proof and circuit bodies.
    pub const FORBIDDEN_IN_CIRCUIT: Self =
        Self(Self::TASK.0 | Self::HOST_IO.0 | Self::UNKNOWN_HOST.0);

    const KNOWN_BITS: u32 = Self::TASK.0
        | Self::HOST_IO.0
        | Self::PROVE.0
        | Self::VERIFY.0
        | Self::CIRCUIT.0
        | Self::UNKNOWN_HOST.0;

    /// Empty effect set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Raw stable bit representation.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Construct an effect set if every bit is known.
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

    /// Union two effect sets in const contexts.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Keep only effects shared with `other`.
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Remove every effect present in `other`.
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Whether no effects are present.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for EffectSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for EffectSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for &(name, effect) in EFFECT_CATALOG {
            if self.contains(effect) {
                if !first {
                    f.write_str(",")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        if first {
            f.write_str("pure")?;
        }
        Ok(())
    }
}
