//! Tests for the top-level `Compiler` orchestrator — warnings,
//! "did you mean?" suggestions, and keyword-argument validation.
//!
//! Grouped into three inline modules by area of concern. All three
//! rely on the real [`Compiler::compile`] entry point rather than
//! stubbing, so each test doubles as an integration check across the
//! parser → resolver → codegen pipeline for the matching diagnostic.

#[cfg(test)]
mod warning_tests {
    use super::super::*;
    use achronyme_parser::Severity;

    fn compile_warnings(source: &str) -> Vec<achronyme_parser::Diagnostic> {
        let mut compiler = Compiler::new();
        let _ = compiler.compile(source);
        compiler.take_warnings()
    }

    // === W001: Unused variables ===

    #[test]
    fn unused_variable_in_function() {
        let ws = compile_warnings("fn test() { let x = 5; 1 }");
        assert!(ws
            .iter()
            .any(|w| w.message.contains("unused variable: `x`")));
    }

    #[test]
    fn used_variable_no_warning() {
        let ws = compile_warnings("fn test() { let x = 5; print(x) }");
        assert!(!ws.iter().any(|w| w.message.contains("unused variable")));
    }

    #[test]
    fn underscore_prefix_suppresses_warning() {
        let ws = compile_warnings("fn test() { let _x = 5; 1 }");
        assert!(!ws.iter().any(|w| w.message.contains("unused variable")));
    }

    #[test]
    fn unused_function_parameter() {
        let ws = compile_warnings("fn test(x) { 1 }");
        assert!(ws
            .iter()
            .any(|w| w.message.contains("unused function parameter: `x`")));
    }

    #[test]
    fn used_function_parameter_no_warning() {
        let ws = compile_warnings("fn test(x) { x }");
        assert!(!ws.iter().any(|w| w.message.contains("unused")));
    }

    #[test]
    fn array_parameters_used_only_by_prove_are_not_reported_unused() {
        let ws = compile_warnings(
            "fn prove_membership(\
                 root,\
                 leaf,\
                 path: Field[2],\
                 indices: Field[2]\
             ) {\
                 prove(root: Public) {\
                     merkle_verify(root, leaf, path, indices)\
                 }\
             }",
        );

        assert!(
            !ws.iter().any(|warning| {
                warning
                    .message
                    .contains("unused function parameter: `path`")
                    || warning
                        .message
                        .contains("unused function parameter: `indices`")
            }),
            "prove captures must count as parameter reads: {ws:?}"
        );
    }

    #[test]
    fn underscore_param_suppresses_warning() {
        let ws = compile_warnings("fn test(_x) { 1 }");
        assert!(!ws
            .iter()
            .any(|w| w.message.contains("unused function parameter")));
    }

    #[test]
    fn unused_for_loop_variable() {
        let ws = compile_warnings("fn test() { for x in [1, 2, 3] { print(1) }; 1 }");
        assert!(ws
            .iter()
            .any(|w| w.message.contains("unused variable: `x`")));
    }

    #[test]
    fn used_for_loop_variable_no_warning() {
        let ws = compile_warnings("fn test() { for x in [1, 2, 3] { print(x) }; 1 }");
        assert!(!ws
            .iter()
            .any(|w| w.message.contains("unused variable: `x`")));
    }

    // === W002: Unused mut ===

    #[test]
    fn unused_mut_warning() {
        let ws = compile_warnings("fn test() { mut y = 5; print(y) }");
        assert!(ws
            .iter()
            .any(|w| w.message.contains("never mutated") && w.message.contains("`y`")));
    }

    #[test]
    fn mut_used_and_mutated_no_warning() {
        let ws = compile_warnings("fn test() { mut y = 5; y = 10; print(y) }");
        assert!(!ws.iter().any(|w| w.message.contains("never mutated")));
    }

    #[test]
    fn unused_mut_not_read_gives_unused_not_mut_warning() {
        // If variable is both unused AND mut, we only warn about unused (more important)
        let ws = compile_warnings("fn test() { mut y = 5; 1 }");
        assert!(ws
            .iter()
            .any(|w| w.message.contains("unused variable: `y`")));
        assert!(!ws.iter().any(|w| w.message.contains("never mutated")));
    }

    // === W003: Unreachable code ===

    #[test]
    fn unreachable_code_after_return() {
        let ws = compile_warnings("fn test() { return 1; let x = 5; x }");
        assert!(ws.iter().any(|w| w.message.contains("unreachable code")));
    }

    #[test]
    fn unreachable_code_after_break() {
        let ws = compile_warnings("fn test() { for x in [1,2,3] { break; print(x) }; 1 }");
        assert!(ws.iter().any(|w| w.message.contains("unreachable code")));
    }

    #[test]
    fn no_unreachable_without_terminator() {
        let ws = compile_warnings("fn test() { let x = 1; let y = 2; x }");
        assert!(!ws.iter().any(|w| w.message.contains("unreachable")));
    }

    // === W004: Variable shadowing ===

    #[test]
    fn shadowing_same_scope() {
        let ws = compile_warnings("fn test() { let x = 1; let x = 2; x }");
        assert!(ws.iter().any(|w| w.message.contains("shadows")));
    }

    #[test]
    fn no_shadowing_different_scopes() {
        // Inner block creates new scope, no shadowing warning
        let ws = compile_warnings("fn test() { let x = 1; if true { let x = 2; x } else { x } }");
        assert!(!ws.iter().any(|w| w.message.contains("shadows")));
    }

    // === General ===

    #[test]
    fn warnings_have_correct_severity() {
        let ws = compile_warnings("fn test() { let x = 5; 1 }");
        for w in &ws {
            assert_eq!(w.severity, Severity::Warning);
        }
    }

    #[test]
    fn warnings_do_not_halt_compilation() {
        let mut compiler = Compiler::new();
        let result = compiler.compile("fn test() { let x = 5; 1 }");
        assert!(
            result.is_ok(),
            "compilation should succeed despite warnings"
        );
        assert!(!compiler.take_warnings().is_empty());
    }

    #[test]
    fn clean_code_no_warnings() {
        let ws = compile_warnings("fn test(x) { let y = x; print(y) }");
        assert!(ws.is_empty(), "expected no warnings, got: {:?}", ws);
    }

    // === W006: Type annotation mismatch ===

    #[test]
    fn w006_bool_annotation_on_field_literal() {
        let ws = compile_warnings("fn test() { let x: Bool = 0p42; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_field_annotation_on_string() {
        let ws = compile_warnings("fn test() { let x: Field = \"hello\"; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_field_annotation_on_bool() {
        let ws = compile_warnings("fn test() { let x: Field = true; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_bool_annotation_on_nil() {
        let ws = compile_warnings("fn test() { let x: Bool = nil; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_no_warning_field_on_field_lit() {
        let ws = compile_warnings("fn test() { let x: Field = 0p42; print(x) }");
        assert!(!ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_no_warning_field_on_int() {
        let ws = compile_warnings("fn test() { let x: Field = 42; print(x) }");
        assert!(!ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_no_warning_bool_on_bool() {
        let ws = compile_warnings("fn test() { let x: Bool = true; print(x) }");
        assert!(!ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_no_warning_dynamic_expression() {
        let ws = compile_warnings("fn f() { true }\nfn test() { let x: Bool = f(); print(x) }");
        assert!(!ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_mut_decl_also_warns() {
        let ws = compile_warnings("fn test() { mut x: Bool = 0p1; x = true; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    #[test]
    fn w006_scalar_annotation_on_array() {
        let ws = compile_warnings("fn test() { let x: Field = [1, 2, 3]; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }

    // === W007: Array size mismatch ===

    #[test]
    fn w007_array_size_mismatch() {
        let ws = compile_warnings("fn test() { let x: Field[3] = [1, 2]; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W007")));
    }

    #[test]
    fn w007_bool_array_size_mismatch() {
        let ws = compile_warnings("fn test() { let x: Bool[2] = [true, false, true]; print(x) }");
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W007")));
    }

    #[test]
    fn w007_no_warning_matching_size() {
        let ws = compile_warnings("fn test() { let x: Field[3] = [1, 2, 3]; print(x) }");
        assert!(!ws.iter().any(|w| w.code.as_deref() == Some("W007")));
    }

    #[test]
    fn w007_no_warning_on_non_array_value() {
        // Field[3] on a non-array value → W006, not W007
        let ws = compile_warnings("fn test() { let x: Field[3] = 42; print(x) }");
        assert!(!ws.iter().any(|w| w.code.as_deref() == Some("W007")));
        assert!(ws.iter().any(|w| w.code.as_deref() == Some("W006")));
    }
}

#[cfg(test)]
include!("tests/concurrency.rs");
