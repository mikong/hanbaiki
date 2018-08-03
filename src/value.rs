/// Represents a deserialized value of the RESP data.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// Not a valid RESP data.
    /// This is only used internally to
    /// represent an uninitialized Value.
    Null,

    /// Represents a RESP Simple String.
    SimpleString(String),

    /// Represents a RESP Error.
    Error(String),

    /// Represents a RESP Integer.
    Integer(i64),

    /// Represents a RESP Bulk String.
    BulkString(String),

    /// Represents a RESP Array.
    Array(Vec<Value>),
}

use std::mem;

impl Value {
    pub fn to_string(self) -> String {
        match self {
            Value::SimpleString(s) => s,
            Value::Error(s) => s,
            Value::Integer(i) => i.to_string(),
            Value::BulkString(s) => s,
            _ => panic!("Unexpected Value type"),
        }
    }

    pub fn take(&mut self) -> Value {
        let mut v = Value::Null;
        mem::swap(self, &mut v);
        v
    }
}
