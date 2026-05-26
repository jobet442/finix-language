use crate::{Expr, Program, Stmt};

/// The standard Visitor trait for Finix AST traversal.
/// Implementors specify the return type `R` (e.g., `Value` for interpreters, `()` for type checkers).
pub trait Visitor<R> {
    /// Entry point for visiting a whole program.
    fn visit_program(&mut self, program: &Program) -> Vec<R> {
        program
            .statements
            .iter()
            .map(|stmt| self.visit_stmt(stmt))
            .collect()
    }

    /// Routes a statement to its specific handler.
    fn visit_stmt(&mut self, stmt: &Stmt) -> R {
        match stmt {
            Stmt::Block { statements, pos } => self.visit_block_stmt(statements, pos),
            Stmt::Let { name, type_ann, initializer, pos } => self.visit_let_stmt(name, type_ann, initializer, pos),
            Stmt::Class { name, superclass, interfaces, methods, pos } => self.visit_class_stmt(name, superclass, interfaces, methods, pos),
            Stmt::Interface { name, methods, pos } => self.visit_interface_stmt(name, methods, pos),
            Stmt::Fun { name, params, body, return_type, pos } => self.visit_fun_stmt(name, params, body, return_type, pos),
            Stmt::If { condition, then_branch, else_branch, pos } => self.visit_if_stmt(condition, then_branch, else_branch, pos),
            Stmt::While { condition, body, pos } => self.visit_while_stmt(condition, body, pos),
            Stmt::For { item, iterator, body, pos } => self.visit_for_stmt(item, iterator, body, pos),
            Stmt::Return { value, pos } => self.visit_return_stmt(value, pos),
            Stmt::Expression { expr, pos } => self.visit_expression_stmt(expr, pos),
            Stmt::TryCatch { try_branch, catch_var, catch_branch, pos } => self.visit_try_catch_stmt(try_branch, catch_var, catch_branch, pos),
            Stmt::Throw { value, pos } => self.visit_throw_stmt(value, pos),
            Stmt::Import { path, alias, pos } => self.visit_import_stmt(path, alias, pos),
        }
    }

    /// Routes an expression to its specific handler.
    fn visit_expr(&mut self, expr: &Expr) -> R {
        match expr {
            Expr::Literal { value, pos } => self.visit_literal_expr(value, pos),
            Expr::Identifier { name, pos } => self.visit_identifier_expr(name, pos),
            Expr::Assign { name, value, pos } => self.visit_assign_expr(name, value, pos),
            Expr::Binary { left, op, right, pos } => self.visit_binary_expr(left, op, right, pos),
            Expr::Unary { op, right, pos } => self.visit_unary_expr(op, right, pos),
            Expr::Logical { left, op, right, pos } => self.visit_logical_expr(left, op, right, pos),
            Expr::Call { callee, arguments, pos } => self.visit_call_expr(callee, arguments, pos),
            Expr::Get { object, name, pos } => self.visit_get_expr(object, name, pos),
            Expr::Set { object, name, value, pos } => self.visit_set_expr(object, name, value, pos),
            Expr::List { elements, pos } => self.visit_list_expr(elements, pos),
            Expr::Dict { entries, pos } => self.visit_dict_expr(entries, pos),
            Expr::This { pos } => self.visit_this_expr(pos),
            Expr::Super { method, pos } => self.visit_super_expr(method, pos),
        }
    }

    // Statement Handlers
    fn visit_block_stmt(&mut self, statements: &[Stmt], pos: &crate::Position) -> R;
    fn visit_let_stmt(&mut self, name: &str, type_ann: &Option<crate::Type>, initializer: &Option<Expr>, pos: &crate::Position) -> R;
    fn visit_class_stmt(&mut self, name: &str, superclass: &Option<String>, interfaces: &[String], methods: &[Stmt], pos: &crate::Position) -> R;
    fn visit_interface_stmt(&mut self, name: &str, methods: &[crate::InterfaceMethod], pos: &crate::Position) -> R;
    fn visit_fun_stmt(&mut self, name: &str, params: &[(String, Option<crate::Type>)], body: &[Stmt], return_type: &Option<crate::Type>, pos: &crate::Position) -> R;
    fn visit_if_stmt(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>, pos: &crate::Position) -> R;
    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt, pos: &crate::Position) -> R;
    fn visit_for_stmt(&mut self, item: &str, iterator: &Expr, body: &Stmt, pos: &crate::Position) -> R;
    fn visit_return_stmt(&mut self, value: &Option<Expr>, pos: &crate::Position) -> R;
    fn visit_expression_stmt(&mut self, expr: &Expr, pos: &crate::Position) -> R;
    fn visit_try_catch_stmt(&mut self, try_branch: &Stmt, catch_var: &str, catch_branch: &Stmt, pos: &crate::Position) -> R;
    fn visit_throw_stmt(&mut self, value: &Expr, pos: &crate::Position) -> R;
    fn visit_import_stmt(&mut self, path: &[String], alias: &Option<String>, pos: &crate::Position) -> R;

    // Expression Handlers
    fn visit_literal_expr(&mut self, value: &crate::LiteralValue, pos: &crate::Position) -> R;
    fn visit_identifier_expr(&mut self, name: &str, pos: &crate::Position) -> R;
    fn visit_assign_expr(&mut self, name: &str, value: &Expr, pos: &crate::Position) -> R;
    fn visit_binary_expr(&mut self, left: &Expr, op: &crate::Op, right: &Expr, pos: &crate::Position) -> R;
    fn visit_unary_expr(&mut self, op: &crate::UnaryOp, right: &Expr, pos: &crate::Position) -> R;
    fn visit_logical_expr(&mut self, left: &Expr, op: &crate::LogicalOp, right: &Expr, pos: &crate::Position) -> R;
    fn visit_call_expr(&mut self, callee: &Expr, arguments: &[Expr], pos: &crate::Position) -> R;
    fn visit_get_expr(&mut self, object: &Expr, name: &str, pos: &crate::Position) -> R;
    fn visit_set_expr(&mut self, object: &Expr, name: &str, value: &Expr, pos: &crate::Position) -> R;
    fn visit_list_expr(&mut self, elements: &[Expr], pos: &crate::Position) -> R;
    fn visit_dict_expr(&mut self, entries: &[(Expr, Expr)], pos: &crate::Position) -> R;
    fn visit_this_expr(&mut self, pos: &crate::Position) -> R;
    fn visit_super_expr(&mut self, method: &str, pos: &crate::Position) -> R;
}