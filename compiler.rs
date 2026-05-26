#![cfg(feature = "llvm")]

use ast::{Expr, LiteralValue, Op, Program, Stmt, Type};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;

/// The LLVM IR Code Generator for Finix.
pub struct CodeGen<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: &'a Module<'ctx>,
    pub builder: &'a Builder<'ctx>,
    
    /// Tracks local variables allocated on the stack (via alloca).
    pub variables: HashMap<String, PointerValue<'ctx>>,
    
    /// The function currently being compiled.
    pub current_fn: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>, builder: &'a Builder<'ctx>) -> Self {
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            current_fn: None,
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Fun { name, params, body, return_type, .. } => {
                self.compile_function(name, params, body, return_type)
            }
            Stmt::Let { name, type_ann, initializer, .. } => {
                self.compile_let(name, type_ann, initializer)
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    let val = self.compile_expr(expr)?;
                    self.builder.build_return(Some(&val));
                } else {
                    self.builder.build_return(None);
                }
                Ok(())
            }
            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr)?;
                Ok(())
            }
            _ => Err("Statement not yet implemented in LLVM backend".to_string()),
        }
    }

    /// Compiles a Finix function into an LLVM Function.
    fn compile_function(&mut self, name: &str, params: &[(String, Option<Type>)], body: &[Stmt], return_type: &Option<Type>) -> Result<(), String> {
        // 1. Determine argument types
        let mut param_types = Vec::new();
        for (_, ty) in params {
            let basic_type = self.get_llvm_type(ty.as_ref().unwrap_or(&Type::Any))?;
            param_types.push(basic_type.into());
        }

        // 2. Determine return type
        let fn_type = match return_type.as_ref().unwrap_or(&Type::Void) {
            Type::Int => self.context.i64_type().fn_type(&param_types, false),
            Type::Float => self.context.f64_type().fn_type(&param_types, false),
            Type::Boolean => self.context.bool_type().fn_type(&param_types, false),
            Type::Void => self.context.void_type().fn_type(&param_types, false),
            _ => return Err("Unsupported return type".to_string()),
        };

        // 3. Create the LLVM function
        let function = self.module.add_function(name, fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        self.current_fn = Some(function);
        self.variables.clear(); // Clear scope for new function

        // 4. Allocate memory for parameters to support mutability (SSA limitation)
        for (i, arg) in function.get_param_iter().enumerate() {
            let param_name = &params[i].0;
            let alloca = self.create_entry_block_alloca(param_name, arg.get_type());
            self.builder.build_store(alloca, arg);
            self.variables.insert(param_name.clone(), alloca);
        }

        // 5. Compile the body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        Ok(())
    }

    fn compile_let(&mut self, name: &str, _type_ann: &Option<Type>, initializer: &Option<Expr>) -> Result<(), String> {
        let init_val = if let Some(expr) = initializer {
            self.compile_expr(expr)?
        } else {
            self.context.i64_type().const_zero().into() // Default MVP to 0
        };

        let alloca = self.create_entry_block_alloca(name, init_val.get_type());
        self.builder.build_store(alloca, init_val);
        self.variables.insert(name.to_string(), alloca);
        
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::Literal { value, .. } => match value {
                LiteralValue::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
                LiteralValue::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
                LiteralValue::Boolean(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
                _ => Err("Unsupported literal type".to_string()),
            },
            Expr::Identifier { name, .. } => {
                match self.variables.get(name) {
                    Some(ptr) => {
                        // Load the value from the allocated stack pointer
                        let loaded = self.builder.build_load(self.context.i64_type(), *ptr, name);
                        Ok(loaded)
                    }
                    None => Err(format!("Undefined variable '{}' in LLVM IR", name)),
                }
            }
            Expr::Binary { left, op, right, .. } => {
                let lhs = self.compile_expr(left)?;
                let rhs = self.compile_expr(right)?;

                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        match op {
                            Op::Add => Ok(self.builder.build_int_add(l, r, "addtmp").into()),
                            Op::Sub => Ok(self.builder.build_int_sub(l, r, "subtmp").into()),
                            Op::Mul => Ok(self.builder.build_int_mul(l, r, "multmp").into()),
                            Op::Div => Ok(self.builder.build_int_signed_div(l, r, "divtmp").into()),
                            _ => Err("Binary op not supported for Ints".to_string())
                        }
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        match op {
                            Op::Add => Ok(self.builder.build_float_add(l, r, "addtmp").into()),
                            _ => Err("Binary op not supported for Floats".to_string())
                        }
                    }
                    _ => Err("Type mismatch in binary expression".to_string()),
                }
            }
            _ => Err("Expression not implemented in LLVM backend".to_string()),
        }
    }

    // ==========================================
    // UTILITIES
    // ==========================================

    /// Creates an `alloca` instruction at the beginning of the current function.
    /// This is crucial for allocating variables on the stack for mutable (SSA) values.
    fn create_entry_block_alloca(&self, name: &str, ty: BasicTypeEnum<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self.current_fn.unwrap().get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        builder.build_alloca(ty, name)
    }

    fn get_llvm_type(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(self.context.i64_type().into()),
            Type::Float => Ok(self.context.f64_type().into()),
            Type::Boolean => Ok(self.context.bool_type().into()),
            _ => Err(format!("Unsupported type for LLVM codegen: {:?}", ty)),
        }
    }
}