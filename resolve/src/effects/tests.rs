use super::*;

#[test]
fn effect_display_is_stable_and_ordered() {
    let effects = EffectSet::IO_FILE | EffectSet::TASK | EffectSet::VERIFY;
    assert_eq!(effects.to_string(), "task,io.file,verify");
    assert_eq!(EffectSet::empty().to_string(), "pure");
}

#[test]
fn capability_effect_mapping_is_complete() {
    let capabilities =
        CapabilitySet::FILE_READ | CapabilitySet::NETWORK_LISTEN | CapabilitySet::CLOCK;
    assert_eq!(
        capabilities.required_effects(),
        EffectSet::IO_FILE | EffectSet::IO_NETWORK | EffectSet::IO_CLOCK
    );
}

#[test]
fn capability_display_is_stable_and_ordered() {
    let capabilities =
        CapabilitySet::NETWORK_LISTEN | CapabilitySet::FILE_READ | CapabilitySet::CLOCK;
    assert_eq!(capabilities.to_string(), "file.read,network.listen,clock");
    assert_eq!(CapabilitySet::empty().to_string(), "none");
}

#[test]
fn unknown_bits_are_rejected() {
    assert!(EffectSet::from_bits(1 << 31).is_none());
    assert!(CapabilitySet::from_bits(1 << 31).is_none());
}

#[test]
fn metadata_enums_have_stable_roundtrip_encodings() {
    for behavior in [
        NativeBehavior::Immediate,
        NativeBehavior::Blocking,
        NativeBehavior::Suspending,
    ] {
        assert_eq!(
            NativeBehavior::from_byte(behavior.to_byte()),
            Some(behavior)
        );
    }
    for cancellation in [
        CancellationPolicy::None,
        CancellationPolicy::BeforeStart,
        CancellationPolicy::Cooperative,
    ] {
        assert_eq!(
            CancellationPolicy::from_byte(cancellation.to_byte()),
            Some(cancellation)
        );
    }
    for resource in [
        ResourceEffect::None,
        ResourceEffect::Creates(ResourceKind::Connection),
        ResourceEffect::Consumes(ResourceKind::Task),
        ResourceEffect::Borrows(ResourceKind::File),
        ResourceEffect::CreatesAndBorrows {
            created: ResourceKind::Connection,
            borrowed: ResourceKind::Listener,
        },
    ] {
        let (operation, kind) = resource.to_bytes();
        assert_eq!(ResourceEffect::from_bytes(operation, kind), Some(resource));
    }
    assert_eq!(NativeBehavior::from_byte(255), None);
    assert_eq!(ResourceEffect::from_bytes(0, 1), None);
}
