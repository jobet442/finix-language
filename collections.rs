//! Implementations for list and dictionary semantics.

use std::collections::HashMap;
use std::hash::Hash;

/// Represents a dynamic list in Finix, inspired by Python's `list` and Java's `ArrayList`.
/// Made generic so the interpreter can store dynamic `Value`s, while later native
/// compilations can monomorphize it for specific unboxed types.
#[derive(Debug, Clone, PartialEq)]
pub struct FinixList<T> {
    elements: Vec<T>,
}

impl<T> FinixList<T> {
    /// Creates a new, empty FinixList.
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    /// Constructs a FinixList from an existing Rust Vector.
    pub fn from_vec(elements: Vec<T>) -> Self {
        Self { elements }
    }

    /// Appends an item to the end of the list.
    pub fn append(&mut self, item: T) {
        self.elements.push(item);
    }

    /// Removes and returns the last item from the list.
    pub fn pop(&mut self) -> Option<T> {
        self.elements.pop()
    }

    /// Retrieves a reference to an item at a specific index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.elements.get(index)
    }

    /// Sets an item at a specific index. Returns an Error if the index is out of bounds.
    pub fn set(&mut self, index: usize, item: T) -> Result<(), String> {
        if index < self.elements.len() {
            self.elements[index] = item;
            Ok(())
        } else {
            Err(format!("Index out of bounds: {}", index))
        }
    }

    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the list has no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T> Default for FinixList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a key-value dictionary in Finix, inspired by Python's `dict` and Java's `HashMap`.
#[derive(Debug, Clone, PartialEq)]
pub struct FinixDict<K, V>
where
    K: Eq + Hash,
{
    entries: HashMap<K, V>,
}

impl<K, V> FinixDict<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new, empty FinixDict.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Inserts a key-value pair into the dictionary. 
    /// If the key existed, the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.entries.insert(key, value)
    }

    /// Retrieves a reference to the value associated with the given key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)
    }

    /// Removes a key from the dictionary, returning the value if the key existed.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key)
    }

    /// Returns `true` if the dictionary contains the given key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    /// Returns the number of key-value pairs in the dictionary.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<K, V> Default for FinixDict<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}