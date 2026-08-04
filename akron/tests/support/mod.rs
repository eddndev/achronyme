use akron::{CallFrame, RuntimeError, VM};
use akronc::Compiler;
use memory::{Closure, Function};

pub(super) fn loaded_vm(source: &str, stress_gc: bool) -> Result<VM, String> {
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(source)
        .map_err(|error| error.to_string())?;
    let main = compiler.compilers.last().expect("main compiler");

    let mut vm = VM::new();
    vm.stress_mode = stress_gc;
    vm.import_strings(compiler.interner.strings);
    for prototype in compiler.prototypes {
        vm.prototypes.push(
            vm.heap
                .alloc_function(prototype)
                .map_err(|e| e.to_string())?,
        );
    }
    let function = vm
        .heap
        .alloc_function(Function {
            name: "main".into(),
            arity: 0,
            max_slots: main.max_slots,
            chunk: bytecode,
            constants: main.constants.clone(),
            upvalue_info: Vec::new(),
            line_info: main.line_info.clone(),
        })
        .map_err(|e| e.to_string())?;
    let closure = vm
        .heap
        .alloc_closure(Closure {
            function,
            upvalues: Vec::new(),
        })
        .map_err(|e| e.to_string())?;
    vm.frames.push(CallFrame::new(closure, 0, 0));
    Ok(vm)
}

pub(super) fn run(source: &str) -> Result<VM, RuntimeError> {
    let mut vm = loaded_vm(source, false).expect("source compiles");
    vm.interpret()?;
    Ok(vm)
}

pub(super) fn global_int(vm: &VM, index: usize) -> i64 {
    vm.globals[index].value.as_int().unwrap_or_else(|| {
        panic!(
            "global {index} is not an int: {:?}",
            vm.globals[index].value
        )
    })
}
