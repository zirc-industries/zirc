//! Environment and bindings for the Zirc interpreter.

use std::collections::HashMap;

use crate::value::Value;
use zirc_syntax::ast::Type;
use zirc_syntax::error::Result;

#[derive(Clone)]
pub struct Binding {
    /// The runtime value of this binding
    pub value: Value,
    /// Optional type annotation for runtime type checking
    pub ty: Option<Type>,
}

#[derive(Clone)]
pub struct Env<'a> {
    /// Variables defined in this scope
    vars: HashMap<String, Binding>,
    /// Reference to parent environment (None for root scope)
    parent: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    pub fn new_root() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }
    pub(crate) fn child(&'a self) -> Env<'a> {
        Env {
            vars: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn vars_snapshot(&self) -> Vec<(String, Value)> {
        self.vars
            .iter()
            .map(|(k, b)| (k.clone(), b.value.clone()))
            .collect()
    }

    pub(crate) fn get(&self, name: &str) -> Option<Binding> {
        if let Some(b) = self.vars.get(name) {
            Some(b.clone())
        } else {
            self.parent.and_then(|p| p.get(name))
        }
    }

    pub(crate) fn define(&mut self, name: String, val: Value, ty: Option<Type>) {
        self.vars.insert(name, Binding { value: val, ty });
    }

    pub(crate) fn assign(&mut self, name: &str, val: Value) -> Result<()> {
        if let Some(b) = self.vars.get_mut(name) {
            if let Some(t) = &b.ty {
                crate::interpreter::Interpreter::check_type(&val, t)?;
            }
            b.value = val;
            Ok(())
        } else {
            zirc_syntax::error::error(format!("Assignment to undefined variable '{}'", name))
        }
    }
}

