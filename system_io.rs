use crate::registry::Module;
use std::io::Write;
use crate::value::Value;

/// Initializes the `system.io` module for the Finix Standard Library.
pub fn init_module() -> Module {
    let mut module = Module::new("system.io");
    
    module.register_native("print", native_print);
    module.register_native("println", native_println);
    module.register_native("read_line", native_read_line);
    
    module
}

fn native_print(args: &[Value]) -> Result<Value, String> {
    for arg in args {
        print!("{}", arg);
    }
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

fn native_println(args: &[Value]) -> Result<Value, String> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    Ok(Value::Null)
}

fn native_read_line(_args: &[Value]) -> Result<Value, String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    
    let trimmed = input.trim_end().to_string();
    Ok(Value::String(trimmed))
}