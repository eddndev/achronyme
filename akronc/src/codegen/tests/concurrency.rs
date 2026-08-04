mod effect_diagnostic_tests {
    use super::super::*;

    #[test]
    fn task_effect_call_requires_explicit_await() {
        let mut compiler = Compiler::new();
        let error = compiler
            .compile("fn work() { concurrent { 1 } }\nwork()")
            .unwrap_err()
            .to_diagnostic();

        assert_eq!(error.code.as_deref(), Some("E021"));
        assert!(error.message.contains("must be prefixed with `await`"));
        assert!(error.notes.iter().any(|note| note.contains("work")));
    }

    #[test]
    fn transitive_host_effect_is_rejected_before_prove_lowering() {
        let mut compiler = Compiler::new();
        let error = compiler
            .compile("fn clocked() { time() }\nfn proof() { prove { clocked() } }")
            .unwrap_err()
            .to_diagnostic();

        assert_eq!(error.code.as_deref(), Some("E022"));
        assert!(error
            .message
            .contains("not allowed inside a `prove` context"));
        assert!(error.notes.iter().any(|note| note.contains("clocked")));
    }
}

#[cfg(test)]
mod resource_ownership_tests {
    use super::super::*;
    use akron::specs::{
        CancellationPolicy, CapabilitySet, EffectSet, NativeBehavior, NativeMeta, ResourceEffect,
        ResourceKind,
    };

    const TEST_NATIVES: &[NativeMeta] = &[
        NativeMeta {
            name: "open_file",
            arity: 1,
            effects: EffectSet::TASK.union(EffectSet::IO_FILE),
            capabilities: CapabilitySet::FILE_READ,
            behavior: NativeBehavior::Suspending,
            cancellation: CancellationPolicy::BeforeStart,
            resource: ResourceEffect::Creates(ResourceKind::File),
        },
        NativeMeta {
            name: "file_read",
            arity: 2,
            effects: EffectSet::TASK.union(EffectSet::IO_FILE),
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Suspending,
            cancellation: CancellationPolicy::BeforeStart,
            resource: ResourceEffect::Borrows(ResourceKind::File),
        },
        NativeMeta {
            name: "file_close",
            arity: 1,
            effects: EffectSet::IO_FILE,
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::Consumes(ResourceKind::File),
        },
        NativeMeta {
            name: "channel",
            arity: 1,
            effects: EffectSet::empty(),
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::Creates(ResourceKind::Channel),
        },
        NativeMeta {
            name: "channel_close",
            arity: 1,
            effects: EffectSet::empty(),
            capabilities: CapabilitySet::empty(),
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::Consumes(ResourceKind::Channel),
        },
    ];

    fn diagnostic(source: &str) -> achronyme_parser::Diagnostic {
        Compiler::with_extra_natives(TEST_NATIVES)
            .compile(source)
            .unwrap_err()
            .to_diagnostic()
    }

    #[test]
    fn parent_reuse_after_spawn_transfer_is_a_compile_error() {
        let error = diagnostic(
            "fn read_one(file) { await file_read(file, 1) }\n\
             let file = await open_file(\"input\")\n\
             concurrent { spawn read_one(file); await file_read(file, 1) }",
        );

        assert_eq!(error.code.as_deref(), Some("E023"));
        assert!(error.message.contains("file"));
        assert!(error.message.contains("moved"));
        assert!(!error.labels.is_empty());
    }

    #[test]
    fn aliasing_an_owned_resource_moves_the_original_binding() {
        let error = diagnostic(
            "let file = await open_file(\"input\")\n\
             let alias = file\n\
             await file_read(file, 1)",
        );

        assert_eq!(error.code.as_deref(), Some("E023"));
        assert!(error.message.contains("file"));
    }

    #[test]
    fn use_after_explicit_close_is_a_compile_error() {
        let error = diagnostic(
            "let file = await open_file(\"input\")\n\
             file_close(file)\n\
             await file_read(file, 1)",
        );

        assert_eq!(error.code.as_deref(), Some("E023"));
        assert!(error.message.contains("closed"));
    }

    #[test]
    fn a_temporary_borrow_cannot_be_spawned_as_a_child_task() {
        let error = diagnostic(
            "let file = await open_file(\"input\")\n\
             concurrent { let task = spawn file_read(file, 1); await task }",
        );

        assert_eq!(error.code.as_deref(), Some("E024"));
        assert!(error.message.contains("borrow"));
    }

    #[test]
    fn one_way_transfer_to_a_child_compiles() {
        let source = "fn read_one(file) { await file_read(file, 1) }\n\
                      let file = await open_file(\"input\")\n\
                      concurrent { let task = spawn read_one(file); await task }";
        Compiler::with_extra_natives(TEST_NATIVES)
            .compile(source)
            .unwrap();
    }

    #[test]
    fn a_move_on_either_branch_is_unavailable_after_the_branch() {
        let error = diagnostic(
            "fn read_one(file) { await file_read(file, 1) }\n\
             let file = await open_file(\"input\")\n\
             concurrent {\n\
               if true { spawn read_one(file) } else { 0 }\n\
               await file_read(file, 1)\n\
             }",
        );

        assert_eq!(error.code.as_deref(), Some("E023"));
    }

    #[test]
    fn consuming_user_function_summary_closes_the_callers_binding() {
        let error = diagnostic(
            "fn close_it(file) { file_close(file) }\n\
             let file = await open_file(\"input\")\n\
             close_it(file)\n\
             await file_read(file, 1)",
        );

        assert_eq!(error.code.as_deref(), Some("E023"));
        assert!(error.message.contains("closed"));
    }

    #[test]
    fn resource_return_summary_tracks_a_wrapper_result() {
        let source = "fn open_it() { await open_file(\"input\") }\n\
                      let file = await open_it()\n\
                      await file_read(file, 1)\n\
                      file_close(file)";
        Compiler::with_extra_natives(TEST_NATIVES)
            .compile(source)
            .unwrap();
    }

    #[test]
    fn shared_channel_aliases_do_not_move_but_close_invalidates_the_binding() {
        let error = diagnostic(
            "let first = channel(1)\n\
             let second = first\n\
             channel_close(first)\n\
             let again = first",
        );

        assert_eq!(error.code.as_deref(), Some("E023"));
        assert!(error.message.contains("closed"));
    }
}

#[cfg(test)]
mod mutable_capture_tests {
    use super::super::*;

    #[test]
    fn multiple_children_capturing_outer_mutable_state_emit_a_warning() {
        let source = "mut total = 0\n\
                      fn increment() { total = total + 1 }\n\
                      concurrent {\n\
                        let first = spawn increment()\n\
                        let second = spawn increment()\n\
                        await first\n\
                        await second\n\
                      }";
        let mut compiler = Compiler::new();
        compiler.compile(source).unwrap();
        let warnings = compiler.take_warnings();

        assert!(warnings.iter().any(|warning| {
            warning.code.as_deref() == Some("W011") && warning.message.contains("total")
        }));
    }

    #[test]
    fn one_child_capture_does_not_warn() {
        let source = "mut total = 0\n\
                      fn increment() { total = total + 1 }\n\
                      concurrent { let task = spawn increment(); await task }";
        let mut compiler = Compiler::new();
        compiler.compile(source).unwrap();
        let warnings = compiler.take_warnings();

        assert!(!warnings
            .iter()
            .any(|warning| warning.code.as_deref() == Some("W011")));
    }
}

#[cfg(test)]
mod suggestion_tests {
    use super::super::*;

    fn compile_error_message(source: &str) -> String {
        let mut compiler = Compiler::new();
        match compiler.compile(source) {
            Ok(_) => panic!("expected compilation to fail"),
            Err(e) => format!("{e}"),
        }
    }

    #[test]
    fn suggests_similar_local_variable() {
        let msg = compile_error_message("fn test() { let count = 5; cout }");
        assert!(msg.contains("undefined variable"), "got: {msg}");
    }

    #[test]
    fn suggests_similar_function_name() {
        let msg = compile_error_message("fn compute() { 1 }\ncompue()");
        assert!(msg.contains("undefined variable"), "got: {msg}");
    }

    #[test]
    fn no_suggestion_for_completely_different_name() {
        let msg = compile_error_message("fn test() { let x = 5; zzzzzz }");
        assert!(msg.contains("undefined variable"), "got: {msg}");
    }

    #[test]
    fn suggestion_in_diagnostic_error() {
        let mut compiler = Compiler::new();
        let err = compiler
            .compile("fn test() { let count = 5; cout }")
            .unwrap_err();
        let diag = err.to_diagnostic();
        assert!(diag.message.contains("undefined variable"));
        // The suggestion should be structured data
        assert!(
            !diag.suggestions.is_empty(),
            "diagnostic should have a suggestion from `cout` to `count`: {diag:?}"
        );
    }

    #[test]
    fn suggestion_for_one_char_typo() {
        let msg = compile_error_message("fn test() { let value = 42; valye }");
        assert!(msg.contains("undefined variable"), "got: {msg}");
    }

    #[test]
    fn suggestion_for_assignment_target() {
        let msg = compile_error_message("fn test() { mut total = 0; totol = 5; total }");
        assert!(msg.contains("undefined variable"), "got: {msg}");
    }
}

#[cfg(test)]
mod kwarg_validation_tests {
    use super::super::*;

    fn compile_error_message(source: &str) -> String {
        let mut compiler = Compiler::new();
        match compiler.compile(source) {
            Ok(_) => panic!("expected compilation to fail"),
            Err(e) => format!("{e}"),
        }
    }

    fn compile_ok(source: &str) -> Vec<u32> {
        let mut compiler = Compiler::new();
        compiler.compile(source).expect("should compile")
    }

    #[test]
    fn valid_kwargs_compile_ok() {
        // Circuit with params, called with correct keyword args
        let src = r#"
            circuit adder(a: Public, b: Witness) {
                assert_eq(a, b)
            }
            adder(a: 1, b: 2)
        "#;
        compile_ok(src);
    }

    #[test]
    fn unknown_kwarg_errors() {
        let src = r#"
            circuit adder(a: Public, b: Witness) {
                assert_eq(a, b)
            }
            adder(x: 1, b: 2)
        "#;
        let msg = compile_error_message(src);
        assert!(msg.contains("unknown keyword argument `x`"), "got: {msg}");
    }

    #[test]
    fn typo_kwarg_suggests_correct_name() {
        let src = r#"
            circuit eligibility(secret: Witness, threshold: Public) {
                assert_eq(secret, threshold)
            }
            eligibility(secrt: 42, threshold: 100)
        "#;
        let msg = compile_error_message(src);
        assert!(
            msg.contains("unknown keyword argument `secrt`"),
            "got: {msg}"
        );
        assert!(msg.contains("did you mean `secret`"), "got: {msg}");
    }

    #[test]
    fn completely_wrong_kwarg_no_suggestion() {
        let src = r#"
            circuit foo(a: Public, b: Witness) {
                assert_eq(a, b)
            }
            foo(zzzzz: 1, b: 2)
        "#;
        let msg = compile_error_message(src);
        assert!(
            msg.contains("unknown keyword argument `zzzzz`"),
            "got: {msg}"
        );
        assert!(!msg.contains("did you mean"), "got: {msg}");
    }
}
