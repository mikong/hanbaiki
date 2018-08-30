#[derive(Debug, PartialEq)]
pub enum RespError {
    InvalidData(String),
}

pub struct RespWriter;

impl RespWriter {

    pub fn to_simple_string(s: &str) -> Result<String, RespError> {
        RespWriter::to_simple_message("+", s)
    }

    pub fn to_error(s: &str) -> Result<String, RespError> {
        RespWriter::to_simple_message("-", s)
    }

    fn to_simple_message(prefix: &str, s: &str) -> Result<String, RespError> {

        if s.contains("\r") || s.contains("\n") {
            return Err(RespError::InvalidData("Contains CR or LF".to_string()));
        }

        let msg = format!("{}{}\r\n", prefix, s);
        Ok(msg)
    }

    pub fn to_integer(i: usize) -> String {
        format!(":{}\r\n", i)
    }

    pub fn to_bulk_string(s: &str) -> String {
        format!("${}\r\n{}\r\n", s.len(), s)
    }

    pub fn null_bulk_string() -> String {
        "$-1\r\n".to_string()
    }

    pub fn to_array(strings: &[&str]) -> String {

        let mut msg = format!("*{}\r\n", strings.len());

        for s in strings {
            msg.push_str(&RespWriter::to_bulk_string(s));
        }
        msg
    }
}

#[cfg(test)]
mod test {

    use super::RespWriter;

    #[test]
    fn check_simple_string() {
        assert_eq!("+OK\r\n", RespWriter::to_simple_string("OK").unwrap());
        assert!(RespWriter::to_simple_string("PING\r\n...").is_err());
    }

    #[test]
    fn check_error() {
        assert_eq!("-ERR\r\n", RespWriter::to_error("ERR").unwrap());
        assert!(RespWriter::to_error("PONG\r\n...").is_err());
    }

    #[test]
    fn check_integer() {
        assert_eq!(":100\r\n", RespWriter::to_integer(100));
    }

    #[test]
    fn check_bulk_string() {
        assert_eq!("$0\r\n\r\n", RespWriter::to_bulk_string(""));
        assert_eq!("$6\r\nfoobar\r\n", RespWriter::to_bulk_string("foobar"));
        assert_eq!("$2\r\n\r\n\r\n", RespWriter::to_bulk_string("\r\n"));
        assert_eq!("$4\r\n😍\r\n", RespWriter::to_bulk_string("😍"));
    }

    #[test]
    fn check_null_bulk_string() {
        assert_eq!("$-1\r\n", RespWriter::null_bulk_string());
    }

    #[test]
    fn check_array() {
        let v = vec!["foo", "bar"];
        assert_eq!("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n", RespWriter::to_array(&v));
    }

}
