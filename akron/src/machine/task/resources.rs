use memory::{Value, ValueResourceKind};

use crate::error::RuntimeError;
use crate::resource::value_resource_kind;
use crate::specs::ResourceEffect;

use super::VM;

#[derive(Default)]
pub(super) struct TaskResourceClaims {
    pub(super) owned: Vec<u32>,
    pub(super) borrowed: Vec<u32>,
    pub(super) close_on_terminal: Vec<u32>,
    created: Vec<u32>,
}

impl VM {
    pub(super) fn claim_async_resources(
        &mut self,
        task: u32,
        owner: Option<u32>,
        args: &[Value],
        effect: ResourceEffect,
        created: &[u32],
    ) -> Result<TaskResourceClaims, RuntimeError> {
        let mut claims = TaskResourceClaims::default();
        let outcome = (|| -> Result<(), RuntimeError> {
            match effect {
                ResourceEffect::None => {
                    if created.is_empty() {
                        Ok(())
                    } else {
                        Err(RuntimeError::resource_error(
                            "native reserved a resource without declaring a create effect",
                        ))
                    }
                }
                ResourceEffect::Creates(kind) => {
                    self.claim_created_resource(task, owner, kind, created, &mut claims)
                }
                ResourceEffect::Borrows(kind) => {
                    if !created.is_empty() {
                        Err(RuntimeError::resource_error(
                            "borrowing native unexpectedly reserved a resource",
                        ))
                    } else {
                        self.claim_borrowed_resource(task, owner, kind, args, &mut claims)
                    }
                }
                ResourceEffect::Consumes(kind) => {
                    if !created.is_empty() {
                        Err(RuntimeError::resource_error(
                            "consuming native unexpectedly reserved a resource",
                        ))
                    } else {
                        let expected = value_resource_kind(kind)?;
                        let handle = find_resource_arg(args, expected)?;
                        self.resources.require(
                            Value::resource(expected, handle),
                            expected,
                            owner,
                        )?;
                        self.resources.transfer(handle, owner, Some(task))?;
                        claims.owned.push(handle);
                        claims.close_on_terminal.push(handle);
                        Ok(())
                    }
                }
                ResourceEffect::CreatesAndBorrows {
                    created: new,
                    borrowed,
                } => {
                    self.claim_created_resource(task, owner, new, created, &mut claims)?;
                    self.claim_borrowed_resource(task, owner, borrowed, args, &mut claims)
                }
            }
        })();
        if let Err(error) = outcome {
            self.rollback_task_resource_claims(task, owner, claims);
            for handle in created {
                self.close_resource_handle(*handle);
            }
            return Err(error);
        }
        Ok(claims)
    }

    pub(super) fn claim_user_task_resources(
        &mut self,
        task: u32,
        owner: Option<u32>,
        args: &[Value],
    ) -> Result<TaskResourceClaims, RuntimeError> {
        let mut claims = TaskResourceClaims::default();
        for argument in args {
            let Some((kind, handle)) = argument.as_resource_handle() else {
                continue;
            };
            if kind == ValueResourceKind::Channel {
                self.resources.require_channel(*argument)?;
                continue;
            }
            if let Err(error) = self.resources.require(*argument, kind, owner) {
                self.rollback_task_resource_claims(task, owner, claims);
                return Err(error);
            }
            if let Err(error) = self.resources.transfer(handle, owner, Some(task)) {
                self.rollback_task_resource_claims(task, owner, claims);
                return Err(error);
            }
            claims.owned.push(handle);
        }
        Ok(claims)
    }

    fn claim_created_resource(
        &mut self,
        task: u32,
        owner: Option<u32>,
        expected: crate::specs::ResourceKind,
        created: &[u32],
        claims: &mut TaskResourceClaims,
    ) -> Result<(), RuntimeError> {
        if created.len() != 1 {
            return Err(RuntimeError::resource_error(
                "resource-creating native must reserve exactly one handle",
            ));
        }
        let expected = value_resource_kind(expected)?;
        let handle = created[0];
        if self.resources.kind(handle)? != expected {
            return Err(RuntimeError::resource_error(
                "native reserved a resource of the wrong kind",
            ));
        }
        self.resources.transfer(handle, owner, Some(task))?;
        claims.owned.push(handle);
        claims.created.push(handle);
        Ok(())
    }

    fn claim_borrowed_resource(
        &mut self,
        task: u32,
        owner: Option<u32>,
        expected: crate::specs::ResourceKind,
        args: &[Value],
        claims: &mut TaskResourceClaims,
    ) -> Result<(), RuntimeError> {
        let expected = value_resource_kind(expected)?;
        let handle = find_resource_arg(args, expected)?;
        self.resources.borrow(handle, expected, owner, task)?;
        claims.borrowed.push(handle);
        Ok(())
    }

    pub(super) fn rollback_task_resource_claims(
        &mut self,
        task: u32,
        owner: Option<u32>,
        claims: TaskResourceClaims,
    ) {
        for handle in claims.borrowed {
            self.resources.release_borrow(handle, task);
        }
        for handle in claims.owned {
            if claims.created.contains(&handle) {
                self.close_resource_handle(handle);
            } else {
                let _ = self.resources.transfer(handle, Some(task), owner);
            }
        }
    }

    pub(super) fn finish_task_resources(&mut self, task: u32, failed: bool) {
        let Some(record) = self.task_scheduler.tasks.get(task) else {
            return;
        };
        let borrowed = record.borrowed_resources.clone();
        let close_on_terminal = record.close_on_terminal.clone();
        for handle in borrowed {
            self.resources.release_borrow(handle, task);
        }
        for handle in close_on_terminal {
            self.close_resource_handle(handle);
        }
        if failed {
            self.close_resources_owned_by(Some(task));
        }
    }

    pub(super) fn transfer_task_result(
        &mut self,
        task: u32,
        value: Value,
        parent: Option<u32>,
    ) -> Result<(), RuntimeError> {
        let Some((_kind, handle)) = value.as_resource_handle() else {
            return Ok(());
        };
        self.resources.transfer(handle, Some(task), parent)
    }
}

fn find_resource_arg(args: &[Value], expected: ValueResourceKind) -> Result<u32, RuntimeError> {
    args.iter()
        .find_map(|value| match value.as_resource_handle() {
            Some((kind, handle)) if kind == expected => Some(handle),
            _ => None,
        })
        .ok_or_else(|| {
            RuntimeError::type_mismatch(format!(
                "native requires an owned {expected:?} resource argument"
            ))
        })
}
