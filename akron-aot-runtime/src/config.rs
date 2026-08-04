use akron::VM;

pub(super) fn apply_environment(vm: &mut VM) -> Result<(), String> {
    if let Ok(value) = std::env::var("AKRON_INSTRUCTION_BUDGET") {
        vm.instruction_budget = value
            .parse()
            .map_err(|_| "AKRON_INSTRUCTION_BUDGET must be an unsigned integer".to_string())?;
    }
    if let Ok(value) = std::env::var("AKRON_MAX_HEAP") {
        let limit = parse_size(&value)
            .ok_or_else(|| "AKRON_MAX_HEAP must be bytes or use K, M, or G".to_string())?;
        vm.heap.set_max_heap_bytes(limit);
    }
    if let Ok(value) = std::env::var("AKRON_STRESS_GC") {
        vm.stress_mode = matches!(value.as_str(), "1" | "true" | "yes");
    }
    if let Some(value) = std::env::var_os("AKRON_ALLOW_READ") {
        for root in std::env::split_paths(&value) {
            vm.host_policy
                .allow_read_root(&root)
                .map_err(|error| format!("invalid AKRON_ALLOW_READ grant: {error}"))?;
        }
    }
    if let Some(value) = std::env::var_os("AKRON_ALLOW_WRITE") {
        for root in std::env::split_paths(&value) {
            vm.host_policy
                .allow_write_root(&root)
                .map_err(|error| format!("invalid AKRON_ALLOW_WRITE grant: {error}"))?;
        }
    }
    apply_address_grants(vm, "AKRON_ALLOW_CONNECT", true)?;
    apply_address_grants(vm, "AKRON_ALLOW_LISTEN", false)?;

    let mut limits = vm.runtime_limits;
    apply_usize_environment("AKRON_MAX_TASKS", &mut limits.max_tasks)?;
    apply_usize_environment("AKRON_MAX_RESOURCES", &mut limits.max_resources)?;
    apply_usize_environment("AKRON_MAX_TASK_SCOPES", &mut limits.max_task_scopes)?;
    apply_usize_environment(
        "AKRON_MAX_PENDING_NATIVE_REQUESTS",
        &mut limits.max_pending_native_requests,
    )?;
    apply_usize_environment(
        "AKRON_MAX_RETAINED_TASK_RESULTS",
        &mut limits.max_retained_task_results,
    )?;
    apply_usize_environment("AKRON_MAX_CHANNELS", &mut limits.max_channels)?;
    apply_usize_environment(
        "AKRON_MAX_CHANNEL_OPERATIONS",
        &mut limits.max_channel_operations,
    )?;
    apply_usize_environment("AKRON_BLOCKING_WORKERS", &mut limits.blocking_workers)?;
    apply_usize_environment(
        "AKRON_BLOCKING_QUEUE_CAPACITY",
        &mut limits.blocking_queue_capacity,
    )?;
    vm.set_runtime_limits(limits)
        .map_err(|error| format!("invalid AOT runtime limits: {error}"))?;
    Ok(())
}

fn apply_address_grants(vm: &mut VM, name: &str, connect: bool) -> Result<(), String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(());
    };
    for raw in value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let address = raw
            .parse()
            .map_err(|_| format!("{name} contains non-numeric address `{raw}`"))?;
        if connect {
            vm.host_policy.allow_connect_addr(address);
        } else {
            vm.host_policy.allow_listen_addr(address);
        }
    }
    Ok(())
}

fn apply_usize_environment(name: &str, target: &mut usize) -> Result<(), String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(());
    };
    *target = value
        .parse()
        .map_err(|_| format!("{name} must be an unsigned integer"))?;
    Ok(())
}

fn parse_size(value: &str) -> Option<usize> {
    let value = value.trim();
    let (number, multiplier) = match value.as_bytes().last()? {
        b'K' | b'k' => (&value[..value.len() - 1], 1024usize),
        b'M' | b'm' => (&value[..value.len() - 1], 1024 * 1024),
        b'G' | b'g' => (&value[..value.len() - 1], 1024 * 1024 * 1024),
        _ => (value, 1),
    };
    number.parse::<usize>().ok()?.checked_mul(multiplier)
}
