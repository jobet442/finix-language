use crate::Position;
use crate as ast;
use miette::Diagnostic;
use thiserror::Error;

/// Represents a static type error caught before execution.
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum TypeError {
    #[error("Type Mismatch: Expected '{expected:?}', but found '{got:?}'.")]
    #[diagnostic(help("Ensure the assigned value matches the declared type."))]
    Mismatch { expected: ast::Type, got: ast::Type, pos: Position },

    #[error("Undefined symbol '{name}'.")]
    UndefinedSymbol { name: String, pos: Position },

    #[error("Invalid Operation: Operator {op:?} cannot be applied to {left:?} and {right:?}.")]
    InvalidOperation { op: ast::Op, left: ast::Type, right: ast::Type, pos: Position },

    #[error("Arity Mismatch: Expected {expected} arguments, but got {got}.")]
    #[diagnostic(help("Ensure you are passing the correct number of parameters."))]
    ArityMismatch { expected: usize, got: usize, pos: Position },
}