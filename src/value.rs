/// Represents a deserialized value of the RESP data.
#[derive(Debug)]
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
