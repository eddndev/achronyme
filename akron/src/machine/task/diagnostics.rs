use std::fmt;

use super::state::{PendingKind, TaskRecord, TaskState, TaskWaitMode};
use super::VM;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskDiagnosticState {
    Ready,
    Pending,
    Running,
    Suspended,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskAwaitMode {
    Propagate,
    Outcome,
    Race,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskWaitReason {
    HostOperation,
    InternalOperation,
    Children {
        tasks: Vec<u32>,
        mode: TaskAwaitMode,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDiagnostic {
    pub id: u32,
    pub parent: Option<u32>,
    pub spawn_function: String,
    pub spawn_line: u32,
    pub last_function: Option<String>,
    pub last_line: Option<u32>,
    pub state: TaskDiagnosticState,
    pub wait_reason: Option<TaskWaitReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskFailureDiagnostic {
    pub task: TaskDiagnostic,
    pub message: String,
    pub cleanup_failures: Vec<String>,
}

impl fmt::Display for TaskFailureDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "task {}", self.task.id)?;
        if let Some(parent) = self.task.parent {
            write!(formatter, " (parent {parent})")?;
        }
        write!(
            formatter,
            " spawned at {}:{}",
            self.task.spawn_function, self.task.spawn_line
        )?;
        if let Some(function) = &self.task.last_function {
            write!(formatter, ", failed in {function}")?;
            if let Some(line) = self.task.last_line {
                write!(formatter, ":{line}")?;
            }
        }
        write!(formatter, ": {}", self.message)?;
        for cleanup in &self.cleanup_failures {
            write!(formatter, "\n  cleanup: {cleanup}")?;
        }
        Ok(())
    }
}

impl VM {
    /// Snapshot every live task without exposing scheduler internals.
    pub fn task_diagnostics(&self) -> Vec<TaskDiagnostic> {
        self.task_scheduler
            .tasks
            .iter()
            .map(|(id, task)| snapshot(id, task))
            .collect()
    }

    /// Primary unhandled child failure retained after scope cleanup.
    pub fn last_task_failure(&self) -> Option<&TaskFailureDiagnostic> {
        self.task_scheduler.last_failure.as_ref()
    }

    pub(crate) fn clear_task_failure(&mut self) {
        self.task_scheduler.last_failure = None;
    }

    pub(super) fn update_task_location(&mut self, id: u32, location: Option<(String, u32)>) {
        let Some((function, line)) = location else {
            return;
        };
        if let Some(task) = self.task_scheduler.tasks.get_mut(id) {
            task.last_function = Some(function);
            task.last_line = (line > 0).then_some(line);
        }
    }

    pub(super) fn record_task_failure(&mut self, id: u32, message: String) {
        if let Some(primary) = self.task_scheduler.last_failure.as_ref() {
            if primary.task.id == id || primary.message == message {
                return;
            }
            let cleanup = self
                .task_scheduler
                .tasks
                .get(id)
                .map(|task| {
                    TaskFailureDiagnostic {
                        task: snapshot(id, task),
                        message: message.clone(),
                        cleanup_failures: Vec::new(),
                    }
                    .to_string()
                })
                .unwrap_or_else(|| format!("task {id}: {message}"));
            let primary = self
                .task_scheduler
                .last_failure
                .as_mut()
                .expect("primary task failure checked above");
            if !primary.cleanup_failures.contains(&cleanup) {
                primary.cleanup_failures.push(cleanup);
            }
            return;
        }
        let Some(task) = self.task_scheduler.tasks.get(id) else {
            return;
        };
        let mut task = snapshot(id, task);
        task.state = TaskDiagnosticState::Failed;
        self.task_scheduler.last_failure = Some(TaskFailureDiagnostic {
            task,
            message,
            cleanup_failures: Vec::new(),
        });
    }
}

fn snapshot(id: u32, task: &TaskRecord) -> TaskDiagnostic {
    let (state, wait_reason) = match &task.state {
        TaskState::Ready { .. } | TaskState::ReadyOutput(_) | TaskState::Runnable(_) => {
            (TaskDiagnosticState::Ready, None)
        }
        TaskState::Pending(_, kind) => (
            TaskDiagnosticState::Pending,
            Some(match kind {
                PendingKind::External => TaskWaitReason::HostOperation,
                PendingKind::Internal => TaskWaitReason::InternalOperation,
            }),
        ),
        TaskState::Running => (TaskDiagnosticState::Running, None),
        TaskState::Suspended(_) => (TaskDiagnosticState::Suspended, None),
        TaskState::Waiting(_, wait) => (
            TaskDiagnosticState::Waiting,
            Some(TaskWaitReason::Children {
                tasks: wait.targets.clone(),
                mode: match wait.mode {
                    TaskWaitMode::Propagate => TaskAwaitMode::Propagate,
                    TaskWaitMode::Outcome => TaskAwaitMode::Outcome,
                    TaskWaitMode::Race => TaskAwaitMode::Race,
                },
            }),
        ),
        TaskState::Completed(_) | TaskState::Consumed => (TaskDiagnosticState::Completed, None),
        TaskState::Failed(_) => (TaskDiagnosticState::Failed, None),
        TaskState::Cancelled => (TaskDiagnosticState::Cancelled, None),
    };
    TaskDiagnostic {
        id,
        parent: task.parent,
        spawn_function: task.spawn_function.clone(),
        spawn_line: task.spawn_line,
        last_function: task.last_function.clone(),
        last_line: task.last_line,
        state,
        wait_reason,
    }
}
