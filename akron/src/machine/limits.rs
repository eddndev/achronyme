use crate::error::RuntimeError;

use super::blocking_pool::{
    BlockingPool, DEFAULT_BLOCKING_QUEUE_CAPACITY, DEFAULT_BLOCKING_WORKERS,
    MAX_BLOCKING_QUEUE_CAPACITY, MAX_BLOCKING_WORKERS,
};
use super::VM;

/// Per-VM bounds for structured concurrency and owned host state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeLimits {
    pub max_tasks: usize,
    pub max_resources: usize,
    pub max_task_scopes: usize,
    pub max_pending_native_requests: usize,
    pub max_retained_task_results: usize,
    pub max_channels: usize,
    pub max_channel_operations: usize,
    pub blocking_workers: usize,
    pub blocking_queue_capacity: usize,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_tasks: u16::MAX as usize,
            max_resources: u16::MAX as usize,
            max_task_scopes: 1_024,
            max_pending_native_requests: 4_096,
            max_retained_task_results: 4_096,
            max_channels: 4_096,
            max_channel_operations: u16::MAX as usize,
            blocking_workers: DEFAULT_BLOCKING_WORKERS,
            blocking_queue_capacity: DEFAULT_BLOCKING_QUEUE_CAPACITY,
        }
    }
}

impl RuntimeLimits {
    pub fn validate(self) -> Result<Self, RuntimeError> {
        let hard = Self::default();
        if self.max_tasks == 0 || self.max_tasks > hard.max_tasks {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_tasks must be 1..={}",
                hard.max_tasks
            )));
        }
        if self.max_resources > hard.max_resources {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_resources must be 0..={}",
                hard.max_resources
            )));
        }
        if self.max_task_scopes == 0 || self.max_task_scopes > hard.max_task_scopes {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_task_scopes must be 1..={}",
                hard.max_task_scopes
            )));
        }
        if self.max_pending_native_requests > hard.max_pending_native_requests {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_pending_native_requests must be 0..={}",
                hard.max_pending_native_requests
            )));
        }
        if self.max_retained_task_results > hard.max_retained_task_results {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_retained_task_results must be 0..={}",
                hard.max_retained_task_results
            )));
        }
        if self.max_channels > hard.max_channels {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_channels must be 0..={}",
                hard.max_channels
            )));
        }
        if self.max_channel_operations > hard.max_channel_operations {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "max_channel_operations must be 0..={}",
                hard.max_channel_operations
            )));
        }
        if self.blocking_workers == 0 || self.blocking_workers > MAX_BLOCKING_WORKERS {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "blocking_workers must be 1..={MAX_BLOCKING_WORKERS}"
            )));
        }
        if self.blocking_queue_capacity == 0
            || self.blocking_queue_capacity > MAX_BLOCKING_QUEUE_CAPACITY
        {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "blocking_queue_capacity must be 1..={MAX_BLOCKING_QUEUE_CAPACITY}"
            )));
        }
        Ok(self)
    }
}

impl VM {
    pub fn set_runtime_limits(&mut self, limits: RuntimeLimits) -> Result<(), RuntimeError> {
        let limits = limits.validate()?;
        if limits.blocking_workers != self.runtime_limits.blocking_workers
            || limits.blocking_queue_capacity != self.runtime_limits.blocking_queue_capacity
        {
            if self.live_task_count() != 0 || self.active_task_scope_count() != 0 {
                return Err(RuntimeError::resource_error(
                    "blocking-pool limits cannot change while tasks are active",
                ));
            }
            self.blocking_pool = BlockingPool::with_capacity(
                limits.blocking_workers,
                limits.blocking_queue_capacity,
            )?;
        }
        self.runtime_limits = limits;
        Ok(())
    }
}
