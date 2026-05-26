use ast::{Expr, LiteralValue, Op, Program, Stmt, UnaryOp, LogicalOp};
use crate::chunk::*;
use crate::value::Value;

struct Local {
    name: String,
    depth: isize,
}

pub struct BytecodeCompiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: isize,
}

impl Default for BytecodeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn compile(mut self, program: &Program) -> Result<Chunk, String> {
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }
        // Implicit return at the end of the program
        self.chunk.write(OP_RETURN, 0);
        Ok(self.chunk)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression { expr, pos } => {
                self.compile_expr(expr)?;
                // Pop the result of expression statements so stack stays clean
                self.emit_byte(OP_POP, pos.line);
                Ok(())
            }
            Stmt::Let { name, initializer, pos, .. } => {
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    self.emit_byte(OP_NULL, pos.line);
                }
                
                // Add variable to locals list
                self.locals.push(Local {
                    name: name.clone(),
                    depth: self.scope_depth,
                });
                
                Ok(())
            }
            Stmt::Block { statements, pos } => {
                self.scope_depth += 1;
                for inner_stmt in statements {
                    self.compile_stmt(inner_stmt)?;
                }
                
                // Pop variables declared in this scope block
                let depth = self.scope_depth;
                self.scope_depth -= 1;
                
                while let Some(local) = self.locals.last() {
                    if local.depth == depth {
                        self.emit_byte(OP_POP, pos.line);
                        self.locals.pop();
                    } else {
                        break;
                    }
                }
                
                Ok(())
            }
            Stmt::If { condition, then_branch, else_branch, pos } => {
                // 1. Evaluate condition
                self.compile_expr(condition)?;
                
                // 2. Jump if false to else/end
                let jump_if_false_instr = self.emit_jump(OP_JUMP_IF_FALSE, pos.line);
                
                // 3. Pop condition (on true branch)
                self.emit_byte(OP_POP, pos.line);
                
                // 4. Execute then branch
                self.compile_stmt(then_branch)?;
                
                // 5. Jump over else branch
                let jump_over_else = self.emit_jump(OP_JUMP, pos.line);
                
                // 6. Patch jump if false to land here (at else branch or end)
                self.patch_jump(jump_if_false_instr);
                
                // 7. Pop condition (on false/else branch)
                self.emit_byte(OP_POP, pos.line);
                
                // 8. Execute else branch if present
                if let Some(el) = else_branch {
                    self.compile_stmt(el)?;
                }
                
                // 9. Patch jump over else to land here
                self.patch_jump(jump_over_else);
                
                Ok(())
            }
            Stmt::While { condition, body, pos } => {
                let start_of_loop = self.chunk.code.len();
                
                // Evaluate condition
                self.compile_expr(condition)?;
                
                // Jump if false to end of loop
                let exit_jump = self.emit_jump(OP_JUMP_IF_FALSE, pos.line);
                
                // Pop condition inside loop
                self.emit_byte(OP_POP, pos.line);
                
                // Compile body
                self.compile_stmt(body)?;
                
                // Jump back to start of loop
                self.emit_loop(start_of_loop, pos.line);
                
                // Patch the exit jump to land here
                self.patch_jump(exit_jump);
                
                // Pop condition when exiting loop
                self.emit_byte(OP_POP, pos.line);
                
                Ok(())
            }
            Stmt::Return { value, pos } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    self.emit_byte(OP_NULL, pos.line);
                }
                self.emit_byte(OP_RETURN, pos.line);
                Ok(())
            }
            _ => Err("Statement type not supported in bytecode compilation".to_string()),
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Literal { value, pos } => {
                match value {
                    LiteralValue::Int(n) => {
                        let idx = self.chunk.add_constant(Value::Int(*n));
                        self.emit_byte(OP_CONSTANT, pos.line);
                        self.emit_byte(idx as u8, pos.line);
                    }
                    LiteralValue::Float(f) => {
                        let idx = self.chunk.add_constant(Value::Float(*f));
                        self.emit_byte(OP_CONSTANT, pos.line);
                        self.emit_byte(idx as u8, pos.line);
                    }
                    LiteralValue::String(s) => {
                        let idx = self.chunk.add_constant(Value::String(s.clone()));
                        self.emit_byte(OP_CONSTANT, pos.line);
                        self.emit_byte(idx as u8, pos.line);
                    }
                    LiteralValue::Boolean(b) => {
                        if *b {
                            self.emit_byte(OP_TRUE, pos.line);
                        } else {
                            self.emit_byte(OP_FALSE, pos.line);
                        }
                    }
                    LiteralValue::Null => {
                        self.emit_byte(OP_NULL, pos.line);
                    }
                }
                Ok(())
            }
            Expr::Identifier { name, pos } => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_byte(OP_GET_LOCAL, pos.line);
                    self.emit_byte(slot as u8, pos.line);
                    Ok(())
                } else {
                    Err(format!("Undefined variable '{}' at line {}, col {}", name, pos.line, pos.col))
                }
            }
            Expr::Assign { name, value, pos } => {
                self.compile_expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_byte(OP_SET_LOCAL, pos.line);
                    self.emit_byte(slot as u8, pos.line);
                    Ok(())
                } else {
                    Err(format!("Cannot assign to undefined variable '{}' at line {}, col {}", name, pos.line, pos.col))
                }
            }
            Expr::Unary { op, right, pos } => {
                self.compile_expr(right)?;
                match op {
                    UnaryOp::Neg => self.emit_byte(OP_NEGATE, pos.line),
                    UnaryOp::Not => self.emit_byte(OP_NOT, pos.line),
                }
                Ok(())
            }
            Expr::Binary { left, op, right, pos } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    Op::Add => self.emit_byte(OP_ADD, pos.line),
                    Op::Sub => self.emit_byte(OP_SUB, pos.line),
                    Op::Mul => self.emit_byte(OP_MUL, pos.line),
                    Op::Div => self.emit_byte(OP_DIV, pos.line),
                    Op::Equal => self.emit_byte(OP_EQUAL, pos.line),
                    Op::NotEqual => {
                        self.emit_byte(OP_EQUAL, pos.line);
                        self.emit_byte(OP_NOT, pos.line);
                    }
                    Op::GreaterThan => self.emit_byte(OP_GREATER, pos.line),
                    Op::LessThan => self.emit_byte(OP_LESS, pos.line),
                    Op::GreaterEqual => {
                        // a >= b is equivalent to !(a < b)
                        self.emit_byte(OP_LESS, pos.line);
                        self.emit_byte(OP_NOT, pos.line);
                    }
                    Op::LessEqual => {
                        // a <= b is equivalent to !(a > b)
                        self.emit_byte(OP_GREATER, pos.line);
                        self.emit_byte(OP_NOT, pos.line);
                    }
                    _ => return Err(format!("Binary operator {:?} not supported in VM compiler", op)),
                }
                Ok(())
            }
            Expr::Logical { left, op, right, pos } => {
                self.compile_expr(left)?;
                match op {
                    LogicalOp::And => {
                        // Short-circuiting AND: jump if left is false
                        let jump = self.emit_jump(OP_JUMP_IF_FALSE, pos.line);
                        self.emit_byte(OP_POP, pos.line); // Pop left result
                        self.compile_expr(right)?;
                        self.patch_jump(jump);
                    }
                    LogicalOp::Or => {
                        // Short-circuiting OR: jump if left is true (i.e. jump if false over the jump to the end)
                        let else_jump = self.emit_jump(OP_JUMP_IF_FALSE, pos.line);
                        let end_jump = self.emit_jump(OP_JUMP, pos.line);
                        self.patch_jump(else_jump);
                        self.emit_byte(OP_POP, pos.line);
                        self.compile_expr(right)?;
                        self.patch_jump(end_jump);
                    }
                }
                Ok(())
            }
            Expr::Call { callee, arguments, pos } => {
                // If it's a builtin "print" or "println", compile it as special print opcodes for our visual playground
                if let Expr::Identifier { name, .. } = &**callee {
                    if name == "print" || name == "println" {
                        if arguments.len() != 1 {
                            return Err(format!("'{}' takes exactly 1 argument", name));
                        }
                        self.compile_expr(&arguments[0])?;
                        if name == "print" {
                            self.emit_byte(OP_PRINT, pos.line);
                        } else {
                            self.emit_byte(OP_PRINTLN, pos.line);
                        }
                        return Ok(());
                    }
                }
                Err("User function calls are not implemented in the current VM bytecode compiler version.".to_string())
            }
            _ => Err("Expression not supported in VM compilation".to_string()),
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    fn emit_byte(&mut self, byte: u8, line: usize) {
        self.chunk.write(byte, line);
    }

    fn emit_jump(&mut self, op: u8, line: usize) -> usize {
        self.emit_byte(op, line);
        self.emit_byte(0xff, line); // Placeholder 16-bit offset
        self.emit_byte(0xff, line);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;
        if jump > u16::MAX as usize {
            panic!("Too much code to jump over!");
        }
        self.chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
        self.chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    fn emit_loop(&mut self, loop_start: usize, line: usize) {
        self.emit_byte(OP_LOOP, line);
        let offset = self.chunk.code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            panic!("Loop body too large!");
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8, line);
        self.emit_byte((offset & 0xff) as u8, line);
    }
}
