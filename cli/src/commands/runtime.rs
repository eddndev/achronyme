use std::net::SocketAddr;
use std::path::PathBuf;

use akron::{HostPolicy, RuntimeLimits, VM};
use anyhow::{Context, Result};

/// Explicit host grants and resource bounds selected for one VM launch.
#[derive(Clone, Debug, Default)]
pub struct RuntimeSecurity {
    pub allow_read: Vec<PathBuf>,
    pub allow_write: Vec<PathBuf>,
    pub allow_connect: Vec<String>,
    pub allow_listen: Vec<String>,
    pub max_tasks: Option<usize>,
    pub max_resources: Option<usize>,
    pub max_task_scopes: Option<usize>,
    pub max_pending_native_requests: Option<usize>,
    pub max_retained_task_results: Option<usize>,
    pub max_channels: Option<usize>,
    pub max_channel_operations: Option<usize>,
    pub blocking_workers: Option<usize>,
    pub blocking_queue_capacity: Option<usize>,
}

impl RuntimeSecurity {
    pub fn host_policy(&self) -> Result<HostPolicy> {
        let mut policy = HostPolicy::standard_cli();
        for root in &self.allow_read {
            policy
                .allow_read_root(root)
                .with_context(|| format!("invalid read grant `{}`", root.display()))?;
        }
        for root in &self.allow_write {
            policy
                .allow_write_root(root)
                .with_context(|| format!("invalid write grant `{}`", root.display()))?;
        }
        for address in &self.allow_connect {
            policy.allow_connect_addr(parse_address(address, "outbound connection grant")?);
        }
        for address in &self.allow_listen {
            policy.allow_listen_addr(parse_address(address, "listener grant")?);
        }
        Ok(policy)
    }

    pub fn runtime_limits(&self) -> Result<RuntimeLimits> {
        let mut limits = RuntimeLimits::default();
        if let Some(value) = self.max_tasks {
            limits.max_tasks = value;
        }
        if let Some(value) = self.max_resources {
            limits.max_resources = value;
        }
        if let Some(value) = self.max_task_scopes {
            limits.max_task_scopes = value;
        }
        if let Some(value) = self.max_pending_native_requests {
            limits.max_pending_native_requests = value;
        }
        if let Some(value) = self.max_retained_task_results {
            limits.max_retained_task_results = value;
        }
        if let Some(value) = self.max_channels {
            limits.max_channels = value;
        }
        if let Some(value) = self.max_channel_operations {
            limits.max_channel_operations = value;
        }
        if let Some(value) = self.blocking_workers {
            limits.blocking_workers = value;
        }
        if let Some(value) = self.blocking_queue_capacity {
            limits.blocking_queue_capacity = value;
        }
        limits
            .validate()
            .map_err(|error| anyhow::anyhow!("invalid VM runtime limits: {error}"))
    }

    pub fn apply(&self, vm: &mut VM) -> Result<()> {
        vm.host_policy = self.host_policy()?;
        vm.set_runtime_limits(self.runtime_limits()?)?;
        Ok(())
    }
}

fn parse_address(value: &str, context: &str) -> Result<SocketAddr> {
    value.parse::<SocketAddr>().with_context(|| {
        format!("invalid {context} `{value}`: expected an explicit numeric IP:port")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use akron::specs::CapabilitySet;

    #[test]
    fn default_cli_launch_explicitly_preserves_compatibility_authority() {
        let policy = RuntimeSecurity::default().host_policy().unwrap();
        assert!(policy.granted().contains(CapabilitySet::CONSOLE_READ));
        assert!(policy.granted().contains(CapabilitySet::CONSOLE_WRITE));
        assert!(policy.granted().contains(CapabilitySet::CLOCK));
        assert!(policy.granted().contains(CapabilitySet::RANDOM));
        assert!(!policy.granted().contains(CapabilitySet::FILE_READ));
        assert!(!policy.granted().contains(CapabilitySet::NETWORK_CONNECT));
    }

    #[test]
    fn applies_exact_grants_and_runtime_limits() {
        let directory = tempfile::tempdir().unwrap();
        let mut vm = VM::new();
        RuntimeSecurity {
            allow_read: vec![directory.path().to_path_buf()],
            allow_connect: vec!["127.0.0.1:443".into()],
            max_tasks: Some(7),
            max_resources: Some(3),
            max_task_scopes: Some(2),
            max_pending_native_requests: Some(6),
            max_retained_task_results: Some(1),
            max_channels: Some(4),
            max_channel_operations: Some(5),
            blocking_workers: Some(2),
            blocking_queue_capacity: Some(7),
            ..RuntimeSecurity::default()
        }
        .apply(&mut vm)
        .unwrap();

        assert!(vm.host_policy.granted().contains(CapabilitySet::FILE_READ));
        assert!(vm
            .host_policy
            .granted()
            .contains(CapabilitySet::NETWORK_CONNECT));
        assert_eq!(vm.runtime_limits.max_tasks, 7);
        assert_eq!(vm.runtime_limits.max_resources, 3);
        assert_eq!(vm.runtime_limits.max_task_scopes, 2);
        assert_eq!(vm.runtime_limits.max_pending_native_requests, 6);
        assert_eq!(vm.runtime_limits.max_retained_task_results, 1);
        assert_eq!(vm.runtime_limits.max_channels, 4);
        assert_eq!(vm.runtime_limits.max_channel_operations, 5);
        assert_eq!(vm.runtime_limits.blocking_workers, 2);
        assert_eq!(vm.runtime_limits.blocking_queue_capacity, 7);
    }

    #[test]
    fn rejects_dns_and_out_of_range_limits_before_execution() {
        let mut vm = VM::new();
        let error = RuntimeSecurity {
            allow_connect: vec!["example.com:443".into()],
            ..RuntimeSecurity::default()
        }
        .apply(&mut vm)
        .unwrap_err();
        assert!(error.to_string().contains("numeric IP:port"), "{error}");

        let error = RuntimeSecurity {
            max_tasks: Some(0),
            ..RuntimeSecurity::default()
        }
        .apply(&mut vm)
        .unwrap_err();
        assert!(error.to_string().contains("max_tasks"), "{error}");
    }
}
