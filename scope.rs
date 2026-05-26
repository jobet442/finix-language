/// Represents a local variable at compile-time.
#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: usize,
}

/// Manages compile-time lexical scoping and symbol resolution.
pub struct ScopeManager {
    pub locals: Vec<Local>,
    pub scope_depth: usize,
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeManager {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) -> usize {
        self.scope_depth -= 1;
        let mut popped = 0;
        // Remove locals that are now out of scope
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
                popped += 1;
            } else {
                break;
            }
        }
        popped
    }

    pub fn add_local(&mut self, name: String) {
        self.locals.push(Local { name, depth: self.scope_depth });
    }

    /// Returns the stack index of a local variable, or None if it's undefined.
    pub fn resolve_local(&self, name: &str) -> Option<usize> {
        self.locals.iter().rposition(|l| l.name == name)
    }
}