pub mod visitor;
pub mod printer;
pub mod checker;
pub mod environment;
pub mod error;

pub use checker::TypeChecker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Boolean,
    String,
    Any,
    Void,
    Class(String),
    Function(Vec<Type>, Box<Type>),
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op { Add, Sub, Mul, Div, Mod, Equal, NotEqual, LessThan, LessEqual, GreaterThan, GreaterEqual }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOp { And, Or }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp { Neg, Not }

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal { value: LiteralValue, pos: Position },
    Identifier { name: String, pos: Position },
    Assign { name: String, value: Box<Expr>, pos: Position },
    Binary { left: Box<Expr>, op: Op, right: Box<Expr>, pos: Position },
    Unary { op: UnaryOp, right: Box<Expr>, pos: Position },
    Logical { left: Box<Expr>, op: LogicalOp, right: Box<Expr>, pos: Position },
    Call { callee: Box<Expr>, arguments: Vec<Expr>, pos: Position },
    Get { object: Box<Expr>, name: String, pos: Position },
    Set { object: Box<Expr>, name: String, value: Box<Expr>, pos: Position },
    List { elements: Vec<Expr>, pos: Position },
    Dict { entries: Vec<(Expr, Expr)>, pos: Position },
    This { pos: Position },
    Super { method: String, pos: Position },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<(String, Option<Type>)>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression { expr: Expr, pos: Position },
    Let { name: String, type_ann: Option<Type>, initializer: Option<Expr>, pos: Position },
    Block { statements: Vec<Stmt>, pos: Position },
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>>, pos: Position },
    While { condition: Expr, body: Box<Stmt>, pos: Position },
    For { item: String, iterator: Expr, body: Box<Stmt>, pos: Position },
    Fun { name: String, params: Vec<(String, Option<Type>)>, body: Vec<Stmt>, return_type: Option<Type>, pos: Position },
    Class { name: String, superclass: Option<String>, interfaces: Vec<String>, methods: Vec<Stmt>, pos: Position },
    Interface { name: String, methods: Vec<InterfaceMethod>, pos: Position },
    Return { value: Option<Expr>, pos: Position },
    TryCatch { try_branch: Box<Stmt>, catch_var: String, catch_branch: Box<Stmt>, pos: Position },
    Throw { value: Expr, pos: Position },
    Import { path: Vec<String>, alias: Option<String>, pos: Position },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}