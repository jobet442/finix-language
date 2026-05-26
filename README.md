# Finix Programming Language

Finix is a modern, statically optional, and strongly-typed programming language built in Rust. It aims to combine Python-like simplicity with Java-like structure, offering a clean developer experience that's both beginner-friendly and future-ready.

## Core Philosophy
- **Python-like simplicity**: Clean, readable syntax without unnecessary boilerplate.
- **Java-like structure**: Classes, interfaces, and modules for large-scale applications.
- **Optional static typing**: Start dynamically, add types as your project scales.
- **Fast execution**: Phased execution from Tree-Walk Interpreter to Bytecode VM, and eventually Native compilation.

## Architecture

The project is modularized into several Cargo workspace members:
- `ast`: Abstract Syntax Tree definitions.
- `lexer`: Tokenizer for converting source text into a stream of tokens.
- `parser`: Pratt parser for generating an AST from tokens.
- `interpreter`: Tree-Walk Interpreter (Phase 1).
- `compiler`: Bytecode compiler for flattening AST (Phase 2).
- `vm`: Stack-based virtual machine for fast execution (Phase 2).
- `std`: The Finix standard library.
- `cli`: Command-line interface, REPL, and entry point.

## Build Instructions

Ensure you have the latest stable Rust installed.

```bash
# Build the entire workspace
cargo build

# Run the tests
cargo test

# Run the Finix REPL
cargo run -p cli
```

## Development Roadmap

- [x] Define AST and Token structures.
- [x] Implement Lexer.
- [x] Implement Pratt Parser (Expressions and Statements).
- [ ] **Phase 1**: Tree-Walk Interpreter implementation (Variables, Control Flow, Functions, Classes).
- [ ] Complete standard library fundamentals.
- [ ] **Phase 2**: Bytecode Compiler and Virtual Machine implementation.
- [ ] Language Server Protocol (LSP) implementation.
- [ ] **Phase 3**: LLVM-based Native Compilation.