//! I/O natives (feature-gated): read_line, read_file, write_file.

use std::io::Read;
use std::path::Path;

use ach_macros::{ach_module, ach_native};
use akron::error::RuntimeError;
use akron::machine::VM;
use memory::Value;

const MAX_COMPAT_FILE_SIZE: usize = 100 * 1024 * 1024;

#[ach_module(name = "io")]
pub mod io_impl {
    use super::*;

    /// `read_line()` → String from stdin (trimmed).
    #[ach_native(
        name = "read_line",
        arity = 0,
        effects = "io.console",
        capabilities = "console.read",
        behavior = "blocking"
    )]
    pub fn native_read_line(vm: &mut VM, _args: &[Value]) -> Result<Value, RuntimeError> {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| RuntimeError::io_error("read_line", e.to_string()))?;
        let trimmed = input.trim_end_matches('\n').trim_end_matches('\r');
        let handle = vm.heap.alloc_string(trimmed.to_string())?;
        Ok(Value::string(handle))
    }

    /// `read_file(path)` → String contents of a file.
    #[ach_native(
        name = "read_file",
        arity = 1,
        effects = "io.file",
        capabilities = "file.read",
        behavior = "blocking"
    )]
    pub fn native_read_file(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::arity_mismatch(
                "read_file() takes exactly 1 argument",
            ));
        }
        if !args[0].is_string() {
            return Err(RuntimeError::type_mismatch(
                "read_file() expects a String path",
            ));
        }
        let handle = args[0]
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("bad string handle"))?;
        let path = vm
            .heap
            .get_string(handle)
            .ok_or(RuntimeError::stale_heap("String", "read_file"))?
            .clone();
        let path = vm.host_policy.authorize_read(&path)?;

        let contents = read_bounded_text(&path)?;
        let h = vm.heap.alloc_string(contents)?;
        Ok(Value::string(h))
    }

    /// `write_file(path, contents)` → nil. Writes string to file.
    #[ach_native(
        name = "write_file",
        arity = 2,
        effects = "io.file",
        capabilities = "file.write",
        behavior = "blocking"
    )]
    pub fn native_write_file(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::arity_mismatch(
                "write_file() takes exactly 2 arguments",
            ));
        }
        if !args[0].is_string() {
            return Err(RuntimeError::type_mismatch(
                "write_file() first argument must be a String path",
            ));
        }
        if !args[1].is_string() {
            return Err(RuntimeError::type_mismatch(
                "write_file() second argument must be a String",
            ));
        }
        let path_handle = args[0]
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("bad string handle"))?;
        let path = vm
            .heap
            .get_string(path_handle)
            .ok_or(RuntimeError::stale_heap("String", "write_file"))?
            .clone();
        let path = vm.host_policy.authorize_write(&path)?;

        let content_handle = args[1]
            .as_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("bad string handle"))?;
        let contents = vm
            .heap
            .get_string(content_handle)
            .ok_or(RuntimeError::stale_heap("String", "write_file"))?
            .clone();
        validate_compat_file_size(contents.len(), "write_file")?;

        std::fs::write(&path, &contents).map_err(|e| {
            RuntimeError::io_error(format!("write_file('{}')", path.display()), e.to_string())
        })?;
        Ok(Value::nil())
    }
}

fn read_bounded_text(path: &Path) -> Result<String, RuntimeError> {
    let file = std::fs::File::open(path).map_err(|error| {
        RuntimeError::io_error(
            format!("read_file('{}')", path.display()),
            error.to_string(),
        )
    })?;
    let mut contents = String::new();
    file.take((MAX_COMPAT_FILE_SIZE + 1) as u64)
        .read_to_string(&mut contents)
        .map_err(|error| {
            RuntimeError::io_error(
                format!("read_file('{}')", path.display()),
                error.to_string(),
            )
        })?;
    validate_compat_file_size(contents.len(), "read_file")?;
    Ok(contents)
}

fn validate_compat_file_size(size: usize, name: &str) -> Result<(), RuntimeError> {
    if size > MAX_COMPAT_FILE_SIZE {
        return Err(RuntimeError::resource_limit_exceeded(format!(
            "{name} payload exceeds {MAX_COMPAT_FILE_SIZE} bytes"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_file_size_limit_accepts_boundary_and_rejects_larger_values() {
        assert!(validate_compat_file_size(MAX_COMPAT_FILE_SIZE, "write_file").is_ok());
        let error = validate_compat_file_size(MAX_COMPAT_FILE_SIZE + 1, "write_file")
            .expect_err("oversized compatibility write must be rejected");
        assert!(
            matches!(error, RuntimeError::ResourceLimitExceeded(_)),
            "{error}"
        );
    }
}
