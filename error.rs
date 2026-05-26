use ast::Position;
use miette::Diagnostic;
use thiserror::Error;

/// Represents an execution error in the Finix interpreter.
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum RuntimeError {
    #[error("Undefined variable '{name}'.")]
    #[diagnostic(help("Ensure the variable is declared using 'let' before using it."))]
    UndefinedVariable { name: String, pos: Position },

    #[error("Type Error: {msg}")]
    TypeError { msg: String, pos: Position },

    #[error("Arity Error: Expected {expected} arguments but got {got}.")]
    ArityError { expected: usize, got: usize, pos: Position },

    #[error("Not callable: Can only call functions and classes.")]
    NotCallable { pos: Position },
}

/// Internal wrapper for the interpreter to handle control flow.
/// A `return` statement isn't a crash, but it needs to bubble up the call stack!
#[derive(Debug, Clone)]
pub enum EvalError {
    Error(RuntimeError),
    Return(crate::value::Value),
}