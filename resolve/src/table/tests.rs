#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::{BuiltinAuditError, BuiltinEntry, ProveIrLowerHandle, VmFnHandle};
    use crate::symbol::{Arity, Availability, ConstKind};
    use crate::{CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, ResourceEffect};

    #[test]
    fn empty_table() {
        let t = SymbolTable::new();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
        assert!(t.lookup("anything").is_none());
        assert!(t.audit().is_ok());
    }

    #[test]
    fn insert_and_lookup() {
        let mut t = SymbolTable::new();
        let id = t.insert(
            "math::PI",
            CallableKind::Constant {
                qualified_name: "math::PI".into(),
                const_kind: ConstKind::Int,
                value_handle: 0,
            },
        );
        assert_eq!(t.len(), 1);
        assert_eq!(t.lookup("math::PI"), Some(id));
        assert!(matches!(t.get(id), CallableKind::Constant { .. }));
    }

    #[test]
    #[should_panic(expected = "duplicate name")]
    fn duplicate_insert_panics() {
        let mut t = SymbolTable::new();
        t.insert(
            "x",
            CallableKind::Constant {
                qualified_name: "x".into(),
                const_kind: ConstKind::Int,
                value_handle: 0,
            },
        );
        t.insert(
            "x",
            CallableKind::Constant {
                qualified_name: "x".into(),
                const_kind: ConstKind::Field,
                value_handle: 0,
            },
        );
    }

    #[test]
    fn alias_chain_follows_to_target() {
        let mut t = SymbolTable::new();
        // base: a UserFn
        let base = t.insert(
            "p::add",
            CallableKind::UserFn {
                qualified_name: "p::add".into(),
                module: crate::module_graph::ModuleId::from_raw(0),
                stmt_index: 0,
                availability: Availability::Both,
            },
        );
        // first alias: a -> p::add
        let a = t.insert("a", CallableKind::FnAlias { target: base });
        // second alias: b -> a
        let b = t.insert("b", CallableKind::FnAlias { target: a });

        assert_eq!(t.resolve_alias(base).unwrap(), base);
        assert_eq!(t.resolve_alias(a).unwrap(), base);
        assert_eq!(t.resolve_alias(b).unwrap(), base);
    }

    #[test]
    fn alias_self_cycle_is_detected() {
        let mut t = SymbolTable::new();
        // The first insert gets SymbolId(0), so pointing at SymbolId(0)
        // from inside the same insert creates a self-cycle.
        let id = t.insert(
            "loop",
            CallableKind::FnAlias {
                target: SymbolId(0),
            },
        );
        let err = t.resolve_alias(id).unwrap_err();
        assert!(matches!(err, ResolveError::FnAliasCycle { .. }));
    }

    #[test]
    fn alias_two_hop_cycle_is_detected_as_cycle() {
        // A -> B -> A must produce FnAliasCycle, not FnAliasDepthExceeded.
        // This is the correctness fix for the hardening audit: earlier
        // versions only caught self-cycles and fell through to the
        // depth-exceeded error for multi-hop cycles, giving users a
        // misleading "max depth 16" message when the real problem was
        // a 2-hop loop.
        let mut t = SymbolTable::new();
        // A = slot 0 points at slot 1 (B)
        let a = t.insert(
            "a",
            CallableKind::FnAlias {
                target: SymbolId(1),
            },
        );
        // B = slot 1 points back at slot 0 (A)
        let b = t.insert("b", CallableKind::FnAlias { target: a });

        let err = t.resolve_alias(a).unwrap_err();
        assert!(
            matches!(err, ResolveError::FnAliasCycle { .. }),
            "expected FnAliasCycle, got {err:?}"
        );
        // Also verify the cycle is caught regardless of entry point.
        let err = t.resolve_alias(b).unwrap_err();
        assert!(matches!(err, ResolveError::FnAliasCycle { .. }));
    }

    #[test]
    fn alias_three_hop_cycle_is_detected_as_cycle() {
        // A -> B -> C -> A, another non-self cycle shape.
        let mut t = SymbolTable::new();
        let a = t.insert(
            "a",
            CallableKind::FnAlias {
                target: SymbolId(1),
            },
        );
        let _b = t.insert(
            "b",
            CallableKind::FnAlias {
                target: SymbolId(2),
            },
        );
        let _c = t.insert("c", CallableKind::FnAlias { target: a });

        let err = t.resolve_alias(a).unwrap_err();
        assert!(matches!(err, ResolveError::FnAliasCycle { .. }));
    }

    #[test]
    fn alias_depth_exceeded_is_detected() {
        // Build a chain of 20 aliases longer than FN_ALIAS_MAX_DEPTH.
        // We populate `symbols` directly since the chain needs forward
        // references that `insert` can't express.
        let mut t = SymbolTable::new();
        for i in 0u32..20 {
            t.symbols.push(CallableKind::FnAlias {
                target: SymbolId(i + 1),
            });
            t.by_qualified_name
                .insert(format!("alias_{i}"), SymbolId(i));
        }
        // Slot 20 does not exist; the walker should error out on
        // depth exceed before hitting the out-of-range access because
        // FN_ALIAS_MAX_DEPTH (16) < 20.
        let err = t.resolve_alias(SymbolId(0)).unwrap_err();
        assert!(matches!(err, ResolveError::FnAliasDepthExceeded { .. }));
    }

    #[test]
    fn with_registry_runs_audit() {
        let mut reg = BuiltinRegistry::new();
        reg.push(BuiltinEntry {
            name: "broken",
            arity: Arity::Fixed(1),
            availability: Availability::Both,
            // Missing vm_fn should fail audit.
            vm_fn: None,
            prove_ir_lower: Some(ProveIrLowerHandle::PLACEHOLDER),
            effects: EffectSet::empty(),
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::None,
        });
        let result = SymbolTable::with_registry(reg);
        assert!(matches!(
            result,
            Err(ResolveError::BuiltinAudit(
                BuiltinAuditError::BothMissingVm { .. }
            ))
        ));
    }

    #[test]
    fn valid_registry_builds_table() {
        let mut reg = BuiltinRegistry::new();
        reg.push(BuiltinEntry {
            name: "poseidon",
            arity: Arity::Fixed(2),
            availability: Availability::Both,
            vm_fn: Some(VmFnHandle::PLACEHOLDER),
            prove_ir_lower: Some(ProveIrLowerHandle::PLACEHOLDER),
            effects: EffectSet::empty(),
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::None,
        });
        let t = SymbolTable::with_registry(reg).unwrap();
        assert!(t.audit().is_ok());
        assert_eq!(t.builtin_registry().len(), 1);
    }

    #[test]
    fn builtin_index_out_of_range_rejected_by_audit() {
        // Use an explicit empty registry; the default() registry is
        // populated with production builtins, so any index in [0, 21)
        // would be valid. We need a table where *every* index is out
        // of range.
        let mut t = SymbolTable::with_registry(BuiltinRegistry::new()).unwrap();
        t.insert("ghost", CallableKind::Builtin { entry_index: 0 });
        let err = t.audit().unwrap_err();
        assert!(matches!(err, ResolveError::BuiltinIndexOutOfRange { .. }));
    }
}
