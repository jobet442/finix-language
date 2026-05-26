use crate::visitor::Visitor;
use crate::{Expr, LiteralValue, Op, Program, Stmt, Type};
use crate::error::TypeError;
use crate::environment::TypeEnvironment;
use crate as ast;

pub struct TypeChecker {
    env: TypeEnvironment,
    errors: Vec<TypeError>,
    current_return_type: Option<Type>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
            errors: Vec::new(),
            current_return_type: None,
        }
    }

    pub fn check(mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        self.visit_program(program);
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// The heart of Gradual Typing!
    /// If either side is `Any`, we allow the assignment and defer to runtime.
    fn is_assignable(target: &Type, source: &Type) -> bool {
        if target == &Type::Any || source == &Type::Any {
            return true;
        }
        target == source
    }
}

impl Visitor<Result<Type, TypeError>> for TypeChecker {
    // ==========================================
    // STATEMENT TYPE CHECKING
    // ==========================================

    fn visit_let_stmt(&mut self, name: &str, type_ann: &Option<Type>, initializer: &Option<Expr>, pos: &ast::Position) -> Result<Type, TypeError> {
        let mut inferred_type = Type::Any;

        if let Some(init_expr) = initializer {
            match self.visit_expr(init_expr) {
                Ok(ty) => inferred_type = ty,
                Err(e) => {
                    self.errors.push(e);
                    return Ok(Type::Any);
                }
            }
        }

        let final_type = if let Some(annotated) = type_ann {
            if !Self::is_assignable(annotated, &inferred_type) {
                self.errors.push(TypeError::Mismatch { expected: annotated.clone(), got: inferred_type, pos: *pos });
            }
            annotated.clone()
        } else {
            inferred_type // Type Inference at work!
        };

        self.env.define(name.to_string(), final_type.clone());
        Ok(final_type)
    }

    fn visit_fun_stmt(&mut self, name: &str, params: &[(String, Option<Type>)], body: &[Stmt], return_type: &Option<Type>, _pos: &ast::Position) -> Result<Type, TypeError> {
        let mut param_types = Vec::new();
        for (_, p_type) in params {
            param_types.push(p_type.clone().unwrap_or(Type::Any));
        }

        let ret_type = return_type.clone().unwrap_or(Type::Any);
        let func_type = Type::Function(param_types.clone(), Box::new(ret_type.clone()));
        
        self.env.define(name.to_string(), func_type.clone());

        self.env.begin_scope();
        for ((p_name, _), p_type) in params.iter().zip(param_types) {
            self.env.define(p_name.clone(), p_type);
        }

        let previous_ret = self.current_return_type.clone();
        self.current_return_type = Some(ret_type);

        for stmt in body {
            let _ = self.visit_stmt(stmt);
        }

        self.current_return_type = previous_ret;
        self.env.end_scope();

        Ok(func_type)
    }

    fn visit_return_stmt(&mut self, value: &Option<Expr>, pos: &ast::Position) -> Result<Type, TypeError> {
        let val_type = if let Some(expr) = value {
            self.visit_expr(expr)?
        } else {
            Type::Void
        };

        if let Some(expected_ret) = &self.current_return_type {
            if !Self::is_assignable(expected_ret, &val_type) {
                self.errors.push(TypeError::Mismatch { expected: expected_ret.clone(), got: val_type.clone(), pos: *pos });
            }
        }

        Ok(val_type)
    }

    // ==========================================
    // EXPRESSION TYPE CHECKING
    // ==========================================

    fn visit_literal_expr(&mut self, value: &LiteralValue, _pos: &ast::Position) -> Result<Type, TypeError> {
        match value {
            LiteralValue::Int(_) => Ok(Type::Int),
            LiteralValue::Float(_) => Ok(Type::Float),
            LiteralValue::Boolean(_) => Ok(Type::Boolean),
            LiteralValue::String(_) => Ok(Type::String),
            LiteralValue::Null => Ok(Type::Any), // Null acts as Any
        }
    }

    fn visit_identifier_expr(&mut self, name: &str, pos: &ast::Position) -> Result<Type, TypeError> {
        if let Some(ty) = self.env.get(name) {
            Ok(ty)
        } else {
            let err = TypeError::UndefinedSymbol { name: name.to_string(), pos: *pos };
            self.errors.push(err.clone());
            Err(err)
        }
    }

    fn visit_binary_expr(&mut self, left: &Expr, op: &Op, right: &Expr, pos: &ast::Position) -> Result<Type, TypeError> {
        let left_ty = self.visit_expr(left)?;
        let right_ty = self.visit_expr(right)?;

        // If either is Any, we gradually defer to runtime.
        if left_ty == Type::Any || right_ty == Type::Any {
            return match op {
                Op::Equal | Op::NotEqual | Op::LessThan | Op::GreaterThan => Ok(Type::Boolean),
                _ => Ok(Type::Any),
            };
        }

        match op {
            Op::Add => {
                if left_ty == Type::Int && right_ty == Type::Int {
                    Ok(Type::Int)
                } else if left_ty == Type::Float && right_ty == Type::Float {
                    Ok(Type::Float)
                } else if left_ty == Type::String || right_ty == Type::String {
                    Ok(Type::String)
                } else {
                    let err = TypeError::InvalidOperation { op: *op, left: left_ty, right: right_ty, pos: *pos };
                    self.errors.push(err.clone());
                    Err(err)
                }
            }
            Op::Sub | Op::Mul | Op::Div => {
                if left_ty == Type::Int && right_ty == Type::Int {
                    Ok(Type::Int)
                } else if left_ty == Type::Float && right_ty == Type::Float {
                    Ok(Type::Float)
                } else {
                    let err = TypeError::InvalidOperation { op: *op, left: left_ty, right: right_ty, pos: *pos };
                    self.errors.push(err.clone());
                    Err(err)
                }
            }
            Op::Equal | Op::NotEqual | Op::LessThan | Op::GreaterThan => {
                Ok(Type::Boolean) // Comparison always results in a Boolean
            }
            _ => Ok(Type::Any)
        }
    }
    
    // ... Other overrides stubbed out mapping to their core equivalents 
    fn visit_block_stmt(&mut self, _s: &[Stmt], _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_class_stmt(&mut self, _n: &str, _s: &Option<String>, _i: &[String], _m: &[Stmt], _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_interface_stmt(&mut self, _n: &str, _m: &[ast::InterfaceMethod], _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_if_stmt(&mut self, _c: &Expr, _t: &Stmt, _e: &Option<Box<Stmt>>, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_while_stmt(&mut self, _c: &Expr, _b: &Stmt, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_for_stmt(&mut self, _i: &str, _it: &Expr, _b: &Stmt, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_expression_stmt(&mut self, _e: &Expr, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_try_catch_stmt(&mut self, _t: &Stmt, _v: &str, _c: &Stmt, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_throw_stmt(&mut self, _v: &Expr, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_import_stmt(&mut self, _path: &[String], _a: &Option<String>, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_assign_expr(&mut self, _n: &str, _v: &Expr, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Void) }
    fn visit_unary_expr(&mut self, _op: &ast::UnaryOp, _r: &Expr, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Any) }
    fn visit_logical_expr(&mut self, _l: &Expr, _op: &ast::LogicalOp, _r: &Expr, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Boolean) }
    fn visit_call_expr(&mut self, _c: &Expr, _a: &[Expr], _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Any) }
    fn visit_get_expr(&mut self, _o: &Expr, _n: &str, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Any) }
    fn visit_set_expr(&mut self, _o: &Expr, _n: &str, _v: &Expr, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Any) }
    fn visit_list_expr(&mut self, _e: &[Expr], _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::List(Box::new(Type::Any))) }
    fn visit_dict_expr(&mut self, _e: &[(Expr, Expr)], _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Dict(Box::new(Type::Any), Box::new(Type::Any))) }
    fn visit_this_expr(&mut self, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Any) }
    fn visit_super_expr(&mut self, _m: &str, _p: &ast::Position) -> Result<Type, TypeError> { Ok(Type::Any) }
}