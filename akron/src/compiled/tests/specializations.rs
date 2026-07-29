use memory::Value;

use crate::compiled::{
    runtime_api, RuntimeContext, STATUS_OK, STATUS_RUNTIME_ERROR, STATUS_SPECIALIZATION_MISS,
};
use crate::opcode::instruction::encode_abc;
use crate::{OpCode, RuntimeError, VM};

#[test]
fn list_push_specialization_reuses_heap_accounting_and_counts_two_boundaries() {
    let mut vm = VM::new();
    let handle = vm.heap.alloc_list(Vec::new()).unwrap();
    vm.stack[2] = Value::list(handle);
    vm.stack[3] = Value::int(7);
    vm.stack[1] = Value::int(99);
    let instruction = encode_abc(OpCode::MethodCall.as_u8(), 1, 2, 1);

    let stats;
    {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().list_push)(context.as_opaque(), 0, instruction) },
            STATUS_OK
        );
        stats = context.stats();
        context.finish(STATUS_OK).unwrap();
    }

    assert_eq!(vm.heap.get_list(handle).unwrap(), &[Value::int(7)]);
    assert!(vm.stack[1].is_nil());
    assert_eq!(stats.specialization_hits, 2);
    assert_eq!(stats.specialization_misses, 0);
}

#[test]
fn list_push_specialization_misses_without_mutating_the_receiver_or_destination() {
    let mut vm = VM::new();
    vm.stack[2] = Value::int(4);
    vm.stack[3] = Value::int(7);
    vm.stack[1] = Value::int(99);
    let instruction = encode_abc(OpCode::MethodCall.as_u8(), 1, 2, 1);

    let stats;
    {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().list_push)(context.as_opaque(), 0, instruction) },
            STATUS_SPECIALIZATION_MISS
        );
        stats = context.stats();
        context.finish(STATUS_OK).unwrap();
    }

    assert_eq!(vm.stack[2], Value::int(4));
    assert_eq!(vm.stack[1], Value::int(99));
    assert_eq!(stats.specialization_hits, 0);
    assert_eq!(stats.specialization_misses, 1);
}

#[test]
fn list_index_specialization_preserves_success_and_index_errors() {
    let mut vm = VM::new();
    let handle = vm
        .heap
        .alloc_list(vec![Value::int(11), Value::int(22)])
        .unwrap();
    vm.stack[0] = Value::list(handle);
    vm.stack[1] = Value::int(1);
    let instruction = encode_abc(OpCode::GetIndex.as_u8(), 2, 0, 1);

    let stats;
    {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().list_index)(context.as_opaque(), 0, instruction) },
            STATUS_OK
        );
        stats = context.stats();
        context.finish(STATUS_OK).unwrap();
    }
    assert_eq!(vm.stack[2], Value::int(22));
    assert_eq!(stats.specialization_hits, 1);

    vm.stack[1] = Value::int(-1);
    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let status = unsafe { (runtime_api().list_index)(context.as_opaque(), 0, instruction) };
        assert_eq!(status, STATUS_RUNTIME_ERROR);
        context.finish(status).unwrap_err()
    };
    assert!(matches!(error, RuntimeError::OutOfBounds(_)));
}

#[test]
fn list_index_specialization_misses_for_non_list_receivers() {
    let mut vm = VM::new();
    vm.stack[0] = Value::int(11);
    vm.stack[1] = Value::int(0);
    vm.stack[2] = Value::int(99);
    let instruction = encode_abc(OpCode::GetIndex.as_u8(), 2, 0, 1);

    let stats;
    {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().list_index)(context.as_opaque(), 0, instruction) },
            STATUS_SPECIALIZATION_MISS
        );
        stats = context.stats();
        context.finish(STATUS_OK).unwrap();
    }
    assert_eq!(vm.stack[2], Value::int(99));
    assert_eq!(stats.specialization_hits, 0);
    assert_eq!(stats.specialization_misses, 1);
}
