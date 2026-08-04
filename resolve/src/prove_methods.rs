//! Canonical metadata for method syntax supported by ProveIR.

/// Lowering selected for a method call in a `prove` or `circuit` context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProveMethodKind {
    /// Return the fixed length of an array value.
    Len,
    /// Treat an integer as a field element.
    ToField,
    /// Compute an integer absolute value with circuit primitives.
    Abs,
    /// Select the smaller of two integer values.
    Min,
    /// Select the larger of two integer values.
    Max,
    /// Raise a value to a statically known exponent.
    Pow,
}

/// Metadata for one method accepted by the ProveIR lowering path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProveMethodDecl {
    /// Source-level method name without the leading dot.
    pub name: &'static str,
    /// Number of explicit arguments, excluding the receiver.
    pub explicit_arity: usize,
    /// Backend-independent lowering selector.
    pub kind: ProveMethodKind,
}

/// Complete set of source-level methods supported in provable code.
pub const PROVE_METHODS: &[ProveMethodDecl] = &[
    ProveMethodDecl {
        name: "len",
        explicit_arity: 0,
        kind: ProveMethodKind::Len,
    },
    ProveMethodDecl {
        name: "to_field",
        explicit_arity: 0,
        kind: ProveMethodKind::ToField,
    },
    ProveMethodDecl {
        name: "abs",
        explicit_arity: 0,
        kind: ProveMethodKind::Abs,
    },
    ProveMethodDecl {
        name: "min",
        explicit_arity: 1,
        kind: ProveMethodKind::Min,
    },
    ProveMethodDecl {
        name: "max",
        explicit_arity: 1,
        kind: ProveMethodKind::Max,
    },
    ProveMethodDecl {
        name: "pow",
        explicit_arity: 1,
        kind: ProveMethodKind::Pow,
    },
];

/// Look up provable method metadata by source-level name.
pub fn lookup_prove_method(name: &str) -> Option<&'static ProveMethodDecl> {
    PROVE_METHODS.iter().find(|decl| decl.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_names_are_unique() {
        let mut names = HashSet::new();
        for decl in PROVE_METHODS {
            assert!(names.insert(decl.name), "duplicate method {}", decl.name);
        }
    }

    #[test]
    fn lookup_returns_declared_arity_and_kind() {
        assert_eq!(
            lookup_prove_method("len"),
            Some(&ProveMethodDecl {
                name: "len",
                explicit_arity: 0,
                kind: ProveMethodKind::Len,
            })
        );
        assert!(lookup_prove_method("unknown").is_none());
    }
}
