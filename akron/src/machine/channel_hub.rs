//! Bounded in-VM channels used for backpressure and permit pools.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use memory::Value;

use crate::machine::blocking_pool::JobReceiver;
use crate::native::{NativeAsyncOutput, NativeChannelRequest};
use crate::RuntimeError;

const MAX_CHANNELS: usize = 4_096;
const MAX_CHANNEL_CAPACITY: usize = 65_535;
const MAX_PENDING_OPERATIONS: usize = 65_535;

struct PendingSend {
    value: Value,
    cancelled: Arc<AtomicBool>,
    result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
}

struct PendingReceive {
    cancelled: Arc<AtomicBool>,
    result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
}

struct Channel {
    capacity: usize,
    queue: VecDeque<Value>,
    senders: VecDeque<PendingSend>,
    receivers: VecDeque<PendingReceive>,
}

#[derive(Default)]
pub(crate) struct ChannelHub {
    channels: HashMap<u32, Channel>,
}

impl ChannelHub {
    pub(crate) fn create(
        &mut self,
        handle: u32,
        capacity: usize,
        max_channels: usize,
    ) -> Result<(), RuntimeError> {
        if capacity > MAX_CHANNEL_CAPACITY {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "channel capacity exceeds {MAX_CHANNEL_CAPACITY}"
            )));
        }
        let max_channels = max_channels.min(MAX_CHANNELS);
        if self.channels.len() >= max_channels {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "open channels exceed {max_channels}"
            )));
        }
        if self
            .channels
            .insert(
                handle,
                Channel {
                    capacity,
                    queue: VecDeque::with_capacity(capacity.min(1_024)),
                    senders: VecDeque::new(),
                    receivers: VecDeque::new(),
                },
            )
            .is_some()
        {
            return Err(RuntimeError::resource_error(
                "channel handle is already active",
            ));
        }
        Ok(())
    }

    pub(crate) fn submit(
        &mut self,
        request: NativeChannelRequest,
        cancelled: Arc<AtomicBool>,
        max_pending_operations: usize,
    ) -> Result<JobReceiver, RuntimeError> {
        self.cancel_requested();
        let max_pending_operations = max_pending_operations.min(MAX_PENDING_OPERATIONS);
        if self.pending_count() >= max_pending_operations {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "pending channel operations exceed {max_pending_operations}"
            )));
        }
        let (result, receiver) = mpsc::channel();
        if cancelled.load(Ordering::Acquire) {
            let _ = result.send(Err("channel operation cancelled before start".into()));
            return Ok(receiver);
        }
        match request {
            NativeChannelRequest::Send { handle, value } => {
                let channel = self.channel_mut(handle)?;
                channel.send(value, cancelled, result);
            }
            NativeChannelRequest::Receive { handle } => {
                let channel = self.channel_mut(handle)?;
                channel.receive(cancelled, result);
            }
        }
        Ok(receiver)
    }

    pub(crate) fn close(&mut self, handle: u32) {
        if let Some(mut channel) = self.channels.remove(&handle) {
            channel.fail_all("channel closed");
        }
    }

    pub(crate) fn seed(
        &mut self,
        handle: u32,
        value: Value,
        count: usize,
    ) -> Result<(), RuntimeError> {
        let channel = self.channel_mut(handle)?;
        if channel.queue.len().saturating_add(count) > channel.capacity {
            return Err(RuntimeError::resource_limit_exceeded(
                "permit seed exceeds channel capacity",
            ));
        }
        channel.queue.extend(std::iter::repeat_n(value, count));
        Ok(())
    }

    pub(crate) fn cancel_requested(&mut self) {
        for channel in self.channels.values_mut() {
            channel.cancel_requested();
        }
    }

    pub(crate) fn roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        for channel in self.channels.values() {
            roots.extend(channel.queue.iter().copied());
            roots.extend(channel.senders.iter().map(|sender| sender.value));
        }
        roots
    }

    fn channel_mut(&mut self, handle: u32) -> Result<&mut Channel, RuntimeError> {
        self.channels
            .get_mut(&handle)
            .ok_or_else(|| RuntimeError::resource_error("channel is closed or stale"))
    }

    fn pending_count(&self) -> usize {
        self.channels
            .values()
            .map(|channel| channel.senders.len() + channel.receivers.len())
            .sum()
    }
}

impl Channel {
    fn send(
        &mut self,
        value: Value,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) {
        self.cancel_requested();
        if let Some(receiver) = self.receivers.pop_front() {
            let _ = receiver.result.send(Ok(NativeAsyncOutput::Value(value)));
            let _ = result.send(Ok(NativeAsyncOutput::Nil));
        } else if self.queue.len() < self.capacity {
            self.queue.push_back(value);
            let _ = result.send(Ok(NativeAsyncOutput::Nil));
        } else {
            self.senders.push_back(PendingSend {
                value,
                cancelled,
                result,
            });
        }
    }

    fn receive(
        &mut self,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) {
        self.cancel_requested();
        if let Some(value) = self.queue.pop_front() {
            let _ = result.send(Ok(NativeAsyncOutput::Value(value)));
            self.refill_from_sender();
        } else if let Some(sender) = self.senders.pop_front() {
            let _ = result.send(Ok(NativeAsyncOutput::Value(sender.value)));
            let _ = sender.result.send(Ok(NativeAsyncOutput::Nil));
        } else {
            self.receivers
                .push_back(PendingReceive { cancelled, result });
        }
    }

    fn refill_from_sender(&mut self) {
        if let Some(sender) = self.senders.pop_front() {
            if self.capacity == 0 {
                self.senders.push_front(sender);
            } else {
                self.queue.push_back(sender.value);
                let _ = sender.result.send(Ok(NativeAsyncOutput::Nil));
            }
        }
    }

    fn cancel_requested(&mut self) {
        let mut active_senders = VecDeque::with_capacity(self.senders.len());
        while let Some(sender) = self.senders.pop_front() {
            if sender.cancelled.load(Ordering::Acquire) {
                let _ = sender.result.send(Err("channel send cancelled".into()));
            } else {
                active_senders.push_back(sender);
            }
        }
        self.senders = active_senders;

        let mut active_receivers = VecDeque::with_capacity(self.receivers.len());
        while let Some(receiver) = self.receivers.pop_front() {
            if receiver.cancelled.load(Ordering::Acquire) {
                let _ = receiver
                    .result
                    .send(Err("channel receive cancelled".into()));
            } else {
                active_receivers.push_back(receiver);
            }
        }
        self.receivers = active_receivers;
    }

    fn fail_all(&mut self, message: &str) {
        for sender in self.senders.drain(..) {
            let _ = sender.result.send(Err(message.into()));
        }
        for receiver in self.receivers.drain(..) {
            let _ = receiver.result.send(Err(message.into()));
        }
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    #[test]
    fn bounded_channel_suspends_sender_until_capacity_is_released() {
        let mut hub = ChannelHub::default();
        hub.create(7, 1, MAX_CHANNELS).unwrap();
        let first = hub
            .submit(
                NativeChannelRequest::Send {
                    handle: 7,
                    value: Value::int(1),
                },
                active(),
                MAX_PENDING_OPERATIONS,
            )
            .unwrap();
        assert!(first.recv().unwrap().is_ok());
        let blocked = hub
            .submit(
                NativeChannelRequest::Send {
                    handle: 7,
                    value: Value::int(2),
                },
                active(),
                MAX_PENDING_OPERATIONS,
            )
            .unwrap();
        assert!(blocked.try_recv().is_err());
        let receive = hub
            .submit(
                NativeChannelRequest::Receive { handle: 7 },
                active(),
                MAX_PENDING_OPERATIONS,
            )
            .unwrap();
        assert!(matches!(
            receive.recv().unwrap().unwrap(),
            NativeAsyncOutput::Value(value) if value.as_int() == Some(1)
        ));
        assert!(blocked.recv().unwrap().is_ok());
    }
}
