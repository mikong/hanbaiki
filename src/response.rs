use respwriter::RespWriter;

#[derive(Debug, PartialEq)]
pub enum Response {
    KeepAlive(String),
    Close(String),
}

impl Response {
    pub fn build_ok() -> Self {
        Response::KeepAlive(RespWriter::to_simple_string("OK").unwrap())
    }

    pub fn build_close_ok() -> Self {
        Response::Close(RespWriter::to_simple_string("OK").unwrap())
    }

    pub fn build_error(s: &str) -> Self {
        Response::KeepAlive(RespWriter::to_error(s).unwrap())
    }
}
