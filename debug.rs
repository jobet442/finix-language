use crate::chunk::*;

/// Disassembles and prints the contents of a bytecode chunk.
pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

/// Decodes a single instruction at the given offset.
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    let instruction = chunk.code[offset];
    match instruction {
        OP_RETURN => simple_instruction("OP_RETURN", offset),
        OP_POP => simple_instruction("OP_POP", offset),
        OP_NULL => simple_instruction("OP_NULL", offset),
        OP_TRUE => simple_instruction("OP_TRUE", offset),
        OP_FALSE => simple_instruction("OP_FALSE", offset),
        OP_ADD => simple_instruction("OP_ADD", offset),
        OP_SUB => simple_instruction("OP_SUB", offset),
        OP_MUL => simple_instruction("OP_MUL", offset),
        OP_DIV => simple_instruction("OP_DIV", offset),
        OP_CONSTANT => constant_instruction("OP_CONSTANT", chunk, offset),
        OP_GET_LOCAL => byte_instruction("OP_GET_LOCAL", chunk, offset),
        OP_SET_LOCAL => byte_instruction("OP_SET_LOCAL", chunk, offset),
        _ => {
            println!("Unknown opcode {}", instruction);
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.code[offset + 1];
    println!("{:-16} {:4}", name, slot);
    offset + 2
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant_index = chunk.code[offset + 1] as usize;
    print!("{:-16} {:4} '", name, constant_index);
    println!("{}'", chunk.constants[constant_index]);
    offset + 2
}