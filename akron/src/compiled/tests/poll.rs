use crate::{CallFrame, RuntimeError, VM};

use super::super::{
    runtime_api, RuntimeContext, STATUS_BAILOUT_REQUIRED, STATUS_INVALID_ARGUMENT, STATUS_OK,
    STATUS_RUNTIME_ERROR, STATUS_SLOW_PATH_REQUIRED,
};

#[test]
fn poll_updates_location_and_consumes_one_unit_of_fuel() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.instruction_budget = 1;

    let error = {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        assert_eq!(unsafe { (runtime_api().poll)(opaque, 0, 9) }, STATUS_OK);
        let status = unsafe { (runtime_api().poll)(opaque, 0, 10) };
        assert_eq!(status, STATUS_RUNTIME_ERROR);
        context.finish(status).unwrap_err()
    };

    assert!(matches!(error, RuntimeError::InstructionBudgetExhausted));
    assert_eq!(vm.frames[0].ip, 11);
    assert_eq!(vm.instruction_budget, 0);
}

#[test]
fn block_poll_precharges_a_complete_block() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.instruction_budget = 10;

    {
        let mut context = RuntimeContext::new(&mut vm);
        let status = unsafe { (runtime_api().poll_block)(context.as_opaque(), 0, 7, 4) };
        assert_eq!(status, STATUS_OK);
        context.finish(status).unwrap();
    }

    assert_eq!(vm.frames[0].ip, 11);
    assert_eq!(vm.instruction_budget, 6);
}

#[test]
fn block_poll_requests_exact_slow_path_without_mutating_state() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.frames[0].ip = 7;
    vm.instruction_budget = 3;

    {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().poll_block)(context.as_opaque(), 0, 7, 4) },
            STATUS_BAILOUT_REQUIRED
        );
    }

    assert_eq!(vm.frames[0].ip, 7);
    assert_eq!(vm.instruction_budget, 3);

    vm.stress_mode = true;
    vm.instruction_budget = 4;
    {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().poll_block)(context.as_opaque(), 0, 7, 4) },
            STATUS_BAILOUT_REQUIRED
        );
    }
    assert_eq!(vm.frames[0].ip, 7);
    assert_eq!(vm.instruction_budget, 4);
}

#[test]
fn block_poll_services_a_pending_gc_request_without_bailing_out() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.heap.request_gc = true;
    let collections = vm.heap.stats.collections;

    {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        assert_eq!(
            unsafe { (runtime_api().poll_block)(opaque, 0, 0, 1023) },
            STATUS_OK
        );
        assert_eq!(
            unsafe { (runtime_api().poll_block)(opaque, 0, 1023, 1) },
            STATUS_OK
        );
        context.finish(STATUS_OK).unwrap();
    }

    assert!(!vm.heap.request_gc);
    assert_eq!(vm.heap.stats.collections, collections + 1);
    assert_eq!(vm.frames[0].ip, 1024);
}

#[test]
fn fast_block_poll_commits_normal_work_without_entering_the_protected_helper() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.frames[0].ip = 7;
    vm.instruction_budget = 10;

    let stats = {
        let mut context = RuntimeContext::new(&mut vm);
        let status = unsafe { (runtime_api().poll_fast_block)(context.as_opaque(), 0, 7, 4, 4) };
        assert_eq!(status, STATUS_OK);
        let stats = context.stats();
        context.finish(status).unwrap();
        stats
    };

    assert_eq!(vm.frames[0].ip, 11);
    assert_eq!(vm.instruction_budget, 6);
    assert_eq!(stats.block_polls, 1);
    assert_eq!(stats.fast_poll_hits, 1);
    assert_eq!(stats.slow_poll_entries, 0);
    assert_eq!(stats.direct_instructions, 4);
}

#[test]
fn fast_block_poll_defers_low_fuel_without_mutating_runtime_state() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.frames[0].ip = 7;
    vm.instruction_budget = 3;

    let mut context = RuntimeContext::new(&mut vm);
    assert_eq!(
        unsafe { (runtime_api().poll_fast_block)(context.as_opaque(), 0, 7, 4, 4) },
        STATUS_SLOW_PATH_REQUIRED
    );
    assert_eq!(context.stats().block_polls, 0);
    assert_eq!(context.stats().fast_poll_hits, 0);
    assert_eq!(
        unsafe { (runtime_api().poll_tier1_block)(context.as_opaque(), 0, 7, 4, 4) },
        STATUS_BAILOUT_REQUIRED
    );
    let stats = context.stats();

    assert_eq!(vm.frames[0].ip, 7);
    assert_eq!(vm.instruction_budget, 3);
    assert_eq!(stats.block_polls, 1);
    assert_eq!(stats.fast_poll_hits, 0);
    assert_eq!(stats.slow_poll_entries, 1);
    assert_eq!(stats.slow_paths, 1);
}

#[test]
fn fast_block_poll_defers_pending_gc_to_the_protected_helper() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.heap.request_gc = true;
    let collections = vm.heap.stats.collections;

    let stats = {
        let mut context = RuntimeContext::new(&mut vm);
        assert_eq!(
            unsafe { (runtime_api().poll_fast_block)(context.as_opaque(), 0, 0, 1024, 1024) },
            STATUS_SLOW_PATH_REQUIRED
        );
        assert_eq!(
            unsafe { (runtime_api().poll_tier1_block)(context.as_opaque(), 0, 0, 1024, 1024) },
            STATUS_OK
        );
        let stats = context.stats();
        context.finish(STATUS_OK).unwrap();
        stats
    };

    assert!(!vm.heap.request_gc);
    assert_eq!(vm.heap.stats.collections, collections + 1);
    assert_eq!(stats.fast_poll_hits, 0);
    assert_eq!(stats.slow_poll_entries, 1);
}

#[test]
fn fast_block_poll_rejects_invalid_block_accounting() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    let mut context = RuntimeContext::new(&mut vm);

    assert_eq!(
        unsafe { (runtime_api().poll_fast_block)(context.as_opaque(), 0, 7, 4, 5) },
        STATUS_INVALID_ARGUMENT
    );
    assert_eq!(context.stats().block_polls, 0);
}

#[test]
fn block_refund_restores_fuel_and_commits_the_requested_ip() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.instruction_budget = 10;

    {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        assert_eq!(
            unsafe { (runtime_api().poll_block)(opaque, 0, 7, 4) },
            STATUS_OK
        );
        assert_eq!(
            unsafe { (runtime_api().refund_block)(opaque, 0, 9, 2) },
            STATUS_OK
        );
        context.finish(STATUS_OK).unwrap();
    }

    assert_eq!(vm.frames[0].ip, 9);
    assert_eq!(vm.instruction_budget, 8);
}

#[test]
fn tier1_block_poll_records_direct_work_and_exact_refunds() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.instruction_budget = 10;

    let stats = {
        let mut context = RuntimeContext::new(&mut vm);
        let opaque = context.as_opaque();
        assert_eq!(
            unsafe { (runtime_api().poll_tier1_block)(opaque, 0, 7, 4, 4) },
            STATUS_OK
        );
        assert_eq!(
            unsafe { (runtime_api().refund_block)(opaque, 0, 9, 2) },
            STATUS_OK
        );
        let stats = context.stats();
        context.finish(STATUS_OK).unwrap();
        stats
    };

    assert_eq!(stats.block_polls, 1);
    assert_eq!(stats.fast_poll_hits, 0);
    assert_eq!(stats.slow_poll_entries, 1);
    assert_eq!(stats.direct_instructions, 2);
    assert_eq!(stats.slow_paths, 0);
    assert_eq!(stats.known_call_fast_hits, 0);
    assert_eq!(stats.known_call_fast_misses, 0);
    assert_eq!(stats.specialization_hits, 0);
    assert_eq!(stats.specialization_misses, 0);
}

#[test]
fn tier1_block_poll_counts_exact_slow_paths_without_direct_work() {
    let mut vm = VM::new();
    vm.frames.push(CallFrame::new(0, 0, 0));
    vm.instruction_budget = 3;

    let mut context = RuntimeContext::new(&mut vm);
    assert_eq!(
        unsafe { (runtime_api().poll_tier1_block)(context.as_opaque(), 0, 7, 4, 4) },
        STATUS_BAILOUT_REQUIRED
    );
    let stats = context.stats();

    assert_eq!(stats.block_polls, 1);
    assert_eq!(stats.fast_poll_hits, 0);
    assert_eq!(stats.slow_poll_entries, 1);
    assert_eq!(stats.direct_instructions, 0);
    assert_eq!(stats.slow_paths, 1);
}
