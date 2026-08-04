//! Bounded channels, permit pools, and cooperative task helpers.

use ach_macros::{ach_module, ach_native};
use akron::error::RuntimeError;
use akron::machine::VM;
use akron::{NativeAsyncRequest, NativeChannelRequest};
use memory::Value;
use std::time::Duration;

const MAX_CHANNEL_CAPACITY: usize = 65_535;

#[ach_module(name = "task")]
pub mod task_impl {
    use super::*;

    #[ach_native(name = "channel", arity = 1, resource = "creates:channel")]
    pub fn native_channel(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
        let capacity = capacity_arg(args, "channel", true)?;
        vm.create_channel_resource(capacity)
    }

    #[ach_native(
        name = "yield_now",
        arity = 0,
        effects = "task",
        behavior = "suspending",
        cancellation = "before-start",
        async_adapter = "start_yield_now"
    )]
    pub fn native_yield_now(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("yield_now")
    }

    pub fn start_yield_now(
        _vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if !args.is_empty() {
            return Err(RuntimeError::arity_mismatch(
                "yield_now() takes no arguments",
            ));
        }
        Ok(NativeAsyncRequest::yield_now())
    }

    #[ach_native(
        name = "sleep",
        arity = 1,
        effects = "task|io.clock",
        capabilities = "clock",
        behavior = "suspending",
        cancellation = "cooperative",
        async_adapter = "start_sleep"
    )]
    pub fn native_sleep(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("sleep")
    }

    pub fn start_sleep(_vm: &mut VM, args: &[Value]) -> Result<NativeAsyncRequest, RuntimeError> {
        timer_request(args, "sleep")
    }

    #[ach_native(
        name = "timeout_after",
        arity = 1,
        effects = "task|io.clock",
        capabilities = "clock",
        behavior = "suspending",
        cancellation = "cooperative",
        async_adapter = "start_timeout_after"
    )]
    pub fn native_timeout_after(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("timeout_after")
    }

    pub fn start_timeout_after(
        _vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        timer_request(args, "timeout_after")
    }

    fn timer_request(args: &[Value], name: &str) -> Result<NativeAsyncRequest, RuntimeError> {
        const MAX_SLEEP_MS: i64 = 86_400_000;
        if args.len() != 1 {
            return Err(RuntimeError::arity_mismatch(format!(
                "{name}() takes one millisecond duration"
            )));
        }
        let milliseconds = args[0]
            .as_int()
            .filter(|value| (0..=MAX_SLEEP_MS).contains(value))
            .ok_or_else(|| {
                RuntimeError::resource_limit_exceeded(format!(
                    "{name} duration must be 0..={MAX_SLEEP_MS} milliseconds"
                ))
            })?;
        Ok(NativeAsyncRequest::timer(Duration::from_millis(
            milliseconds as u64,
        )))
    }

    #[ach_native(
        name = "channel_send",
        arity = 2,
        effects = "task",
        behavior = "suspending",
        cancellation = "cooperative",
        async_adapter = "start_channel_send"
    )]
    pub fn native_channel_send(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("channel_send")
    }

    pub fn start_channel_send(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::arity_mismatch(
                "channel_send() takes a Channel and a sendable value",
            ));
        }
        let handle = vm.require_channel_resource(args[0])?;
        let value = sendable_value(args[1])?;
        Ok(NativeAsyncRequest::channel(NativeChannelRequest::Send {
            handle,
            value,
        }))
    }

    #[ach_native(
        name = "channel_receive",
        arity = 1,
        effects = "task",
        behavior = "suspending",
        cancellation = "cooperative",
        async_adapter = "start_channel_receive"
    )]
    pub fn native_channel_receive(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("channel_receive")
    }

    pub fn start_channel_receive(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::arity_mismatch(
                "channel_receive() takes exactly one Channel",
            ));
        }
        let handle = vm.require_channel_resource(args[0])?;
        Ok(NativeAsyncRequest::channel(NativeChannelRequest::Receive {
            handle,
        }))
    }

    #[ach_native(name = "channel_close", arity = 1, resource = "consumes:channel")]
    pub fn native_channel_close(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::arity_mismatch(
                "channel_close() takes exactly one Channel",
            ));
        }
        vm.close_channel_resource(args[0])?;
        Ok(Value::nil())
    }

    #[ach_native(name = "permit_pool", arity = 1, resource = "creates:channel")]
    pub fn native_permit_pool(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
        create_permit_controller(vm, args, "permit_pool")
    }

    /// Create the canonical active-handler controller for an accept loop.
    /// Acquire a permit before accepting a connection and release it only
    /// after that connection's structured handler has finished cleanup.
    #[ach_native(name = "bounded_server", arity = 1, resource = "creates:channel")]
    pub fn native_bounded_server(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
        create_permit_controller(vm, args, "bounded_server")
    }

    fn create_permit_controller(
        vm: &mut VM,
        args: &[Value],
        name: &str,
    ) -> Result<Value, RuntimeError> {
        let limit = capacity_arg(args, name, false)?;
        vm.create_permit_pool_resource(limit)
    }

    #[ach_native(
        name = "permit_acquire",
        arity = 1,
        effects = "task",
        behavior = "suspending",
        cancellation = "cooperative",
        async_adapter = "start_permit_acquire"
    )]
    pub fn native_permit_acquire(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("permit_acquire")
    }

    pub fn start_permit_acquire(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        task_impl::start_channel_receive(vm, args)
    }

    #[ach_native(
        name = "permit_release",
        arity = 1,
        effects = "task",
        behavior = "suspending",
        cancellation = "cooperative",
        async_adapter = "start_permit_release"
    )]
    pub fn native_permit_release(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("permit_release")
    }

    pub fn start_permit_release(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::arity_mismatch(
                "permit_release() takes exactly one permit pool",
            ));
        }
        let handle = vm.require_channel_resource(args[0])?;
        Ok(NativeAsyncRequest::channel(NativeChannelRequest::Send {
            handle,
            value: Value::nil(),
        }))
    }
}

fn capacity_arg(args: &[Value], name: &str, allow_zero: bool) -> Result<usize, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(format!(
            "{name}() takes exactly one capacity"
        )));
    }
    args[0]
        .as_int()
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value <= MAX_CHANNEL_CAPACITY && (allow_zero || *value != 0))
        .ok_or_else(|| {
            RuntimeError::resource_limit_exceeded(format!(
                "{name} capacity must be {}..={MAX_CHANNEL_CAPACITY}",
                usize::from(!allow_zero)
            ))
        })
}

fn sendable_value(value: Value) -> Result<Value, RuntimeError> {
    if value.is_nil()
        || value.is_bool()
        || value.is_int()
        || value.is_string()
        || value.is_bytes()
        || value.is_field()
        || value.is_bigint()
        || value.is_proof()
    {
        Ok(value)
    } else {
        Err(RuntimeError::type_mismatch(
            "channel messages must be immutable scalar, String, Bytes, Field, BigInt, or Proof values",
        ))
    }
}

fn suspending_only(name: &str) -> Result<Value, RuntimeError> {
    Err(RuntimeError::task_failed(format!(
        "`{name}` is suspending and must be called with await"
    )))
}
