use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use memory::Value;

use crate::error::RuntimeError;

use super::state::{PendingKind, TaskRecord, TaskScope, TaskState};
use super::VM;

impl VM {
    pub(super) fn enter_task_scope(&mut self) -> Result<(), RuntimeError> {
        if self.task_scheduler.scopes.len() >= self.runtime_limits.max_task_scopes {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "live task scopes exceed {}",
                self.runtime_limits.max_task_scopes
            )));
        }
        let id = self.task_scheduler.next_scope_id;
        self.task_scheduler.next_scope_id = id
            .checked_add(1)
            .ok_or_else(|| RuntimeError::resource_limit_exceeded("task scope id overflow"))?;
        self.task_scheduler.scopes.push(TaskScope {
            id,
            owner_task: self.task_scheduler.current_task(),
            children: Vec::new(),
        });
        Ok(())
    }

    pub(super) fn spawn_task(
        &mut self,
        callee: Value,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let owner = self.task_scheduler.current_task();
        let (spawn_function, spawn_line) = self
            .current_source_location()
            .unwrap_or_else(|| ("main".to_string(), 0));
        let task_function = self
            .task_callable_name(callee)
            .unwrap_or_else(|| spawn_function.clone());
        let owner_scope = self
            .task_scheduler
            .scopes
            .iter()
            .rev()
            .find(|scope| scope.owner_task == owner)
            .map(|scope| scope.id)
            .ok_or(RuntimeError::TaskOutOfScope)?;
        if self.task_scheduler.tasks.len() >= self.runtime_limits.max_tasks {
            return Err(RuntimeError::resource_limit_exceeded(format!(
                "live child task count exceeds {}",
                self.runtime_limits.max_tasks
            )));
        }
        let id = self.task_scheduler.tasks.reserve().ok_or_else(|| {
            RuntimeError::resource_limit_exceeded("task handle generations are exhausted")
        })?;
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let prepared = match self.prepare_async_native(callee, &args) {
            Ok(prepared) => prepared,
            Err(error) => {
                self.task_scheduler.tasks.release_reserved(id);
                return Err(error);
            }
        };
        let (state, claims, blocking_job, enqueue_ready) = match prepared {
            Some(prepared) => {
                let crate::native::NativeAsyncRequest {
                    work,
                    created_resources,
                } = prepared.request;
                let claims = match self.claim_async_resources(
                    id,
                    owner,
                    &args,
                    prepared.resource,
                    &created_resources,
                ) {
                    Ok(claims) => claims,
                    Err(error) => {
                        self.task_scheduler.tasks.release_reserved(id);
                        return Err(error);
                    }
                };
                let requires_pending_slot = !matches!(&work, crate::native::NativeAsyncWork::Yield);
                if requires_pending_slot
                    && self.task_scheduler.pending_native_request_count()
                        >= self.runtime_limits.max_pending_native_requests
                {
                    self.rollback_task_resource_claims(id, owner, claims);
                    self.task_scheduler.tasks.release_reserved(id);
                    return Err(RuntimeError::resource_limit_exceeded(format!(
                        "pending native requests exceed {}",
                        self.runtime_limits.max_pending_native_requests
                    )));
                }
                let dispatched = match work {
                    crate::native::NativeAsyncWork::Blocking(job) => {
                        self.blocking_pool.submit(job).map(|submission| {
                            (
                                TaskState::Pending(submission.receiver, PendingKind::External),
                                Some(submission.control),
                                false,
                            )
                        })
                    }
                    crate::native::NativeAsyncWork::Network(request) => self
                        .network_reactor
                        .submit(request, Arc::clone(&cancel_requested))
                        .map(|receiver| {
                            (
                                TaskState::Pending(receiver, PendingKind::External),
                                None,
                                false,
                            )
                        }),
                    crate::native::NativeAsyncWork::Channel(request) => self
                        .channel_hub
                        .submit(
                            request,
                            Arc::clone(&cancel_requested),
                            self.runtime_limits.max_channel_operations,
                        )
                        .map(|receiver| {
                            (
                                TaskState::Pending(receiver, PendingKind::Internal),
                                None,
                                false,
                            )
                        }),
                    crate::native::NativeAsyncWork::Yield => {
                        let (result, receiver) = std::sync::mpsc::channel();
                        let _ = result.send(Ok(crate::native::NativeAsyncOutput::Nil));
                        Ok((TaskState::ReadyOutput(receiver), None, true))
                    }
                    crate::native::NativeAsyncWork::Timer(duration) => self
                        .timer_reactor
                        .submit(duration, Arc::clone(&cancel_requested))
                        .map(|receiver| {
                            (
                                TaskState::Pending(receiver, PendingKind::External),
                                None,
                                false,
                            )
                        }),
                };
                match dispatched {
                    Ok((state, blocking_job, enqueue_ready)) => {
                        (state, claims, blocking_job, enqueue_ready)
                    }
                    Err(error) => {
                        self.rollback_task_resource_claims(id, owner, claims);
                        self.task_scheduler.tasks.release_reserved(id);
                        return Err(error);
                    }
                }
            }
            None => {
                let claims = match self.claim_user_task_resources(id, owner, &args) {
                    Ok(claims) => claims,
                    Err(error) => {
                        self.task_scheduler.tasks.release_reserved(id);
                        return Err(error);
                    }
                };
                (
                    TaskState::Ready {
                        instruction_budget: self.instruction_budget,
                    },
                    claims,
                    None,
                    true,
                )
            }
        };
        let inserted = self.task_scheduler.tasks.insert(
            id,
            TaskRecord {
                parent: owner,
                owner_scope,
                spawn_function: spawn_function.clone(),
                spawn_line,
                last_function: Some(task_function),
                last_line: None,
                callee,
                args,
                state,
                cancel_requested,
                blocking_job,
                borrowed_resources: claims.borrowed,
                close_on_terminal: claims.close_on_terminal,
                capture_failure: false,
                observed: true,
            },
        );
        if !inserted {
            self.task_scheduler.tasks.release_reserved(id);
            return Err(RuntimeError::task_failed(
                "reserved task slot could not be initialized",
            ));
        }
        self.task_scheduler
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.id == owner_scope)
            .expect("owning scope checked above")
            .children
            .push(id);
        if enqueue_ready {
            self.task_scheduler.ready.push_back(id);
        }
        Ok(Value::task(id))
    }

    fn task_callable_name(&self, callee: Value) -> Option<String> {
        if callee.is_closure() {
            let closure = self.heap.get_closure(callee.as_handle()?)?;
            return self
                .heap
                .get_function(closure.function)
                .map(|function| function.name.clone());
        }
        if callee.is_native() {
            return self
                .natives
                .get(callee.as_handle()? as usize)
                .map(|native| native.name.clone());
        }
        None
    }
}
