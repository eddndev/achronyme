use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use memory::Value;

use crate::machine::blocking_pool::{BlockingJobControl, JobReceiver};
use crate::machine::frame::CallFrame;

#[derive(Debug)]
pub(super) struct TaskContext {
    pub(super) stack: Vec<Value>,
    pub(super) frames: Vec<CallFrame>,
    pub(super) last_result: Value,
    pub(super) instruction_budget: u64,
}

impl TaskContext {
    pub(super) fn roots(&self, roots: &mut Vec<Value>) {
        roots.extend_from_slice(&self.stack);
        roots.extend(
            self.frames
                .iter()
                .map(|frame| Value::closure(frame.closure)),
        );
    }
}

#[derive(Debug)]
pub(super) enum TaskState {
    Ready { instruction_budget: u64 },
    ReadyOutput(JobReceiver),
    Runnable(TaskContext),
    Pending(JobReceiver, PendingKind),
    Running,
    Suspended(TaskContext),
    Waiting(TaskContext, TaskWait),
    Completed(Value),
    Consumed,
    Failed(String),
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PendingKind {
    External,
    Internal,
}

#[derive(Clone, Debug)]
pub(super) struct TaskWait {
    pub(super) targets: Vec<u32>,
    pub(super) destination: usize,
    pub(super) mode: TaskWaitMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskWaitMode {
    Propagate,
    Outcome,
    Race,
}

#[derive(Debug)]
pub(super) struct TaskRecord {
    pub(super) parent: Option<u32>,
    pub(super) owner_scope: u32,
    pub(super) spawn_function: String,
    pub(super) spawn_line: u32,
    pub(super) last_function: Option<String>,
    pub(super) last_line: Option<u32>,
    pub(super) callee: Value,
    pub(super) args: Vec<Value>,
    pub(super) state: TaskState,
    pub(super) cancel_requested: Arc<AtomicBool>,
    pub(super) blocking_job: Option<Arc<BlockingJobControl>>,
    pub(super) borrowed_resources: Vec<u32>,
    pub(super) close_on_terminal: Vec<u32>,
    pub(super) capture_failure: bool,
    pub(super) observed: bool,
}

impl TaskRecord {
    fn root() -> Self {
        Self {
            parent: None,
            owner_scope: 0,
            spawn_function: "main".to_string(),
            spawn_line: 0,
            last_function: Some("main".to_string()),
            last_line: None,
            callee: Value::nil(),
            args: Vec::new(),
            state: TaskState::Running,
            cancel_requested: Arc::new(AtomicBool::new(false)),
            blocking_job: None,
            borrowed_resources: Vec::new(),
            close_on_terminal: Vec::new(),
            capture_failure: false,
            observed: true,
        }
    }

    fn roots(&self, roots: &mut Vec<Value>) {
        match &self.state {
            TaskState::Ready { .. }
            | TaskState::ReadyOutput(_)
            | TaskState::Pending(_, _)
            | TaskState::Running => {
                roots.push(self.callee);
                roots.extend_from_slice(&self.args);
            }
            TaskState::Runnable(context)
            | TaskState::Suspended(context)
            | TaskState::Waiting(context, _) => {
                roots.push(self.callee);
                roots.extend_from_slice(&self.args);
                context.roots(roots);
            }
            TaskState::Completed(value) => roots.push(*value),
            TaskState::Consumed | TaskState::Failed(_) | TaskState::Cancelled => {}
        }
    }
}

#[derive(Debug)]
struct TaskSlot {
    generation: u16,
    record: Option<TaskRecord>,
    last_consumed_generation: Option<u16>,
}

/// Generational storage for live task records.
///
/// A handle encodes a 16-bit slot and a 16-bit generation. Reusing a slot
/// therefore cannot make a stale task value point at a later child.
#[derive(Debug, Default)]
pub(super) struct TaskTable {
    slots: Vec<TaskSlot>,
    free: Vec<u16>,
    live: usize,
}

impl TaskTable {
    const MAX_SLOTS: usize = u16::MAX as usize;

    pub(super) fn reserve(&mut self) -> Option<u32> {
        let slot = if let Some(slot) = self.free.pop() {
            slot
        } else {
            if self.slots.len() >= Self::MAX_SLOTS {
                return None;
            }
            let slot = self.slots.len() as u16;
            self.slots.push(TaskSlot {
                generation: 1,
                record: None,
                last_consumed_generation: None,
            });
            slot
        };
        self.live += 1;
        Some(Self::encode(slot, self.slots[slot as usize].generation))
    }

    pub(super) fn insert(&mut self, handle: u32, record: TaskRecord) -> bool {
        let Some(slot) = self.slot_mut(handle) else {
            return false;
        };
        if slot.record.is_some() {
            return false;
        }
        slot.record = Some(record);
        true
    }

    pub(super) fn get(&self, handle: u32) -> Option<&TaskRecord> {
        self.slot(handle)?.record.as_ref()
    }

    pub(super) fn get_mut(&mut self, handle: u32) -> Option<&mut TaskRecord> {
        self.slot_mut(handle)?.record.as_mut()
    }

    pub(super) fn release(&mut self, handle: u32) -> Option<TaskRecord> {
        self.release_with_status(handle, false)
    }

    pub(super) fn release_consumed(&mut self, handle: u32) -> Option<TaskRecord> {
        self.release_with_status(handle, true)
    }

    fn release_with_status(&mut self, handle: u32, consumed: bool) -> Option<TaskRecord> {
        let (slot_index, generation) = Self::decode(handle);
        let slot = self.slots.get_mut(slot_index as usize)?;
        if slot.generation != generation {
            return None;
        }
        let record = slot.record.take();
        slot.last_consumed_generation = consumed.then_some(generation);
        if self.live != 0 {
            self.live -= 1;
        }
        if slot.generation != u16::MAX {
            slot.generation += 1;
            self.free.push(slot_index);
        }
        record
    }

    pub(super) fn was_consumed(&self, handle: u32) -> bool {
        let (slot_index, generation) = Self::decode(handle);
        self.slots
            .get(slot_index as usize)
            .is_some_and(|slot| slot.last_consumed_generation == Some(generation))
    }

    pub(super) fn release_reserved(&mut self, handle: u32) {
        let _ = self.release(handle);
    }

    pub(super) fn len(&self) -> usize {
        self.live
    }

    pub(super) fn handles(&self) -> impl Iterator<Item = u32> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.record
                .as_ref()
                .map(|_| Self::encode(index as u16, slot.generation))
        })
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (u32, &TaskRecord)> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.record
                .as_ref()
                .map(|record| (Self::encode(index as u16, slot.generation), record))
        })
    }

    fn slot(&self, handle: u32) -> Option<&TaskSlot> {
        let (slot, generation) = Self::decode(handle);
        self.slots
            .get(slot as usize)
            .filter(|entry| entry.generation == generation)
    }

    fn slot_mut(&mut self, handle: u32) -> Option<&mut TaskSlot> {
        let (slot, generation) = Self::decode(handle);
        self.slots
            .get_mut(slot as usize)
            .filter(|entry| entry.generation == generation)
    }

    const fn encode(slot: u16, generation: u16) -> u32 {
        ((generation as u32) << 16) | slot as u32
    }

    const fn decode(handle: u32) -> (u16, u16) {
        (handle as u16, (handle >> 16) as u16)
    }
}

#[derive(Debug)]
pub(super) struct TaskScope {
    pub(super) id: u32,
    pub(super) owner_task: Option<u32>,
    pub(super) children: Vec<u32>,
}

#[derive(Debug)]
pub(crate) struct TaskScheduler {
    pub(super) scopes: Vec<TaskScope>,
    pub(super) tasks: TaskTable,
    pub(super) current_tasks: Vec<u32>,
    pub(super) ready: VecDeque<u32>,
    pub(super) root_task: TaskRecord,
    pub(super) next_scope_id: u32,
    pub(super) pending_wait: Option<TaskWait>,
    pub(super) last_failure: Option<super::diagnostics::TaskFailureDiagnostic>,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self {
            scopes: Vec::new(),
            tasks: TaskTable::default(),
            current_tasks: Vec::new(),
            ready: VecDeque::new(),
            root_task: TaskRecord::root(),
            next_scope_id: 0,
            pending_wait: None,
            last_failure: None,
        }
    }
}

impl TaskScheduler {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        self.root_task.roots(&mut roots);
        for (_, task) in self.tasks.iter() {
            task.roots(&mut roots);
        }
        roots
    }

    pub(super) fn pending_native_request_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|(_, task)| matches!(task.state, TaskState::Pending(_, _)))
            .count()
    }

    pub(super) fn retained_task_result_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|(_, task)| matches!(task.state, TaskState::Completed(_)))
            .count()
    }

    pub(crate) fn current_task(&self) -> Option<u32> {
        self.current_tasks.last().copied()
    }
}

#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;
