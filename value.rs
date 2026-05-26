use std::fmt;
use crate::gc::ObjHeader;

/// A highly optimized runtime value for the Finix Virtual Machine.
/// In Phase 3, this could be further optimized using NaN tagging!
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Int(i64),
    Float(f64),
    String(String),
    /// A native Rust function bound to the Finix runtime.
    NativeFunction(fn(&[Value]) -> Result<Value, String>),
    /// A managed garbage collected object on the heap.
    Obj(*mut ObjHeader),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::NativeFunction(a), Self::NativeFunction(b)) => *a as *const () == *b as *const (),
            (Self::Obj(a), Self::Obj(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::NativeFunction(_) => write!(f, "<native fn>"),
            Value::Obj(ptr) => {
                let obj = unsafe { &(*(*ptr)).payload };
                write!(f, "{:?}", obj)
            }
        }
    }
}