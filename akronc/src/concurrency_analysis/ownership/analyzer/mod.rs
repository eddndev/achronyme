use std::collections::HashMap;

use super::{Binding, FunctionSummary, NativeResources, ResourceBinding};

mod bindings;
mod expressions;
mod statements;

pub(super) struct Analyzer<'a> {
    natives: &'a NativeResources,
    functions: &'a HashMap<String, FunctionSummary>,
    scopes: Vec<HashMap<String, Binding>>,
}

impl<'a> Analyzer<'a> {
    pub(super) fn new(
        natives: &'a NativeResources,
        functions: &'a HashMap<String, FunctionSummary>,
    ) -> Self {
        Self {
            natives,
            functions,
            scopes: vec![HashMap::new()],
        }
    }

    pub(super) fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(super) fn declare(&mut self, name: &str, resource: Option<ResourceBinding>) {
        self.scopes
            .last_mut()
            .expect("ownership analyzer always has a scope")
            .insert(name.to_string(), Binding { resource });
    }

    pub(super) fn binding(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    pub(super) fn binding_mut(&mut self, name: &str) -> Option<&mut Binding> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }
}
