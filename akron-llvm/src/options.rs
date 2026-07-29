#[cfg(any(feature = "llvm", feature = "aot", test))]
use akron::compiled::{
    RuntimeCapabilities, RUNTIME_ABI_V2_SIZE, RUNTIME_ABI_V3_SIZE, RUNTIME_ABI_V4_SIZE,
    RUNTIME_ABI_V5_SIZE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlvmTierOptions {
    pub fast_poll: bool,
    pub known_calls: bool,
    pub list_specializations: bool,
}

impl LlvmTierOptions {
    pub const fn tier1() -> Self {
        Self {
            fast_poll: false,
            known_calls: false,
            list_specializations: false,
        }
    }

    #[cfg(any(feature = "llvm", feature = "aot", test))]
    pub(crate) const fn runtime_requirement(self) -> RuntimeRequirement {
        let mut bits = RuntimeCapabilities::LLVM_TIER1.bits();
        if self.fast_poll {
            bits |= RuntimeCapabilities::FAST_POLL.bits();
        }
        if self.known_calls {
            bits |= RuntimeCapabilities::KNOWN_CALLS.bits();
        }
        if self.list_specializations {
            bits |= RuntimeCapabilities::LIST_SPECIALIZATION.bits();
        }
        let (version, size) = if self.list_specializations {
            (5, RUNTIME_ABI_V5_SIZE)
        } else if self.known_calls {
            (4, RUNTIME_ABI_V4_SIZE)
        } else if self.fast_poll {
            (3, RUNTIME_ABI_V3_SIZE)
        } else {
            (2, RUNTIME_ABI_V2_SIZE)
        };
        RuntimeRequirement {
            version,
            size,
            capabilities: RuntimeCapabilities::from_bits(bits),
        }
    }
}

impl Default for LlvmTierOptions {
    fn default() -> Self {
        Self {
            fast_poll: true,
            known_calls: true,
            list_specializations: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(feature = "llvm", feature = "aot", test))]
pub(crate) struct RuntimeRequirement {
    pub version: u32,
    pub size: u32,
    pub capabilities: RuntimeCapabilities,
}

#[cfg(test)]
mod tests {
    use akron::compiled::{
        RuntimeCapabilities, RUNTIME_ABI_V2_SIZE, RUNTIME_ABI_V3_SIZE, RUNTIME_ABI_V4_SIZE,
        RUNTIME_ABI_V5_SIZE,
    };

    use super::LlvmTierOptions;

    #[test]
    fn runtime_requirements_follow_enabled_capabilities() {
        let tier1 = LlvmTierOptions::tier1().runtime_requirement();
        assert_eq!(tier1.version, 2);
        assert_eq!(tier1.size, RUNTIME_ABI_V2_SIZE);
        assert_eq!(tier1.capabilities, RuntimeCapabilities::LLVM_TIER1);

        let fast_only = LlvmTierOptions {
            fast_poll: true,
            known_calls: false,
            list_specializations: false,
        }
        .runtime_requirement();
        assert_eq!(fast_only.version, 3);
        assert_eq!(fast_only.size, RUNTIME_ABI_V3_SIZE);
        assert_eq!(
            fast_only.capabilities,
            RuntimeCapabilities::LLVM_TIER1 | RuntimeCapabilities::FAST_POLL
        );

        let known_only = LlvmTierOptions {
            fast_poll: false,
            known_calls: true,
            list_specializations: false,
        }
        .runtime_requirement();
        assert_eq!(known_only.version, 4);
        assert_eq!(known_only.size, RUNTIME_ABI_V4_SIZE);
        assert_eq!(
            known_only.capabilities,
            RuntimeCapabilities::LLVM_TIER1 | RuntimeCapabilities::KNOWN_CALLS
        );

        let lists_only = LlvmTierOptions {
            fast_poll: false,
            known_calls: false,
            list_specializations: true,
        }
        .runtime_requirement();
        assert_eq!(lists_only.version, 5);
        assert_eq!(lists_only.size, RUNTIME_ABI_V5_SIZE);
        assert_eq!(
            lists_only.capabilities,
            RuntimeCapabilities::LLVM_TIER1 | RuntimeCapabilities::LIST_SPECIALIZATION
        );

        let tier2 = LlvmTierOptions::default().runtime_requirement();
        assert_eq!(tier2.version, 5);
        assert_eq!(tier2.size, RUNTIME_ABI_V5_SIZE);
        assert_eq!(tier2.capabilities, RuntimeCapabilities::LLVM_TIER2);
    }
}
