use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

#[derive(Clone)]
pub(super) struct Scope {
    inner: Rc<RefCell<InnerScope>>,
}

pub struct InnerScope {
    variables: HashMap<String, Value>,
    parent: Option<Scope>,
}

impl Scope {
    pub(super) fn new(parent: Option<Scope>) -> Self {
        Scope {
            inner: Rc::new(RefCell::new(InnerScope {
                variables: HashMap::new(),
                parent,
            })),
        }
    }

    pub(super) fn define(&self, name: String, value: Value) {
        self.inner.borrow_mut().define(name, value);
    }

    pub(super) fn get(&self, name: &str) -> Option<Value> {
        self.inner.borrow().get(name)
    }

    pub(super) fn set(&self, name: &str, value: Value) -> Result<(), String> {
        self.inner.borrow_mut().set(name, value)
    }
}
impl InnerScope {
    fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(parent_scope) = &self.parent {
            parent_scope.get(name)
        } else {
            None
        }
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(slot) = self.variables.get_mut(name) {
            *slot = value;
            Ok(())
        } else if let Some(parent_scope) = &self.parent {
            parent_scope.set(name, value)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }
}
