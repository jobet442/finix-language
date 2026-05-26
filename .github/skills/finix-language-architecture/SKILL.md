---
name: finix-language-architecture
user-invocable: true
description: |
  Guide the creation, extension, and review of Finix compiler components using Rust.
  Use this skill when designing or implementing lexer, parser, AST, interpreter, compiler, VM, CLI, runtime, or standard library modules with Python-inspired simplicity and Java-like structure.
---

# Finix Language Architecture Skill

## Purpose

This skill helps generate and evolve Finix language components in a structured, idiomatic Rust architecture.
It is designed for a modern language engineer building Finix with an eye on:
- Python-like simplicity
- Java-like structure
- optional static typing
- interpreter-to-VM evolution
- future native compilation

## When to Use

Use this skill for tasks such as:
- defining lexer/parser/AST module boundaries
- implementing language constructs (variables, functions, classes, interfaces, loops, conditionals, modules, imports)
- adding optional type annotations or error handling
- designing interpreter or bytecode VM architecture
- creating professional documentation and unit tests
- explaining compiler decisions, tradeoffs, and future extensibility

## Workflow

1. Clarify the target component and phase
   - Phase 1: Lexer → Parser → AST → Tree-Walk Interpreter
   - Phase 2: Bytecode Compiler → VM
   - Phase 3: LLVM-native compilation

2. Define the module contract
   - specify inputs/outputs and public API for the module
   - keep lexer and parser separate from AST and runtime
   - prefer small, composable types with clear ownership

3. Generate robust implementation guidance
   - choose idiomatic Rust structs/enums and lifetime-safe APIs
   - include error types, diagnostics, and recovery strategies
   - add documentation comments explaining compilation concepts

4. Create tests and examples
   - unit tests for lexer tokens, parser grammar, AST visitors, interpreter evaluation
   - integration tests for source-to-output behavior
   - sample programs illustrating language syntax and type semantics

5. Review architecture tradeoffs
   - explain why a tree-walk interpreter is a good Phase 1 choice
   - show how bytecode lowers from AST and why VM design matters
   - describe native compilation benefits and future-proofing decisions

## Quality Criteria

- Clear module separation: lexer, parser, ast, interpreter, compiler, vm, std, cli, tests, docs
- Strong error handling: descriptive diagnostics, recoverable parser errors, runtime checks
- Professional naming: consistent Rust naming conventions and API design
- Extensible architecture: open for future VM and native compilation phases
- Documentation: architecture rationale, tradeoffs, and compiler-concepts explanation
- Unit tests: focused coverage and realistic language examples

## Example Prompts

- "Create a Rust lexer module for Finix with token definitions, comments, and error recovery."
- "Design a Finix parser AST for classes, functions, if/while, lists, dictionaries, and optional type annotations."
- "Implement a tree-walk interpreter for Finix expressions, statements, classes, and import resolution."
- "Explain the tradeoffs between a bytecode VM and a native LLVM compiler for Finix."

## Notes

- Keep all generated components idiomatic to Rust.
- Prefer phased implementation planning and incremental proof-of-concept code.
- Maintain a beginner-friendly developer experience while preserving room for performance evolution.
