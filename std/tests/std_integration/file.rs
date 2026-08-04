#[cfg(feature = "io")]
#[test]
fn file_io_requires_an_explicit_root_grant() {
    use achronyme_std::io::io_impl::{native_read_file, native_write_file};
    use akron::RuntimeError;
    use memory::Value;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.txt");
    let output = directory.path().join("output.txt");
    std::fs::write(&input, "hello").unwrap();

    let mut vm = akron::VM::new();
    let input_handle = vm
        .heap
        .alloc_string(input.to_string_lossy().into_owned())
        .unwrap();
    assert!(matches!(
        native_read_file(&mut vm, &[Value::string(input_handle)]),
        Err(RuntimeError::CapabilityDenied(_))
    ));

    vm.host_policy.allow_read_root(directory.path()).unwrap();
    vm.host_policy.allow_write_root(directory.path()).unwrap();
    let read = native_read_file(&mut vm, &[Value::string(input_handle)]).unwrap();
    assert_eq!(
        vm.heap.get_string(read.as_handle().unwrap()).unwrap(),
        "hello"
    );

    let output_handle = vm
        .heap
        .alloc_string(output.to_string_lossy().into_owned())
        .unwrap();
    let contents_handle = vm.heap.alloc_string("written".to_string()).unwrap();
    native_write_file(
        &mut vm,
        &[Value::string(output_handle), Value::string(contents_handle)],
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(output).unwrap(), "written");
}
#[cfg(feature = "io")]
#[test]
fn awaited_file_read_runs_through_structured_task_and_artifact_metadata() {
    use akron::{CompiledProgram, ProgramCapabilities};
    use akronc::CompileOptions;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.txt");
    std::fs::write(&input, "async hello").unwrap();
    let source = format!(
        "let result = await read_file({:?})",
        input.to_string_lossy()
    );
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();
    let result_index = compiler.global_symbols["result"].index as usize;
    assert!(program.capabilities.contains(ProgramCapabilities::TASKS));
    assert!(program.capabilities.contains(ProgramCapabilities::FILE_IO));

    let mut bytes = Vec::new();
    program.write_executable(&mut bytes).unwrap();
    let decoded = CompiledProgram::read_executable(&mut bytes.as_slice()).unwrap();
    assert_eq!(decoded.native_metadata, program.native_metadata);

    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_read_root(directory.path()).unwrap();
    vm.load_program(decoded).unwrap();
    vm.interpret().unwrap();

    let value = vm.globals[result_index].value;
    assert_eq!(
        vm.heap.get_string(value.as_handle().unwrap()).unwrap(),
        "async hello"
    );
    assert_eq!(vm.active_task_scope_count(), 0);
}
#[cfg(feature = "io")]
#[test]
fn compatibility_file_read_remains_a_sequential_call() {
    use akron::specs::EffectSet;
    use akronc::CompileOptions;

    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(
            "let contents = read_file(\"input.txt\")",
            &CompileOptions::default(),
        )
        .unwrap();

    assert!(program.requested_effects().contains(EffectSet::IO_FILE));
    assert!(!program.requested_effects().contains(EffectSet::TASK));
}

#[cfg(feature = "io")]
#[test]
fn owned_file_resource_writes_and_closes_through_structured_tasks() {
    use akron::specs::{ResourceEffect, ResourceKind};
    use akronc::CompileOptions;

    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("owned.txt");
    let source = format!(
        "let file = await create_file({:?})\n\
         let written = await file_write(file, \"owned hello\")\n\
         await file_close(file)",
        output.to_string_lossy()
    );
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();
    let written = compiler.global_symbols["written"].index as usize;

    let create = table
        .iter()
        .find(|meta| meta.name == "create_file")
        .unwrap();
    let write = table.iter().find(|meta| meta.name == "file_write").unwrap();
    assert_eq!(create.resource, ResourceEffect::Creates(ResourceKind::File));
    assert_eq!(write.resource, ResourceEffect::Borrows(ResourceKind::File));

    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_write_root(directory.path()).unwrap();
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();

    assert_eq!(vm.globals[written].value.as_int(), Some(11));
    assert_eq!(std::fs::read_to_string(output).unwrap(), "owned hello");
}

#[cfg(feature = "io")]
#[test]
fn resource_transfer_rejects_parent_reuse_at_compile_time() {
    use akronc::CompileOptions;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.txt");
    std::fs::write(&input, "abc").unwrap();
    let source = format!(
        "fn read_one(file) {{ await file_read(file, 1) }}\n\
         let file = await open_file({:?})\n\
         concurrent {{ spawn read_one(file); await file_read(file, 1) }}",
        input.to_string_lossy()
    );
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let error = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap_err()
        .to_diagnostic();

    assert_eq!(error.code.as_deref(), Some("E023"));
    assert!(error.message.contains("file"));
    assert!(error.message.contains("moved"));
}

#[cfg(feature = "io")]
#[test]
fn unclosed_root_resource_is_closed_when_program_finishes() {
    use akronc::CompileOptions;
    use memory::ValueResourceKind;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.txt");
    std::fs::write(&input, "abc").unwrap();
    let source = format!("let file = await open_file({:?})", input.to_string_lossy());
    let table = achronyme_std::std_native_table();
    let mut compiler = akronc::Compiler::with_extra_natives(&table);
    let program = compiler
        .compile_program(&source, &CompileOptions::default())
        .unwrap();
    let file_index = compiler.global_symbols["file"].index as usize;
    let mut vm = akron::VM::new();
    for module in achronyme_std::std_modules() {
        vm.register_module(&*module).unwrap();
    }
    vm.host_policy.allow_read_root(directory.path()).unwrap();
    vm.load_program(program).unwrap();
    vm.interpret().unwrap();

    let file = vm.globals[file_index].value;
    assert!(file.is_resource());
    assert!(vm.require_resource(file, ValueResourceKind::File).is_err());
}
