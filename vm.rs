use crate::chunk::*;
use crate::value::Value;
use crate::gc::Heap;

/// A single execution context within the VM.
/// Represents an active function call.
pub struct CallFrame {
    pub ip: usize,
    /// Where this frame's local variables begin on the global VM stack.
    pub stack_offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError(String),
}

/// The Finix Bytecode Virtual Machine.
pub struct Vm {
    pub chunk: Chunk,
    pub ip: usize,
    
    /// The main execution stack. Fast, contiguous memory!
    pub stack: Vec<Value>,
    
    /// Stack of active function calls
    pub _frames: Vec<CallFrame>,

    /// The garbage collector's heap
    pub heap: Heap,

    /// Accumulated output messages printed by Finix print/println statements
    pub output: Vec<String>,

    /// Tracks if VM execution has completed
    pub is_finished: bool,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(256),
            _frames: Vec::with_capacity(64),
            heap: Heap::new(),
            output: Vec::new(),
            is_finished: false,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        self.stack.clear();
        self._frames.clear();
        self.output.clear();
        self.is_finished = false;
        
        self.run()
    }

    /// The core execution loop of the Virtual Machine.
    pub fn run(&mut self) -> InterpretResult {
        while !self.is_finished {
            match self.step() {
                Ok(true) => break,
                Ok(false) => {}
                Err(err) => return InterpretResult::RuntimeError(err),
            }
        }
        InterpretResult::Ok
    }

    /// Execute a single instruction. Returns Ok(true) if execution is complete, Ok(false) otherwise.
    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.chunk.code.len() {
            self.is_finished = true;
            return Ok(true);
        }

        let instruction = self.read_byte();

        match instruction {
            OP_CONSTANT => {
                let constant = self.read_constant();
                self.stack.push(constant);
            }
            OP_NULL => self.stack.push(Value::Null),
            OP_TRUE => self.stack.push(Value::Boolean(true)),
            OP_FALSE => self.stack.push(Value::Boolean(false)),
            OP_POP => {
                self.stack.pop().ok_or_else(|| "Stack underflow on POP".to_string())?;
            }
            OP_ADD => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on ADD".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on ADD".to_string())?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x + y)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x + y)),
                    (Value::String(x), Value::String(y)) => self.stack.push(Value::String(format!("{}{}", x, y))),
                    (Value::String(x), other) => self.stack.push(Value::String(format!("{}{}", x, other))),
                    (other, Value::String(y)) => self.stack.push(Value::String(format!("{}{}", other, y))),
                    _ => return Err("Operands must be two numbers or two strings for ADD.".to_string()),
                }
            }
            OP_SUB => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on SUB".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on SUB".to_string())?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x - y)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x - y)),
                    _ => return Err("Operands must be two numbers for SUB.".to_string()),
                }
            }
            OP_MUL => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on MUL".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on MUL".to_string())?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x * y)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x * y)),
                    _ => return Err("Operands must be two numbers for MUL.".to_string()),
                }
            }
            OP_DIV => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on DIV".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on DIV".to_string())?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        if y == 0 {
                            return Err("Division by zero.".to_string());
                        }
                        self.stack.push(Value::Int(x / y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err("Division by zero.".to_string());
                        }
                        self.stack.push(Value::Float(x / y));
                    }
                    _ => return Err("Operands must be two numbers for DIV.".to_string()),
                }
            }
            OP_NEGATE => {
                let val = self.stack.pop().ok_or_else(|| "Stack underflow on NEGATE".to_string())?;
                match val {
                    Value::Int(x) => self.stack.push(Value::Int(-x)),
                    Value::Float(x) => self.stack.push(Value::Float(-x)),
                    _ => return Err("Operand must be a number for negation.".to_string()),
                }
            }
            OP_NOT => {
                let val = self.stack.pop().ok_or_else(|| "Stack underflow on NOT".to_string())?;
                match val {
                    Value::Boolean(b) => self.stack.push(Value::Boolean(!b)),
                    _ => return Err("Operand must be a boolean for NOT.".to_string()),
                }
            }
            OP_EQUAL => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on EQUAL".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on EQUAL".to_string())?;
                self.stack.push(Value::Boolean(a == b));
            }
            OP_GREATER => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on GREATER".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on GREATER".to_string())?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Boolean(x > y)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Boolean(x > y)),
                    _ => return Err("Operands must be two numbers for GREATER.".to_string()),
                }
            }
            OP_LESS => {
                let b = self.stack.pop().ok_or_else(|| "Stack underflow on LESS".to_string())?;
                let a = self.stack.pop().ok_or_else(|| "Stack underflow on LESS".to_string())?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Boolean(x < y)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Boolean(x < y)),
                    _ => return Err("Operands must be two numbers for LESS.".to_string()),
                }
            }
            OP_GET_LOCAL => {
                let slot = self.read_byte() as usize;
                if slot >= self.stack.len() {
                    return Err(format!("Invalid local slot index {} (stack size {})", slot, self.stack.len()));
                }
                let value = self.stack[slot].clone();
                self.stack.push(value);
            }
            OP_SET_LOCAL => {
                let slot = self.read_byte() as usize;
                if slot >= self.stack.len() {
                    return Err(format!("Invalid local slot index {} (stack size {})", slot, self.stack.len()));
                }
                self.stack[slot] = self.stack.last().unwrap().clone();
            }
            OP_JUMP => {
                let offset = self.read_short();
                self.ip += offset as usize;
            }
            OP_JUMP_IF_FALSE => {
                let offset = self.read_short();
                let cond = self.stack.last().ok_or_else(|| "Stack empty on JUMP_IF_FALSE".to_string())?;
                let is_false = match cond {
                    Value::Boolean(b) => !*b,
                    Value::Null => true,
                    _ => false,
                };
                if is_false {
                    self.ip += offset as usize;
                }
            }
            OP_LOOP => {
                let offset = self.read_short();
                self.ip -= offset as usize;
            }
            OP_PRINT => {
                let val = self.stack.pop().ok_or_else(|| "Stack underflow on PRINT".to_string())?;
                let text = format!("{}", val);
                print!("{}", text);
                self.output.push(text);
                self.stack.push(Value::Null);
            }
            OP_PRINTLN => {
                let val = self.stack.pop().ok_or_else(|| "Stack underflow on PRINTLN".to_string())?;
                let text = format!("{}\n", val);
                print!("{}", text);
                self.output.push(text);
                self.stack.push(Value::Null);
            }
            OP_RETURN => {
                self.is_finished = true;
                return Ok(true);
            }
            _ => return Err(format!("Unknown opcode: {}", instruction)),
        }

        if self.ip >= self.chunk.code.len() {
            self.is_finished = true;
            return Ok(true);
        }

        Ok(false)
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_short(&mut self) -> u16 {
        let b1 = self.read_byte() as u16;
        let b2 = self.read_byte() as u16;
        (b1 << 8) | b2
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        self.chunk.constants[index].clone()
    }

    /// Triggers a garbage collection cycle to reclaim unused heap memory.
    pub fn collect_garbage(&mut self) {
        #[cfg(debug_assertions)]
        println!("-- GC Begin --");

        // 1. Mark Roots (Execution Stack)
        for value in &self.stack {
            self.heap.mark_value(value);
        }

        // Mark Roots (Constants Pool)
        for constant in &self.chunk.constants {
            self.heap.mark_value(constant);
        }

        // 2. Trace through reachable object references
        self.heap.trace_references();

        // 3. Sweep unmarked objects from the heap
        self.heap.sweep();

        #[cfg(debug_assertions)]
        println!("-- GC End --");
    }
}