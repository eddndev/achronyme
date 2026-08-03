use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::commands::runtime::RuntimeSecurity;
use crate::commands::{new_compiler, print_warnings, render_compile_error, ErrorFormat};

pub fn inspect_manifest(
    path: &str,
    prime_id: memory::field::PrimeId,
    circom_lib_dirs: &[PathBuf],
    runtime_security: &RuntimeSecurity,
    error_format: ErrorFormat,
) -> Result<()> {
    let program = if path.ends_with(".achb") {
        let bytes = fs::read(path).with_context(|| format!("cannot read binary file: {path}"))?;
        akron::CompiledProgram::read_executable(&mut Cursor::new(bytes))
            .map_err(|error| anyhow::anyhow!("Loader error: {error}"))?
    } else {
        let source =
            fs::read_to_string(path).with_context(|| format!("cannot read source file: {path}"))?;
        let mut compiler = new_compiler();
        let options = akronc::CompileOptions::for_source(path)
            .with_prime(prime_id)
            .with_circom_lib_dirs(circom_lib_dirs.to_vec());
        let program = compiler
            .compile_program(&source, &options)
            .map_err(|error| {
                anyhow::anyhow!(render_compile_error(&error, &source, error_format))
            })?;
        print_warnings(&mut compiler, &source, error_format);
        program
    };
    let policy = runtime_security.host_policy()?;
    let limits = runtime_security.runtime_limits()?;
    let manifest = manifest_value(&program, policy.granted(), limits, runtime_security);
    if error_format == ErrorFormat::Json {
        println!("{}", serde_json::to_string(&manifest)?);
    } else {
        println!("format: 0x{:02x}", program.format_version);
        println!("bytecode: {}", program.bytecode_version);
        println!("effects: {}", program.requested_effects());
        println!("program-capabilities: {}", program.capabilities);
        println!(
            "requested-host-capabilities: {}",
            program.requested_host_capabilities()
        );
        println!("granted-host-capabilities: {}", policy.granted());
        println!(
            "allow-read: {}",
            display_paths(&runtime_security.allow_read)
        );
        println!(
            "allow-write: {}",
            display_paths(&runtime_security.allow_write)
        );
        println!(
            "allow-connect: {}",
            display_values(&runtime_security.allow_connect)
        );
        println!(
            "allow-listen: {}",
            display_values(&runtime_security.allow_listen)
        );
        println!(
            "limits: tasks={},resources={},task-scopes={},pending-native-requests={},retained-task-results={},channels={},channel-operations={},blocking-workers={},blocking-queue-capacity={}",
            limits.max_tasks,
            limits.max_resources,
            limits.max_task_scopes,
            limits.max_pending_native_requests,
            limits.max_retained_task_results,
            limits.max_channels,
            limits.max_channel_operations,
            limits.blocking_workers,
            limits.blocking_queue_capacity
        );
    }
    Ok(())
}

fn manifest_value(
    program: &akron::CompiledProgram,
    granted: akron::specs::CapabilitySet,
    limits: akron::RuntimeLimits,
    security: &RuntimeSecurity,
) -> serde_json::Value {
    serde_json::json!({
        "format_version": program.format_version,
        "bytecode_version": program.bytecode_version,
        "effects": program.requested_effects().to_string(),
        "program_capabilities": program.capabilities.to_string(),
        "requested_host_capabilities": program.requested_host_capabilities().to_string(),
        "granted_host_capabilities": granted.to_string(),
        "grants": {
            "read_roots": security.allow_read.iter().map(|path| path.display().to_string()).collect::<Vec<_>>(),
            "write_roots": security.allow_write.iter().map(|path| path.display().to_string()).collect::<Vec<_>>(),
            "connect": &security.allow_connect,
            "listen": &security.allow_listen,
        },
        "limits": {
            "tasks": limits.max_tasks,
            "resources": limits.max_resources,
            "task_scopes": limits.max_task_scopes,
            "pending_native_requests": limits.max_pending_native_requests,
            "retained_task_results": limits.max_retained_task_results,
            "channels": limits.max_channels,
            "channel_operations": limits.max_channel_operations,
            "blocking_workers": limits.blocking_workers,
            "blocking_queue_capacity": limits.blocking_queue_capacity,
        }
    })
}

fn display_paths(paths: &[PathBuf]) -> String {
    display_values(
        &paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>(),
    )
}

fn display_values(values: &[String]) -> String {
    if values.is_empty() {
        "none".into()
    } else {
        values.join(",")
    }
}
