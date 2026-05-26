use crate::value::Value;
use std::collections::HashMap;

/// Represents complex, dynamically sized runtime objects allocated on the heap.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Instance {
        class_name: String,
        fields: HashMap<String, Value>,
    },
    Closure, // To be expanded with compiled Chunk code and captured Upvalues!
}

/// The memory header wrapping every allocated object.
#[derive(Debug)]
pub struct ObjHeader {
    pub is_marked: bool,
    pub payload: Object,
}

/// The Garbage Collector managing the Finix VM's dynamic memory.
pub struct Heap {
    pub objects: Vec<*mut ObjHeader>,
    pub gray_stack: Vec<*mut ObjHeader>, // Worklist for the tracing phase
    pub bytes_allocated: usize,
    pub next_gc: usize,
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            gray_stack: Vec::new(),
            bytes_allocated: 0,
            next_gc: 1024 * 1024, // Trigger the first GC at 1MB of allocations
        }
    }

    /// Allocates a new object on the heap and returns a raw pointer to it.
    pub fn allocate(&mut self, object: Object) -> *mut ObjHeader {
        let header = Box::new(ObjHeader {
            is_marked: false,
            payload: object,
        });
        
        let ptr = Box::into_raw(header);
        self.objects.push(ptr);
        self.bytes_allocated += std::mem::size_of::<ObjHeader>();
        
        ptr
    }

    /// Marks a value as reachable. If it's a heap object, it is queued for tracing.
    pub fn mark_value(&mut self, value: &Value) {
        if let Value::Obj(ptr) = value {
            unsafe {
                self.mark_object(*ptr);
            }
        }
    }

    /// Marks a specific heap pointer as reachable.
    /// 
    /// # Safety
    /// The caller must ensure that the pointer `ptr` is either null or points to a valid `ObjHeader`.
    pub unsafe fn mark_object(&mut self, ptr: *mut ObjHeader) {
        if ptr.is_null() { return; }
        if (*ptr).is_marked { return; }
        (*ptr).is_marked = true;
        self.gray_stack.push(ptr); // Add to the worklist to trace its children!
    }

    /// Recursively traverses marked objects to mark their nested references.
    pub fn trace_references(&mut self) {
        while let Some(_ptr) = self.gray_stack.pop() {
            // In the future, we match on `(*ptr).payload` here to pull out 
            // nested Lists, Dicts, or Instance fields and call `mark_value` on them!
        }
    }

    /// Reclaims all unmarked memory from the heap.
    pub fn sweep(&mut self) {
        self.objects.retain(|&ptr| {
            unsafe {
                if (*ptr).is_marked {
                    (*ptr).is_marked = false; // Reset for the next GC cycle
                    true // Keep this object
                } else {
                    let _ = Box::from_raw(ptr); // Drop and free the memory!
                    false // Remove from tracking list
                }
            }
        });
    }
}