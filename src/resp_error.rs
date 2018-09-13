use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RespError {
    /// Data could not be serialized with RESP.
    InvalidData(String),

    /// An operation could not be completed because the reader could no
    /// longer read from the stream.
    UnexpectedEof,

    /// The type of the data that's declared in the first byte is invalid.
    InvalidType,

    /// This typically happens when a CRLF, i.e. \r\n, was expected but
    /// was not present.
    InvalidTerminator,

    /// This typically happens when parsing a RESP Integer or the size of
    /// a RESP Bulk String or Array.
    NotInteger,
}

impl fmt::Display for RespError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RespError::InvalidData(s) => write!(f, "{}", s),
            RespError::UnexpectedEof => write!(f, "Reader could no longer read from stream"),
            RespError::InvalidType => write!(f, "Invalid RESP type"),
            RespError::InvalidTerminator => write!(f, "Does not end with CRLF"),
            RespError::NotInteger => write!(f, "Not an integer"),
        }
    }
}
