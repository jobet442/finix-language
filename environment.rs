use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use ast::Position;
use crate::value::Value;
use crate::error::RuntimeError;

/// Maps identifiers to values, supporting lexical scope chains.
#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Creates a new global environment.
    pub fn new() -> Self {
        Self { values: HashMap::new(), enclosing: None }
    }

    /// Creates a locally scoped environment wrapped inside a parent environment.
    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self { values: HashMap::new(), enclosing: Some(enclosing) }
    }

    /// Declares a new variable in the current scope.
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// Retrieves a variable from the current or an enclosing scope.
    pub fn get(&self, name: &str, pos: &Position) -> Result<Value, RuntimeError> {
        if let Some(val) = self.values.get(name) {
            return Ok(val.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name, pos);
        }
        Err(RuntimeError::UndefinedVariable { name: name.to_string(), pos: *pos })
    }

    /// Assigns a new value to an existing variable.
    pub fn assign(&mut self, name: &str, value: Value, pos: &Position) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value, pos);
        }
        Err(RuntimeError::UndefinedVariable { name: name.to_string(), pos: *pos })
    }
}