use crate::Type;
use std::collections::HashMap;

/// Lexical scope manager for the static type checker.
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    scopes: Vec<HashMap<String, Type>>,
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    pub fn get(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
}