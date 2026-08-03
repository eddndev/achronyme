use memory::{Value, ValueResourceKind};

use crate::error::RuntimeError;
use crate::machine::VM;

use super::SharedFileResource;

impl VM {
    /// Reserve a monotonic VM-owned handle for an async native result.
    pub fn reserve_resource(&mut self, kind: ValueResourceKind) -> Result<u32, RuntimeError> {
        if self.resources.len() >= self.runtime_limits.max_resources {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "open host resources exceed {}",
                self.runtime_limits.max_resources
            )));
        }
        self.resources
            .reserve(kind, self.task_scheduler.current_task())
    }

    /// Validate an owned resource before constructing a host request.
    pub fn require_resource(
        &self,
        value: Value,
        kind: ValueResourceKind,
    ) -> Result<u32, RuntimeError> {
        self.resources
            .require(value, kind, self.task_scheduler.current_task())
    }

    /// Get the synchronized file payload for one bounded worker operation.
    pub fn file_resource(&self, value: Value) -> Result<SharedFileResource, RuntimeError> {
        self.resources
            .file(value, self.task_scheduler.current_task())
    }

    /// Create a bounded, shareable channel capability.
    pub fn create_channel_resource(&mut self, capacity: usize) -> Result<Value, RuntimeError> {
        let handle = self.reserve_resource(ValueResourceKind::Channel)?;
        if let Err(error) =
            self.channel_hub
                .create(handle, capacity, self.runtime_limits.max_channels)
        {
            self.close_resource_handle(handle);
            return Err(error);
        }
        self.resources.activate_channel(handle)
    }

    /// Create a bounded permit pool pre-filled with `limit` unit tokens.
    pub fn create_permit_pool_resource(&mut self, limit: usize) -> Result<Value, RuntimeError> {
        if limit == 0 {
            return Err(RuntimeError::resource_limit_exceeded(
                "permit pool limit must be at least 1",
            ));
        }
        let value = self.create_channel_resource(limit)?;
        let handle = self.resources.require_channel(value)?;
        if let Err(error) = self.channel_hub.seed(handle, Value::nil(), limit) {
            self.close_resource_handle(handle);
            return Err(error);
        }
        Ok(value)
    }

    /// Validate a shareable channel capability without ambient authority.
    pub fn require_channel_resource(&self, value: Value) -> Result<u32, RuntimeError> {
        self.resources.require_channel(value)
    }

    /// Deterministically close a channel and wake its pending operations.
    pub fn close_channel_resource(&mut self, value: Value) -> Result<(), RuntimeError> {
        let handle = self.resources.require_channel(value)?;
        self.close_resource_handle(handle);
        Ok(())
    }

    pub(crate) fn close_resource_handle(&mut self, handle: u32) {
        if let Some(kind) = self.resources.close(handle) {
            if matches!(
                kind,
                ValueResourceKind::Listener | ValueResourceKind::Connection
            ) {
                self.network_reactor.close_silently(handle);
            } else if kind == ValueResourceKind::Channel {
                self.channel_hub.close(handle);
            }
        }
    }

    pub(crate) fn close_resources_owned_by(&mut self, owner: Option<u32>) {
        let closed = self.resources.close_owned_by(owner);
        for (handle, kind) in closed {
            if matches!(
                kind,
                ValueResourceKind::Listener | ValueResourceKind::Connection
            ) {
                self.network_reactor.close_silently(handle);
            } else if kind == ValueResourceKind::Channel {
                self.channel_hub.close(handle);
            }
        }
    }

    pub(crate) fn close_all_resources(&mut self) {
        let closed = self.resources.close_all();
        for (handle, kind) in closed {
            if matches!(
                kind,
                ValueResourceKind::Listener | ValueResourceKind::Connection
            ) {
                self.network_reactor.close_silently(handle);
            } else if kind == ValueResourceKind::Channel {
                self.channel_hub.close(handle);
            }
        }
    }
}
