use respwriter::RespWriter;

#[derive(Debug, PartialEq)]
pub enum Response {
    KeepAlive(String),
    Close(String),
}

impl Response {
    pub fn ok() -> Self {
        Response::KeepAlive(RespWriter::to_simple_string("OK").unwrap())
    }

    pub fn close_ok() -> Self {
        Response::Close(RespWriter::to_simple_string("OK").unwrap())
    }

    pub fn error(s: &str) -> Self {
        Response::KeepAlive(RespWriter::to_error(s).unwrap())
    }
}
