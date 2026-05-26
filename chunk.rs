use crate::value::Value;

/// Finix Bytecode Instruction Set
pub const OP_CONSTANT: u8 = 0;
pub const OP_NULL: u8     = 1;
pub const OP_TRUE: u8     = 2;
pub const OP_FALSE: u8    = 3;
pub const OP_POP: u8      = 4;
pub const OP_GET_LOCAL: u8 = 5;
pub const OP_SET_LOCAL: u8 = 6;
pub const OP_ADD: u8      = 7;
pub const OP_SUB: u8      = 8;
pub const OP_MUL: u8      = 9;
pub const OP_DIV: u8      = 10;
pub const OP_CALL: u8     = 11;
pub const OP_CLOSURE: u8  = 12;
pub const OP_RETURN: u8   = 13;
pub const OP_JUMP: u8     = 14;
pub const OP_JUMP_IF_FALSE: u8 = 15;
pub const OP_LOOP: u8     = 16;
pub const OP_EQUAL: u8    = 17;
pub const OP_GREATER: u8  = 18;
pub const OP_LESS: u8     = 19;
pub const OP_NOT: u8      = 20;
pub const OP_NEGATE: u8   = 21;
pub const OP_PRINT: u8    = 22;
pub const OP_PRINTLN: u8  = 23;

/// A sequence of bytecode instructions and their associated data.
#[derive(Default, Debug, Clone)]
pub struct Chunk {
    /// The flat bytecode instructions
    pub code: Vec<u8>,
    /// The constant pool (numbers, strings, etc.)
    pub constants: Vec<Value>,
    /// Line numbers matching each byte for accurate error tracking
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new instruction byte to the chunk.
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Adds a value to the constant pool and returns its index.
    /// Note: A real VM handles >256 constants via `OP_CONSTANT_16` instructions,
    /// but we use a `u8` index for simplicity here.
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}