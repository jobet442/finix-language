use crate::registry::Module;
use crate::value::Value;

/// Initializes the `math` module for the Finix Standard Library.
pub fn init_module() -> Module {
    let mut module = Module::new("math");
    
    module.register_const("PI", Value::Float(std::f64::consts::PI));
    module.register_const("E", Value::Float(std::f64::consts::E));
    
    module.register_native("sqrt", native_sqrt);
    module.register_native("abs", native_abs);
    
    module
}

fn native_sqrt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("sqrt() takes exactly 1 argument.".to_string());
    }
    match args[0] {
        Value::Float(f) => Ok(Value::Float(f.sqrt())),
        Value::Int(i) => Ok(Value::Float((i as f64).sqrt())),
        _ => Err("sqrt() requires a numeric argument.".to_string()),
    }
}

fn native_abs(_args: &[Value]) -> Result<Value, String> {
    // Implementation for absolute value
    Ok(Value::Null)
}