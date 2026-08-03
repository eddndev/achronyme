use super::*;

#[test]
fn capabilities_detect_verify_proof_global_usage() {
    let registry = resolve::BuiltinRegistry::default();
    let verify_index = registry
        .lookup("verify_proof")
        .and_then(|entry| entry.vm_fn)
        .unwrap()
        .as_u32() as u16;
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abx(OpCode::GetGlobal.as_u8(), 0, verify_index),
        encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
    ];
    program.main.line_info = vec![1; 2];

    assert!(program
        .derived_capabilities()
        .contains(ProgramCapabilities::VERIFY));
}

#[test]
fn executable_roundtrip_preserves_verify_capability() {
    let registry = resolve::BuiltinRegistry::default();
    let verify_index = registry
        .lookup("verify_proof")
        .and_then(|entry| entry.vm_fn)
        .unwrap()
        .as_u32() as u16;
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abx(OpCode::GetGlobal.as_u8(), 0, verify_index),
        encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
    ];
    program.main.line_info = vec![1; 2];
    program.capabilities = program.derived_capabilities();
    let mut bytes = Vec::new();
    program.write_executable(&mut bytes).unwrap();

    let decoded = CompiledProgram::read_executable(&mut bytes.as_slice()).unwrap();

    assert!(decoded.capabilities.contains(ProgramCapabilities::VERIFY));
}

#[test]
fn generic_user_call_does_not_request_native_authority() {
    let mut program = minimal_program();
    program.functions.push(Function {
        name: "pure".to_string(),
        arity: 0,
        max_slots: 1,
        chunk: vec![encode_abc(OpCode::Return.as_u8(), 0, 0, 0)],
        constants: Vec::new(),
        upvalue_info: Vec::new(),
        line_info: vec![1],
    });
    program.main.max_slots = 2;
    program.main.chunk = vec![
        encode_abx(OpCode::Closure.as_u8(), 0, 0),
        encode_abc(OpCode::Call.as_u8(), 1, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
    ];
    program.main.line_info = vec![1; 3];

    assert!(!program
        .derived_capabilities()
        .contains(ProgramCapabilities::NATIVE_CALLS));
}

#[test]
fn registered_file_native_derives_precise_capability() {
    let mut program = minimal_program();
    let native_index = first_extra_native_index();
    program.register_extra_natives(
        native_index,
        &[NativeMeta {
            name: "read_file",
            arity: 1,
            effects: EffectSet::IO_FILE,
            capabilities: CapabilitySet::FILE_READ,
            behavior: NativeBehavior::Blocking,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::None,
        }],
    );
    program.main.max_slots = 2;
    program.main.chunk = vec![
        encode_abx(OpCode::GetGlobal.as_u8(), 0, native_index),
        encode_abc(OpCode::Call.as_u8(), 1, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
    ];
    program.main.line_info = vec![1; 3];

    let capabilities = program.derived_capabilities();
    assert!(capabilities.contains(ProgramCapabilities::NATIVE_CALLS));
    assert!(capabilities.contains(ProgramCapabilities::FILE_IO));
}

#[test]
fn requested_effects_and_host_capabilities_ignore_unused_natives() {
    let mut program = minimal_program();
    program.main.max_slots = 1;
    program.main.chunk = vec![
        encode_abx(OpCode::GetGlobal.as_u8(), 0, 4),
        encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
    ];
    program.native_metadata.insert(
        4,
        ProgramNativeMetadata {
            effects: EffectSet::TASK | EffectSet::IO_FILE,
            capabilities: CapabilitySet::FILE_READ,
            behavior: NativeBehavior::Suspending,
            cancellation: CancellationPolicy::BeforeStart,
            resource: ResourceEffect::None,
        },
    );
    program.native_metadata.insert(
        5,
        ProgramNativeMetadata {
            effects: EffectSet::IO_NETWORK,
            capabilities: CapabilitySet::NETWORK_CONNECT,
            behavior: NativeBehavior::Immediate,
            cancellation: CancellationPolicy::None,
            resource: ResourceEffect::None,
        },
    );

    assert_eq!(
        program.requested_effects(),
        EffectSet::TASK | EffectSet::IO_FILE
    );
    assert_eq!(
        program.requested_host_capabilities(),
        CapabilitySet::FILE_READ
    );
}

#[test]
fn executable_roundtrip_preserves_extra_native_metadata() {
    let mut program = minimal_program();
    let native_index = first_extra_native_index();
    program.register_extra_natives(
        native_index,
        &[NativeMeta {
            name: "read_file",
            arity: 1,
            effects: EffectSet::IO_FILE,
            capabilities: CapabilitySet::FILE_READ,
            behavior: NativeBehavior::Blocking,
            cancellation: CancellationPolicy::BeforeStart,
            resource: ResourceEffect::None,
        }],
    );
    program.main.max_slots = 1;
    program.main.chunk = vec![
        encode_abx(OpCode::GetGlobal.as_u8(), 0, native_index),
        encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
    ];
    program.main.line_info = vec![1; 2];
    program.capabilities = program.derived_capabilities();

    let mut bytes = Vec::new();
    program.write_executable(&mut bytes).unwrap();
    let decoded = CompiledProgram::read_executable(&mut bytes.as_slice()).unwrap();

    assert_eq!(decoded.native_metadata, program.native_metadata);
    assert!(decoded.capabilities.contains(ProgramCapabilities::FILE_IO));
}

#[test]
fn structured_task_opcodes_derive_task_capability() {
    let mut program = minimal_program();
    program.main.max_slots = 1;
    program.main.chunk = vec![
        encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
        encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
    ];
    program.main.line_info = vec![1; 3];

    assert!(program
        .derived_capabilities()
        .contains(ProgramCapabilities::TASKS));
}

#[test]
fn validation_rejects_return_with_open_task_scope() {
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
    ];
    program.main.line_info = vec![1; 2];
    program.capabilities |= ProgramCapabilities::TASKS;

    let error = program.validate().unwrap_err().to_string();
    assert!(error.contains("returns with 1 open task scope"), "{error}");
}

#[test]
fn validation_rejects_scope_exit_without_enter() {
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
    ];
    program.main.line_info = vec![1; 2];
    program.capabilities |= ProgramCapabilities::TASKS;

    let error = program.validate().unwrap_err().to_string();
    assert!(error.contains("before entering one"), "{error}");
}

#[test]
fn validation_rejects_branch_join_with_mismatched_scope_depth() {
    let mut program = minimal_program();
    program.main.chunk = vec![
        encode_abx(OpCode::JumpIfFalse.as_u8(), 0, 2),
        encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
    ];
    program.main.line_info = vec![1; 3];
    program.capabilities |= ProgramCapabilities::TASKS;

    let error = program.validate().unwrap_err().to_string();
    assert!(
        error.contains("inconsistent task scope depths")
            || error.contains("returns with 1 open task scope"),
        "{error}"
    );
}

#[test]
fn validation_rejects_out_of_range_spawn_arguments() {
    let mut program = minimal_program();
    program.main.max_slots = 2;
    program.main.chunk = vec![
        encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
        encode_abc(OpCode::Spawn.as_u8(), 0, 1, 1),
        encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
        encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
    ];
    program.main.line_info = vec![1; 4];
    program.capabilities |= ProgramCapabilities::TASKS;

    let error = program.validate().unwrap_err().to_string();
    assert!(error.contains("out-of-range Spawn registers"), "{error}");
}

#[test]
fn validation_rejects_task_handles_in_constant_pool() {
    let mut program = minimal_program();
    program.main.constants = vec![Value::task(0)];

    let error = program.validate().unwrap_err().to_string();
    assert!(error.contains("unsupported type"), "{error}");
}

fn task_program(chunk: Vec<u32>, max_slots: u16) -> CompiledProgram {
    let mut program = minimal_program();
    program.main.max_slots = max_slots;
    program.main.line_info = vec![1; chunk.len()];
    program.main.chunk = chunk;
    program.capabilities |= ProgramCapabilities::TASKS;
    program
}

#[test]
fn validation_rejects_returning_a_task_handle() {
    let program = task_program(
        vec![
            encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Spawn.as_u8(), 0, 1, 0),
            encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 1, 0),
        ],
        2,
    );

    let error = program.validate().unwrap_err().to_string();
    assert!(
        error.contains("task handle escapes through Return"),
        "{error}"
    );
}

#[test]
fn validation_rejects_storing_a_task_handle_in_a_global_or_container() {
    for escaping in [
        encode_abx(OpCode::DefGlobalLet.as_u8(), 0, 20),
        encode_abc(OpCode::BuildList.as_u8(), 2, 0, 1),
    ] {
        let program = task_program(
            vec![
                encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
                encode_abc(OpCode::Spawn.as_u8(), 0, 1, 0),
                escaping,
                encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
                encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
            ],
            3,
        );

        let error = program.validate().unwrap_err().to_string();
        assert!(error.contains("task handle escapes"), "{error}");
    }
}

#[test]
fn validation_rejects_capturing_a_task_handle_in_a_closure() {
    let mut program = task_program(
        vec![
            encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Spawn.as_u8(), 0, 1, 0),
            encode_abx(OpCode::Closure.as_u8(), 2, 0),
            encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Return.as_u8(), 0, 0, 0),
        ],
        3,
    );
    program.functions.push(Function {
        name: "captures_task".to_string(),
        arity: 0,
        max_slots: 1,
        chunk: vec![encode_abc(OpCode::Return.as_u8(), 0, 0, 0)],
        constants: Vec::new(),
        upvalue_info: vec![1, 0],
        line_info: vec![1],
    });

    let error = program.validate().unwrap_err().to_string();
    assert!(
        error.contains("task handle escapes through Closure"),
        "{error}"
    );
}

#[test]
fn validation_rejects_awaiting_a_non_task_register() {
    let program = task_program(
        vec![
            encode_abc(OpCode::ScopeEnter.as_u8(), 0, 0, 0),
            encode_abc(OpCode::LoadNil.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Await.as_u8(), 1, 0, 0),
            encode_abc(OpCode::ScopeExit.as_u8(), 0, 0, 0),
            encode_abc(OpCode::Return.as_u8(), 1, 1, 0),
        ],
        2,
    );

    let error = program.validate().unwrap_err().to_string();
    assert!(error.contains("AWAIT reads a non-task register"), "{error}");
}
