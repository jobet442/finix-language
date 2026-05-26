use crate::{Expr, LiteralValue, LogicalOp, Op, Program, Stmt, UnaryOp};

/// A utility structure for converting a Finix AST into an easy-to-read string format.
#[derive(Default)]
pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print_program(&self, program: &Program) -> String {
        let mut out = String::new();
        for stmt in &program.statements {
            out.push_str(&self.print_stmt(stmt));
            out.push('\n');
        }
        out
    }

    pub fn print_stmt(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Expression { expr, .. } => format!("(expr {})", self.print_expr(expr)),
            Stmt::Let { name, type_ann, initializer, .. } => {
                let type_str = if let Some(t) = type_ann { format!(":{:?}", t) } else { "".to_string() };
                let init_str = if let Some(i) = initializer { format!(" = {}", self.print_expr(i)) } else { "".to_string() };
                format!("(let {}{} {})", name, type_str, init_str)
            }
            Stmt::Block { statements, .. } => {
                let inner: Vec<String> = statements.iter().map(|s| self.print_stmt(s)).collect();
                format!("(block {})", inner.join(" "))
            }
            Stmt::If { condition, then_branch, else_branch, .. } => {
                let else_str = if let Some(e) = else_branch { format!(" (else {})", self.print_stmt(e)) } else { "".to_string() };
                format!("(if {} {}{})", self.print_expr(condition), self.print_stmt(then_branch), else_str)
            }
            Stmt::While { condition, body, .. } => format!("(while {} {})", self.print_expr(condition), self.print_stmt(body)),
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    format!("(return {})", self.print_expr(v))
                } else {
                    "(return)".to_string()
                }
            }
            // More formatting omitted for brevity...
            _ => "(unimplemented stmt)".to_string(),
        }
    }

    pub fn print_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal { value, .. } => match value {
                LiteralValue::Int(n) => n.to_string(),
                LiteralValue::Float(n) => n.to_string(),
                LiteralValue::Boolean(b) => b.to_string(),
                LiteralValue::String(s) => format!("\"{}\"", s),
                LiteralValue::Null => "null".to_string(),
            },
            Expr::Identifier { name, .. } => name.clone(),
            Expr::Assign { name, value, .. } => format!("(= {} {})", name, self.print_expr(value)),
            Expr::Binary { left, op, right, .. } => self.parenthesize(self.op_to_str(op), &[left, right]),
            Expr::Unary { op, right, .. } => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                self.parenthesize(op_str, &[right])
            }
            Expr::Logical { left, op, right, .. } => {
                let op_str = match op {
                    LogicalOp::And => "&&",
                    LogicalOp::Or => "||",
                };
                self.parenthesize(op_str, &[left, right])
            }
            Expr::Call { callee, arguments, .. } => {
                let args: Vec<String> = arguments.iter().map(|arg| self.print_expr(arg)).collect();
                format!("(call {} ({}))", self.print_expr(callee), args.join(" "))
            }
            _ => "(unimplemented expr)".to_string(),
        }
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let mut out = format!("({}", name);
        for expr in exprs {
            out.push(' ');
            out.push_str(&self.print_expr(expr));
        }
        out.push(')');
        out
    }

    fn op_to_str(&self, op: &Op) -> &'static str {
        match op {
            Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/", Op::Mod => "%",
            Op::Equal => "==", Op::NotEqual => "!=", Op::LessThan => "<", Op::LessEqual => "<=",
            Op::GreaterThan => ">", Op::GreaterEqual => ">=",
        }
    }
}