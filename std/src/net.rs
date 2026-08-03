//! Capability-scoped, reactor-backed TCP operations.

use ach_macros::{ach_module, ach_native};
use akron::error::RuntimeError;
use akron::machine::VM;
use akron::{NativeAsyncRequest, NativeNetworkRequest};
use memory::{Value, ValueResourceKind};

const MAX_NETWORK_CHUNK: usize = 16 * 1024 * 1024;

#[ach_module(name = "net")]
pub mod net_impl {
    use super::*;

    #[ach_native(
        name = "tcp_connect",
        arity = 1,
        effects = "task|io.network",
        capabilities = "network.connect",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "creates:connection",
        async_adapter = "start_tcp_connect"
    )]
    pub fn native_tcp_connect(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_connect")
    }

    pub fn start_tcp_connect(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let address = address_arg(vm, args, "tcp_connect")?;
        let address = vm.host_policy.authorize_connect(&address)?;
        let handle = vm.reserve_resource(ValueResourceKind::Connection)?;
        Ok(
            NativeAsyncRequest::network(NativeNetworkRequest::Connect { handle, address })
                .with_created_resource(handle),
        )
    }

    #[ach_native(
        name = "tcp_listen",
        arity = 1,
        effects = "task|io.network",
        capabilities = "network.listen",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "creates:listener",
        async_adapter = "start_tcp_listen"
    )]
    pub fn native_tcp_listen(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_listen")
    }

    pub fn start_tcp_listen(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let address = address_arg(vm, args, "tcp_listen")?;
        let address = vm.host_policy.authorize_listen(&address)?;
        let handle = vm.reserve_resource(ValueResourceKind::Listener)?;
        Ok(
            NativeAsyncRequest::network(NativeNetworkRequest::Listen { handle, address })
                .with_created_resource(handle),
        )
    }

    #[ach_native(
        name = "tcp_accept",
        arity = 1,
        effects = "task|io.network",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "creates:connection+borrows:listener",
        async_adapter = "start_tcp_accept"
    )]
    pub fn native_tcp_accept(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_accept")
    }

    pub fn start_tcp_accept(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let listener = one_resource_arg(vm, args, ValueResourceKind::Listener, "tcp_accept")?;
        let connection = vm.reserve_resource(ValueResourceKind::Connection)?;
        Ok(NativeAsyncRequest::network(NativeNetworkRequest::Accept {
            listener,
            connection,
        })
        .with_created_resource(connection))
    }

    #[ach_native(
        name = "tcp_read",
        arity = 2,
        effects = "task|io.network",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "borrows:connection",
        async_adapter = "start_tcp_read"
    )]
    pub fn native_tcp_read(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_read")
    }

    pub fn start_tcp_read(vm: &mut VM, args: &[Value]) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::arity_mismatch(
                "tcp_read() takes a Connection and maximum byte count",
            ));
        }
        let handle = vm.require_resource(args[0], ValueResourceKind::Connection)?;
        let max_bytes = bounded_size(args[1], "tcp_read")?;
        Ok(NativeAsyncRequest::network(NativeNetworkRequest::Read {
            handle,
            max_bytes,
        }))
    }

    #[ach_native(
        name = "tcp_write",
        arity = 2,
        effects = "task|io.network",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "borrows:connection",
        async_adapter = "start_tcp_write"
    )]
    pub fn native_tcp_write(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_write")
    }

    pub fn start_tcp_write(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::arity_mismatch(
                "tcp_write() takes a Connection and String or Bytes value",
            ));
        }
        let handle = vm.require_resource(args[0], ValueResourceKind::Connection)?;
        let bytes = bytes_arg(vm, args[1], "tcp_write")?;
        if bytes.len() > MAX_NETWORK_CHUNK {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "tcp_write chunk exceeds {MAX_NETWORK_CHUNK} bytes"
            )));
        }
        Ok(NativeAsyncRequest::network(NativeNetworkRequest::Write {
            handle,
            bytes,
        }))
    }

    #[ach_native(
        name = "tcp_close",
        arity = 1,
        effects = "task|io.network",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "consumes:connection",
        async_adapter = "start_tcp_close"
    )]
    pub fn native_tcp_close(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_close")
    }

    pub fn start_tcp_close(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let handle = one_resource_arg(vm, args, ValueResourceKind::Connection, "tcp_close")?;
        Ok(NativeAsyncRequest::network(NativeNetworkRequest::Close {
            handle,
        }))
    }

    #[ach_native(
        name = "tcp_listener_close",
        arity = 1,
        effects = "task|io.network",
        behavior = "suspending",
        cancellation = "cooperative",
        resource = "consumes:listener",
        async_adapter = "start_tcp_listener_close"
    )]
    pub fn native_tcp_listener_close(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("tcp_listener_close")
    }

    pub fn start_tcp_listener_close(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let handle = one_resource_arg(vm, args, ValueResourceKind::Listener, "tcp_listener_close")?;
        Ok(NativeAsyncRequest::network(NativeNetworkRequest::Close {
            handle,
        }))
    }
}

fn suspending_only(name: &str) -> Result<Value, RuntimeError> {
    Err(RuntimeError::task_failed(format!(
        "`{name}` is suspending and must be called with await"
    )))
}

fn address_arg(vm: &VM, args: &[Value], name: &str) -> Result<String, RuntimeError> {
    if args.len() != 1 || !args[0].is_string() {
        return Err(RuntimeError::type_mismatch(format!(
            "{name} expects one numeric IP:port String"
        )));
    }
    let handle = args[0]
        .as_handle()
        .ok_or_else(|| RuntimeError::type_mismatch("bad address string handle"))?;
    vm.heap
        .get_string(handle)
        .cloned()
        .ok_or(RuntimeError::stale_heap("String", "network address"))
}

fn one_resource_arg(
    vm: &VM,
    args: &[Value],
    kind: ValueResourceKind,
    name: &str,
) -> Result<u32, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(format!(
            "{name} takes exactly one {kind:?} resource"
        )));
    }
    vm.require_resource(args[0], kind)
}

fn bounded_size(value: Value, name: &str) -> Result<usize, RuntimeError> {
    value
        .as_int()
        .and_then(|size| usize::try_from(size).ok())
        .filter(|size| (1..=MAX_NETWORK_CHUNK).contains(size))
        .ok_or_else(|| {
            RuntimeError::resource_limit_exceeded(format!(
                "{name} size must be between 1 and {MAX_NETWORK_CHUNK} bytes"
            ))
        })
}

fn bytes_arg(vm: &VM, value: Value, name: &str) -> Result<Vec<u8>, RuntimeError> {
    let handle = value
        .as_handle()
        .ok_or_else(|| RuntimeError::type_mismatch(format!("{name} expects String or Bytes")))?;
    if value.is_string() {
        vm.heap
            .get_string(handle)
            .map(|value| value.as_bytes().to_vec())
            .ok_or(RuntimeError::stale_heap("String", "network write"))
    } else if value.is_bytes() {
        vm.heap
            .get_bytes(handle)
            .cloned()
            .ok_or(RuntimeError::stale_heap("Bytes", "network write"))
    } else {
        Err(RuntimeError::type_mismatch(format!(
            "{name} expects String or Bytes"
        )))
    }
}
