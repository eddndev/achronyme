//! Owned, suspending file-resource operations.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use ach_macros::{ach_module, ach_native};
use akron::error::RuntimeError;
use akron::machine::VM;
use akron::{NativeAsyncOutput, NativeAsyncRequest};
use memory::{Value, ValueResourceKind};

const MAX_FILE_CHUNK: usize = 16 * 1024 * 1024;

#[ach_module(name = "file")]
pub mod file_impl {
    use super::*;

    #[ach_native(
        name = "open_file",
        arity = 1,
        effects = "task|io.file",
        capabilities = "file.read",
        behavior = "suspending",
        cancellation = "before-start",
        resource = "creates:file",
        async_adapter = "start_open_file"
    )]
    pub fn native_open_file(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("open_file")
    }

    pub fn start_open_file(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let path = string_arg(vm, args, 0, "open_file")?;
        let path = vm.host_policy.authorize_read(path)?;
        let handle = vm.reserve_resource(ValueResourceKind::File)?;
        Ok(NativeAsyncRequest::blocking(Box::new(move || {
            File::open(&path)
                .map(|file| NativeAsyncOutput::FileResource { handle, file })
                .map_err(|error| format!("open_file('{}') failed: {error}", path.display()))
        }))
        .with_created_resource(handle))
    }

    #[ach_native(
        name = "create_file",
        arity = 1,
        effects = "task|io.file",
        capabilities = "file.write",
        behavior = "suspending",
        cancellation = "before-start",
        resource = "creates:file",
        async_adapter = "start_create_file"
    )]
    pub fn native_create_file(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("create_file")
    }

    pub fn start_create_file(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        let path = string_arg(vm, args, 0, "create_file")?;
        let path = vm.host_policy.authorize_write(path)?;
        let handle = vm.reserve_resource(ValueResourceKind::File)?;
        Ok(NativeAsyncRequest::blocking(Box::new(move || {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map(|file| NativeAsyncOutput::FileResource { handle, file })
                .map_err(|error| format!("create_file('{}') failed: {error}", path.display()))
        }))
        .with_created_resource(handle))
    }

    #[ach_native(
        name = "file_read",
        arity = 2,
        effects = "task|io.file",
        behavior = "suspending",
        cancellation = "before-start",
        resource = "borrows:file",
        async_adapter = "start_file_read"
    )]
    pub fn native_file_read(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("file_read")
    }

    pub fn start_file_read(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::arity_mismatch(
                "file_read() takes a File and maximum byte count",
            ));
        }
        let file = vm.file_resource(args[0])?;
        let max_bytes = bounded_size(args[1], "file_read")?;
        Ok(NativeAsyncRequest::blocking(Box::new(move || {
            let mut file = file
                .lock()
                .map_err(|_| "file_read resource lock was poisoned".to_string())?;
            let mut bytes = vec![0; max_bytes];
            let read = file
                .read(&mut bytes)
                .map_err(|error| format!("file_read failed: {error}"))?;
            bytes.truncate(read);
            Ok(NativeAsyncOutput::Bytes(bytes))
        })))
    }

    #[ach_native(
        name = "file_write",
        arity = 2,
        effects = "task|io.file",
        behavior = "suspending",
        cancellation = "before-start",
        resource = "borrows:file",
        async_adapter = "start_file_write"
    )]
    pub fn native_file_write(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("file_write")
    }

    pub fn start_file_write(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::arity_mismatch(
                "file_write() takes a File and String or Bytes value",
            ));
        }
        let file = vm.file_resource(args[0])?;
        let bytes = bytes_arg(vm, args[1], "file_write")?;
        if bytes.len() > MAX_FILE_CHUNK {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "file_write chunk exceeds {MAX_FILE_CHUNK} bytes"
            )));
        }
        Ok(NativeAsyncRequest::blocking(Box::new(move || {
            let mut file = file
                .lock()
                .map_err(|_| "file_write resource lock was poisoned".to_string())?;
            file.write_all(&bytes)
                .map_err(|error| format!("file_write failed: {error}"))?;
            Ok(NativeAsyncOutput::Int(bytes.len() as i64))
        })))
    }

    #[ach_native(
        name = "file_close",
        arity = 1,
        effects = "task|io.file",
        behavior = "suspending",
        cancellation = "before-start",
        resource = "consumes:file",
        async_adapter = "start_file_close"
    )]
    pub fn native_file_close(_vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        suspending_only("file_close")
    }

    pub fn start_file_close(
        vm: &mut VM,
        args: &[Value],
    ) -> Result<NativeAsyncRequest, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::arity_mismatch("file_close() takes one File"));
        }
        vm.require_resource(args[0], ValueResourceKind::File)?;
        Ok(NativeAsyncRequest::blocking(Box::new(|| {
            Ok(NativeAsyncOutput::Nil)
        })))
    }
}

fn suspending_only(name: &str) -> Result<Value, RuntimeError> {
    Err(RuntimeError::task_failed(format!(
        "`{name}` is suspending and must be called with await"
    )))
}

fn string_arg(vm: &VM, args: &[Value], index: usize, name: &str) -> Result<String, RuntimeError> {
    let value = args
        .get(index)
        .filter(|value| value.is_string())
        .ok_or_else(|| RuntimeError::type_mismatch(format!("{name} expects a String path")))?;
    let handle = value
        .as_handle()
        .ok_or_else(|| RuntimeError::type_mismatch("bad string handle"))?;
    vm.heap
        .get_string(handle)
        .cloned()
        .ok_or(RuntimeError::stale_heap("String", "file path"))
}

fn bounded_size(value: Value, name: &str) -> Result<usize, RuntimeError> {
    let size = value
        .as_int()
        .and_then(|size| usize::try_from(size).ok())
        .filter(|size| (1..=MAX_FILE_CHUNK).contains(size))
        .ok_or_else(|| {
            RuntimeError::resource_limit_exceeded(format!(
                "{name} size must be between 1 and {MAX_FILE_CHUNK} bytes"
            ))
        })?;
    Ok(size)
}

fn bytes_arg(vm: &VM, value: Value, name: &str) -> Result<Vec<u8>, RuntimeError> {
    let handle = value
        .as_handle()
        .ok_or_else(|| RuntimeError::type_mismatch(format!("{name} expects String or Bytes")))?;
    if value.is_string() {
        vm.heap
            .get_string(handle)
            .map(|value| value.as_bytes().to_vec())
            .ok_or(RuntimeError::stale_heap("String", "file write"))
    } else if value.is_bytes() {
        vm.heap
            .get_bytes(handle)
            .cloned()
            .ok_or(RuntimeError::stale_heap("Bytes", "file write"))
    } else {
        Err(RuntimeError::type_mismatch(format!(
            "{name} expects String or Bytes"
        )))
    }
}
