use std::io::Read;
use std::str;
use value::Value;

#[derive(Debug)]
pub struct RespReader {
    pub message: Vec<u8>,
    index: usize,
    stack: Vec<State>,
    value: Value,
}

impl RespReader {
    pub fn new() -> Self {
        RespReader {
            message: vec![],
            index: 0,
            stack: vec![],
            value: Value::Null,
        }
    }

    pub fn frame_message<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {

        self.stack.push(State::GetType);
        self.read(stream)?;

        loop {
            let get_fn = match self.current_state() {
                Some(&State::GetType) => Self::get_type,
                Some(&State::GetSimpleString(_, _)) => Self::get_simple_string,
                Some(&State::GetError(_, _)) => Self::get_error,
                Some(&State::GetInteger(_)) => Self::get_integer,
                Some(&State::GetBulkString(_, _)) => Self::get_bulk_string,
                Some(&State::GetArray(_, _)) => Self::get_array,
                None => return Ok(()),
            };

            match get_fn(self)? {
                Some(_) => {
                    if self.stack.is_empty() {
                        return Ok(());
                    }
                },
                None => self.read(stream)?,
            }
        }
    }

    fn current_state(&self) -> Option<&State> {
        self.stack.last()
    }

    fn transition_to(&mut self, state: State) {
        self.stack.pop();
        self.stack.push(state);
    }

    fn set_value(&mut self, value: Value) {
        if self.value == Value::Null {
            self.value = value;
        }
        // TODO: handle arrays and nested arrays
    }

    fn read<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {
        let mut buf = vec![0; 20];
        let length = stream.read(&mut buf).unwrap();

        if length == 0 {
            return Err("EOF before end of frame".to_string());
        }

        for byte in buf[0..length].iter() {
            self.message.push(*byte)
        }

        Ok(())
    }

    fn get_type(&mut self) -> Result<Option<()>, String> {
        let i = self.index + 1;
        match self.message.get(self.index) {
            Some(&b'+') =>
                self.transition_to(State::GetSimpleString(SubState::CheckCR, i)),
            Some(&b'-') =>
                self.transition_to(State::GetError(SubState::CheckCR, i)),
            Some(&b':') =>
                self.transition_to(State::GetInteger(SubState::CheckCR)),
            Some(&b'$') =>
                self.transition_to(State::GetBulkString(SubState::GetSize, 0)),
            Some(&b'*') =>
                self.transition_to(State::GetArray(SubState::GetSize, 0)),
            _ => return Err("Invalid RESP type".to_string()),
        }

        self.index += 1;
        Ok(Some(()))
    }

    fn get_simple_string(&mut self) -> Result<Option<()>, String> {
        let (mut substate, i0) = match self.current_state() {
            Some(&State::GetSimpleString(ss, i)) => (ss, i),
            _ => panic!("Invalid state in get_simple_string"),
        };

        // TODO: return Err("LF before CR".to_string());
        if substate == SubState::CheckCR {
            let start_index = self.index;
            if let Some(i) = self.find_break(start_index) {
                let v = String::from_utf8(self.message[i0..i].to_vec()).unwrap();
                self.set_value(Value::SimpleString(v));
                self.index = i + 1;
                substate = SubState::CheckLF;
                self.transition_to(State::GetSimpleString(substate, i0));
            } else {
                self.index = self.message.len();
            }
        }

        if substate == SubState::CheckLF {
            if self.check_lf()?.is_some() {
                self.stack.pop();
                return Ok(Some(()));
            }
        }

        Ok(None)
    }

    fn get_error(&mut self) -> Result<Option<()>, String> {
        let (mut substate, i0) = match self.current_state() {
            Some(&State::GetError(ss, i)) => (ss, i),
            _ => panic!("Invalid state in get_error"),
        };

        // TODO: return Err("LF before CR".to_string());
        if substate == SubState::CheckCR {
            let start_index = self.index;
            if let Some(i) = self.find_break(start_index) {
                let v = String::from_utf8(self.message[i0..i].to_vec()).unwrap();
                self.set_value(Value::Error(v));
                self.index = i + 1;
                substate = SubState::CheckLF;
                self.transition_to(State::GetError(substate, i0));
            } else {
                self.index = self.message.len();
            }
        }

        if substate == SubState::CheckLF {
            if self.check_lf()?.is_some() {
                self.stack.pop();
                return Ok(Some(()));
            }
        }

        Ok(None)
    }

    fn get_integer(&mut self) -> Result<Option<()>, String> {
        let mut substate = match self.current_state() {
            Some(&State::GetInteger(s)) => s,
            _ => panic!("Invalid state in get_integer"),
        };

        if substate == SubState::CheckCR {
            let start_index = self.index;
            if let Some(i) = self.find_break(start_index) {
                match self.parse_int(start_index, i) {
                    Some(v) => {
                        self.set_value(Value::Integer(v));
                        self.index = i + 1;
                        substate = SubState::CheckLF;
                        self.transition_to(State::GetInteger(substate));
                    },
                    None => return Err("Not an integer".to_string()),
                }
            }
        }

        if substate == SubState::CheckLF {
            if self.check_lf()?.is_some() {
                self.stack.pop();
                return Ok(Some(()));
            }
        }

        Ok(None)
    }

    fn get_bulk_string(&mut self) -> Result<Option<()>, String> {
        let (mut substate, mut size) = match self.current_state() {
            Some(&State::GetBulkString(ss, sz)) => (ss, sz),
            _ => panic!("Invalid state in get_bulk_string"),
        };

        if substate == SubState::GetSize {
            let start_index = self.index;
            if let Some(n) = self.get_size(start_index)? {
                size = n;
                substate = SubState::CheckLF;
                self.transition_to(State::GetBulkString(substate, size));
            }
        }

        if substate == SubState::CheckLF {
            if self.check_lf()?.is_some() {
                substate = SubState::BuildString;
                self.transition_to(State::GetBulkString(substate, size));
            }
        }

        if substate == SubState::BuildString {
            if self.build_string(size as usize)?.is_some() {
                self.stack.pop();
                return Ok(Some(()));
            }
        }

        Ok(None)
    }

    fn get_array(&mut self) -> Result<Option<()>, String> {
        let (mut substate, mut size) = match self.current_state() {
            Some(&State::GetArray(ss, sz)) => (ss, sz),
            _ => panic!("Invalid state in get_array"),
        };

        if substate == SubState::GetSize {
            let start_index = self.index;
            if let Some(n) = self.get_size(start_index)? {
                size = n;
                substate = SubState::CheckLF;
                self.transition_to(State::GetArray(substate, size));
            }
        }

        if substate == SubState::CheckLF {
            if self.check_lf()?.is_some() {
                substate = SubState::GetElements;
                self.transition_to(State::GetArray(substate, size));
                self.set_value(Value::Array(vec![]));
            }
        }

        if substate == SubState::GetElements {
            if size > 0 {
                size -= 1;
                substate = SubState::GetElements;
                self.transition_to(State::GetArray(substate, size));
                self.stack.push(State::GetType);
            } else {
                self.stack.pop();
            }
            return Ok(Some(()));
        }

        Ok(None)
    }

    fn get_size(&mut self, start_index: usize) -> Result<Option<usize>, String> {
        let mut size = None;

        if let Some(i) = self.find_break(start_index) {
            match self.parse_int(start_index, i) {
                Some(n) => {
                    size = Some(n as usize);
                    self.index = i + 1;
                },
                None => return Err("Not an integer".to_string()),
            }
        }

        Ok(size)
    }

    fn check_lf(&mut self) -> Result<Option<()>, String> {
        if let Some(&byte) = self.message.get(self.index) {
            if byte == b'\n' {
                self.index += 1;
                return Ok(Some(()));
            } else {
                return Err("CR not followed by LF".to_string());
            }
        }

        Ok(None)
    }

    fn build_string(&mut self, size: usize) -> Result<Option<()>, String> {
        if self.message.len() > self.index + size + 1 {
            let start = self.index;
            self.index += size;
            if self.message[self.index] == b'\r' && self.message[self.index + 1] == b'\n' {
                let end = self.index;
                let v = String::from_utf8(self.message[start..end].to_vec()).unwrap();
                self.set_value(Value::BulkString(v));
                self.index += 2;
                return Ok(Some(()));
            } else {
                return Err("Does not end with CRLF".to_string());
            }
        }

        Ok(None)
    }

    fn find_break(&self, start_index: usize) -> Option<usize> {
        if let Some(slice) = self.message.get(start_index..) {
            for (i, byte) in slice.iter().enumerate() {
                if *byte == b'\r' {
                    return Some(start_index + i);
                }
            }
        }
        None
    }

    fn parse_int(&self, start_index: usize, end_index: usize) -> Option<i64> {
        match str::from_utf8(&self.message[start_index..end_index]) {
            Ok(s) => s.parse::<i64>().ok(),
            Err(_) => None,
        }
    }

}

#[derive(Debug)]
enum State {
    GetType,
    GetSimpleString(SubState, usize),
    GetError(SubState, usize),
    GetInteger(SubState),
    GetBulkString(SubState, usize),
    GetArray(SubState, usize),
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum SubState {
    CheckCR,
    CheckLF,
    // bulk string:
    GetSize,
    BuildString,
    // array:
    GetElements,
}

#[cfg(test)]
mod test {

    use super::RespReader;
    use super::Value;
    use std::io;
    use std::io::Read;

    struct MockStream {
        message: Vec<u8>,
        pos: usize,
    }

    impl MockStream {
        fn from(s: &str) -> Self {
            MockStream {
                message: s.as_bytes().to_vec(),
                pos: 0,
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

            let len = self.message.len();
            let end_pos = if self.pos + buf.len() > len {
                len
            } else {
                buf.len()
            };

            for (i, j) in (self.pos..end_pos).enumerate() {
                buf[i] = self.message[j];
            }

            for i in end_pos..buf.len() {
                buf[i] = 0;
            }

            let size = end_pos - self.pos;
            self.pos += size;
            Ok(size)
        }
    }

    #[test]
    fn check_mock_stream() {
        let mut stream = MockStream::from("$12\r\nHello World!\r\n");
        let mut buf = vec![0; 10];
        let len = stream.read(&mut buf).unwrap();
        assert_eq!(10, len);
        assert_eq!(&buf[..], b"$12\r\nHello");
    }

    fn check_valid(s: &str) {
        let mut reader = RespReader::new();
        let mut stream = MockStream::from(s);

        reader.frame_message(&mut stream).unwrap();

        assert_eq!(s.to_string(), String::from_utf8(reader.message).unwrap());
    }

    fn check_invalid(s: &str, e: &str) {
        let mut reader = RespReader::new();
        let mut stream = MockStream::from(s);

        let result = reader.frame_message(&mut stream);

        assert_eq!(result, Err(e.to_string()));
    }

    #[test]
    fn check_simple_string() {
        let empty = "+\r\n";
        check_valid(empty);

        let simple = "+OK\r\n";
        check_valid(simple);

        // with reader buf size of 20, \n is on the next read
        let split_crlf = "+123456789012345678\r\n";
        check_valid(split_crlf);

        let broken_crlf = "+123\r4\n";
        let err = "CR not followed by LF";
        check_invalid(broken_crlf, err);
    }

    #[test]
    fn check_error() {
        let error_message = "-ERROR: Key not found\r\n";
        check_valid(error_message);
    }

    #[test]
    fn check_integer() {
        let integer = ":100\r\n";
        check_valid(integer);
    }

    #[test]
    fn check_bulk_string() {
        let simple = "$12\r\nHello World!\r\n";
        check_valid(simple);

        let no_size = "$\r\nHi\r\n";
        let err = "Not an integer";
        check_invalid(no_size, err);

        let s = "$12\r\nHello World!\r\r";
        let err = "Does not end with CRLF";
        check_invalid(s, err);
    }

    #[test]
    fn check_array() {
        let empty = "*0\r\n";
        check_valid(empty);

        let simple = "*1\r\n$12\r\nHello World!\r\n";
        check_valid(simple);

        let mixed = "*3\r\n$12\r\nHello World!\r\n+OK\r\n:25\r\n";
        check_valid(mixed);
    }

    fn get_value(s: &str) -> Value {
        let mut reader = RespReader::new();
        let mut stream = MockStream::from(s);

        reader.frame_message(&mut stream).unwrap();
        reader.value
    }

    #[test]
    fn check_simple_string_val() {
        let empty = "+\r\n";
        let mut v = get_value(empty);
        assert_eq!(v, Value::SimpleString("".to_string()));

        let simple = "+OK\r\n";
        v = get_value(simple);
        assert_eq!(v, Value::SimpleString("OK".to_string()));
    }

    #[test]
    fn check_error_val() {
        let error_message = "-ERROR: Key not found\r\n";
        let v = get_value(error_message);
        assert_eq!(v, Value::Error("ERROR: Key not found".to_string()));
    }

    #[test]
    fn check_integer_val() {
        let integer = ":100\r\n";
        let v = get_value(integer);
        assert_eq!(v, Value::Integer(100));
    }

    #[test]
    fn check_bulk_string_val() {
        let simple = "$12\r\nHello World!\r\n";
        let v = get_value(simple);
        assert_eq!(v, Value::BulkString("Hello World!".to_string()));
    }

    #[test]
    fn check_array_val() {
        let empty = "*0\r\n";
        let v = get_value(empty);
        assert_eq!(v, Value::Array(vec![]));
    }
}
