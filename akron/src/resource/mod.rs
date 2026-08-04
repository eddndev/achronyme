//! Linear, VM-owned host resources.

use std::fs::File;
use std::sync::{Arc, Mutex};

use memory::{Value, ValueResourceKind};

use crate::error::RuntimeError;
use crate::specs::ResourceKind;

mod vm;

const MAX_RESOURCES: usize = u16::MAX as usize;

/// Shared file state used only by bounded blocking-pool jobs. The mutex does
/// not define language concurrency: the resource table permits one borrower.
pub type SharedFileResource = Arc<Mutex<File>>;

#[derive(Debug)]
enum ResourcePayload {
    Reserved,
    File(SharedFileResource),
    Network,
    Channel,
    Closed,
}

#[derive(Debug)]
struct ResourceEntry {
    kind: ValueResourceKind,
    owner: Option<u32>,
    borrower: Option<u32>,
    payload: ResourcePayload,
}

#[derive(Debug)]
struct ResourceSlot {
    generation: u16,
    entry: Option<ResourceEntry>,
}

#[derive(Debug, Default)]
pub(crate) struct ResourceTable {
    slots: Vec<ResourceSlot>,
    free: Vec<u16>,
    live: usize,
    #[cfg(test)]
    close_counts: std::collections::HashMap<u32, usize>,
}

impl ResourceTable {
    pub(crate) fn len(&self) -> usize {
        self.live
    }

    pub(crate) fn reserve(
        &mut self,
        kind: ValueResourceKind,
        owner: Option<u32>,
    ) -> Result<u32, RuntimeError> {
        if self.live >= MAX_RESOURCES {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "open host resources exceed {MAX_RESOURCES}"
            )));
        }
        let entry = ResourceEntry {
            kind,
            owner,
            borrower: None,
            payload: ResourcePayload::Reserved,
        };
        let (index, generation) = if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            debug_assert!(slot.entry.is_none());
            slot.entry = Some(entry);
            (index, slot.generation)
        } else {
            if self.slots.len() >= MAX_RESOURCES {
                return Err(RuntimeError::resource_limit_exceeded(
                    "resource handle generations are exhausted",
                ));
            }
            let index = self.slots.len() as u16;
            self.slots.push(ResourceSlot {
                generation: 0,
                entry: Some(entry),
            });
            (index, 0)
        };
        self.live += 1;
        Ok(encode_handle(index, generation))
    }

    pub(crate) fn activate_file(&mut self, handle: u32, file: File) -> Result<Value, RuntimeError> {
        let entry = self.entry_mut(handle)?;
        if entry.kind != ValueResourceKind::File
            || !matches!(entry.payload, ResourcePayload::Reserved)
        {
            return Err(RuntimeError::resource_error(
                "file activation requires a reserved file handle",
            ));
        }
        entry.payload = ResourcePayload::File(Arc::new(Mutex::new(file)));
        Ok(Value::resource(entry.kind, handle))
    }

    pub(crate) fn activate_network(
        &mut self,
        handle: u32,
        kind: ValueResourceKind,
    ) -> Result<Value, RuntimeError> {
        let entry = self.entry_mut(handle)?;
        if entry.kind != kind || !matches!(entry.payload, ResourcePayload::Reserved) {
            return Err(RuntimeError::resource_error(
                "network activation requires the matching reserved handle",
            ));
        }
        if !matches!(
            kind,
            ValueResourceKind::Listener | ValueResourceKind::Connection
        ) {
            return Err(RuntimeError::resource_error(
                "only listener and connection resources use the network reactor",
            ));
        }
        entry.payload = ResourcePayload::Network;
        Ok(Value::resource(kind, handle))
    }

    pub(crate) fn activate_channel(&mut self, handle: u32) -> Result<Value, RuntimeError> {
        let entry = self.entry_mut(handle)?;
        if entry.kind != ValueResourceKind::Channel
            || !matches!(entry.payload, ResourcePayload::Reserved)
        {
            return Err(RuntimeError::resource_error(
                "channel activation requires a reserved channel handle",
            ));
        }
        entry.payload = ResourcePayload::Channel;
        Ok(Value::resource(ValueResourceKind::Channel, handle))
    }

    pub(crate) fn require_channel(&self, value: Value) -> Result<u32, RuntimeError> {
        let (kind, handle) = value
            .as_resource_handle()
            .ok_or_else(|| RuntimeError::type_mismatch("expected an opaque Channel resource"))?;
        if kind != ValueResourceKind::Channel {
            return Err(RuntimeError::type_mismatch(format!(
                "expected Channel, got {kind:?}"
            )));
        }
        let entry = self.entry(handle)?;
        if entry.kind != ValueResourceKind::Channel
            || !matches!(entry.payload, ResourcePayload::Channel)
        {
            return Err(RuntimeError::resource_error("channel is closed or stale"));
        }
        Ok(handle)
    }

    pub(crate) fn require(
        &self,
        value: Value,
        kind: ValueResourceKind,
        owner: Option<u32>,
    ) -> Result<u32, RuntimeError> {
        let (actual_kind, handle) = value.as_resource_handle().ok_or_else(|| {
            RuntimeError::type_mismatch(format!("expected an opaque {kind:?} resource"))
        })?;
        if actual_kind != kind {
            return Err(RuntimeError::type_mismatch(format!(
                "expected {kind:?}, got {actual_kind:?}"
            )));
        }
        let entry = self.entry(handle)?;
        if entry.kind != kind || matches!(entry.payload, ResourcePayload::Closed) {
            return Err(RuntimeError::resource_error("resource is closed or stale"));
        }
        if matches!(entry.payload, ResourcePayload::Reserved) {
            return Err(RuntimeError::resource_error("resource is not ready"));
        }
        if entry.owner != owner {
            return Err(RuntimeError::resource_error(
                "resource is owned by another task",
            ));
        }
        if entry.borrower.is_some() {
            return Err(RuntimeError::resource_error(
                "resource already has an active I/O operation",
            ));
        }
        Ok(handle)
    }

    pub(crate) fn file(
        &self,
        value: Value,
        owner: Option<u32>,
    ) -> Result<SharedFileResource, RuntimeError> {
        let handle = self.require(value, ValueResourceKind::File, owner)?;
        match &self.entry(handle)?.payload {
            ResourcePayload::File(file) => Ok(Arc::clone(file)),
            _ => Err(RuntimeError::resource_error("file resource is unavailable")),
        }
    }

    pub(crate) fn transfer(
        &mut self,
        handle: u32,
        from: Option<u32>,
        to: Option<u32>,
    ) -> Result<(), RuntimeError> {
        let entry = self.entry_mut(handle)?;
        if matches!(entry.payload, ResourcePayload::Closed) {
            return Err(RuntimeError::resource_error(
                "cannot transfer a closed resource",
            ));
        }
        if entry.owner != from {
            return Err(RuntimeError::resource_error(
                "resource transfer attempted by a non-owner",
            ));
        }
        if entry.borrower.is_some() {
            return Err(RuntimeError::resource_error(
                "cannot transfer a borrowed resource",
            ));
        }
        entry.owner = to;
        Ok(())
    }

    pub(crate) fn kind(&self, handle: u32) -> Result<ValueResourceKind, RuntimeError> {
        Ok(self.entry(handle)?.kind)
    }

    pub(crate) fn borrow(
        &mut self,
        handle: u32,
        kind: ValueResourceKind,
        owner: Option<u32>,
        borrower: u32,
    ) -> Result<(), RuntimeError> {
        let entry = self.entry_mut(handle)?;
        if entry.kind != kind
            || matches!(
                entry.payload,
                ResourcePayload::Reserved | ResourcePayload::Closed
            )
        {
            return Err(RuntimeError::resource_error("resource is not open"));
        }
        if entry.owner != owner {
            return Err(RuntimeError::resource_error(
                "resource borrow attempted by a non-owner",
            ));
        }
        if entry.borrower.is_some() {
            return Err(RuntimeError::resource_error(
                "resource already has an active borrower",
            ));
        }
        entry.borrower = Some(borrower);
        Ok(())
    }

    pub(crate) fn release_borrow(&mut self, handle: u32, borrower: u32) {
        if let Ok(entry) = self.entry_mut(handle) {
            if entry.borrower == Some(borrower) {
                entry.borrower = None;
            }
        }
    }

    /// Close a resource and return its kind so the VM can notify external
    /// services such as the network reactor. Cleanup is intentionally
    /// idempotent; user-facing double-close checks happen before this call.
    pub(crate) fn close(&mut self, handle: u32) -> Option<ValueResourceKind> {
        let (index, generation) = decode_handle(handle);
        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation != generation {
            return None;
        }
        let mut entry = slot.entry.take()?;
        entry.payload = ResourcePayload::Closed;
        let kind = entry.kind;
        self.live = self.live.saturating_sub(1);
        if slot.generation != u16::MAX {
            slot.generation += 1;
            self.free.push(index);
        }
        #[cfg(test)]
        {
            *self.close_counts.entry(handle).or_default() += 1;
        }
        Some(kind)
    }

    #[cfg(test)]
    pub(crate) fn close_count(&self, handle: u32) -> usize {
        self.close_counts.get(&handle).copied().unwrap_or(0)
    }

    pub(crate) fn close_owned_by(&mut self, owner: Option<u32>) -> Vec<(u32, ValueResourceKind)> {
        let handles = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let entry = slot.entry.as_ref()?;
                (entry.owner == owner).then_some(encode_handle(index as u16, slot.generation))
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .filter_map(|handle| self.close(handle).map(|kind| (handle, kind)))
            .collect()
    }

    pub(crate) fn close_all(&mut self) -> Vec<(u32, ValueResourceKind)> {
        let handles = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.entry
                    .as_ref()
                    .map(|_| encode_handle(index as u16, slot.generation))
            })
            .collect::<Vec<_>>();
        let mut closed = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Some(kind) = self.close(handle) {
                closed.push((handle, kind));
            }
        }
        closed
    }

    fn entry(&self, handle: u32) -> Result<&ResourceEntry, RuntimeError> {
        let (index, generation) = decode_handle(handle);
        let slot = self
            .slots
            .get(index as usize)
            .filter(|slot| slot.generation == generation)
            .ok_or_else(|| RuntimeError::resource_error("stale resource handle"))?;
        slot.entry
            .as_ref()
            .ok_or_else(|| RuntimeError::resource_error("closed resource handle"))
    }

    fn entry_mut(&mut self, handle: u32) -> Result<&mut ResourceEntry, RuntimeError> {
        let (index, generation) = decode_handle(handle);
        let slot = self
            .slots
            .get_mut(index as usize)
            .filter(|slot| slot.generation == generation)
            .ok_or_else(|| RuntimeError::resource_error("stale resource handle"))?;
        slot.entry
            .as_mut()
            .ok_or_else(|| RuntimeError::resource_error("closed resource handle"))
    }
}

fn encode_handle(index: u16, generation: u16) -> u32 {
    ((generation as u32) << 16) | index as u32
}

fn decode_handle(handle: u32) -> (u16, u16) {
    (handle as u16, (handle >> 16) as u16)
}

pub(crate) fn value_resource_kind(kind: ResourceKind) -> Result<ValueResourceKind, RuntimeError> {
    match kind {
        ResourceKind::File => Ok(ValueResourceKind::File),
        ResourceKind::Listener => Ok(ValueResourceKind::Listener),
        ResourceKind::Connection => Ok(ValueResourceKind::Connection),
        ResourceKind::Channel => Ok(ValueResourceKind::Channel),
        ResourceKind::Task => Err(RuntimeError::resource_error(
            "task handles are not host resources",
        )),
    }
}

#[cfg(test)]
mod tests;
