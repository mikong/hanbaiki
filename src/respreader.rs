use std::io::Read;
use std::str;
use std::mem;
use std::result;
use value::Value;

use resp_error::RespError;

type Result<T> = result::Result<T, RespError>;

#[derive(Debug)]
pub struct RespReader {
    pub message: Vec<u8>,
    index: usize,
    stack: Vec<State>,
    pub value: Value,
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

    pub fn frame_message<T: Read>(&mut self, stream: &mut T) -> Result<()> {

        self.stack.push(State::GetType);
        self.read(stream)?;

        loop {
            let get_fn = match self.current_state() {
                Some(State::GetType) => Self::get_type,
                Some(State::GetSimpleString(_, _)) => Self::get_simple_string,
                Some(State::GetError(_, _)) => Self::get_error,
                Some(State::GetInteger(_)) => Self::get_integer,
                Some(State::GetBulkString(_, _)) => Self::get_bulk_string,
                Some(State::GetArray(_)) => Self::get_array,
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
        let len = self.stack.len();

        if len > 1 {
            match self.stack.get_mut(len - 2) {
                Some(State::GetArray(ga)) => ga.elements.push(value),
                _ => panic!("Invalid state in stack"),
            }
        } else {
            self.value = value;
        }
    }

    fn read<T: Read>(&mut self, stream: &mut T) -> Result<()> {
        let mut buf = vec![0; 20];
        let length = stream.read(&mut buf).unwrap();

        if length == 0 {
            return Err(RespError::UnexpectedEof);
        }

        for byte in buf[0..length].iter() {
            self.message.push(*byte)
        }

        Ok(())
    }

    fn get_type(&mut self) -> Result<Option<()>> {
        let i = self.index + 1;
        match self.message.get(self.index) {
            Some(b'+') =>
                self.transition_to(State::GetSimpleString(SubState::CheckCR, i)),
            Some(b'-') =>
                self.transition_to(State::GetError(SubState::CheckCR, i)),
            Some(b':') =>
                self.transition_to(State::GetInteger(SubState::CheckCR)),
            Some(b'$') =>
                self.transition_to(State::GetBulkString(SubState::GetSize, 0)),
            Some(b'*') =>
                self.transition_to(State::GetArray(GetArray::new())),
            _ => return Err(RespError::InvalidType),
        }

        self.index += 1;
        Ok(Some(()))
    }

    fn get_simple_string(&mut self) -> Result<Option<()>> {
        let (mut substate, i0) = match self.current_state() {
            Some(&State::GetSimpleString(ss, i)) => (ss, i),
            _ => unreachable!(),
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

    fn get_error(&mut self) -> Result<Option<()>> {
        let (mut substate, i0) = match self.current_state() {
            Some(&State::GetError(ss, i)) => (ss, i),
            _ => unreachable!(),
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

    fn get_integer(&mut self) -> Result<Option<()>> {
        let mut substate = match self.current_state() {
            Some(&State::GetInteger(s)) => s,
            _ => unreachable!(),
        };

        if substate == SubState::CheckCR {
            let start_index = self.index;
            if let Some(i) = self.find_break(start_index) {
                let v = self.parse_int(start_index, i)?;
                self.set_value(Value::Integer(v));
                self.index = i + 1;
                substate = SubState::CheckLF;
                self.transition_to(State::GetInteger(substate));
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

    fn get_bulk_string(&mut self) -> Result<Option<()>> {
        let (mut substate, mut size) = match self.current_state() {
            Some(&State::GetBulkString(ss, sz)) => (ss, sz),
            _ => unreachable!(),
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

    fn get_array(&mut self) -> Result<Option<()>> {
        let (mut substate, mut size) = match self.current_state() {
            Some(State::GetArray(ga)) => (ga.substate, ga.size),
            _ => unreachable!(),
        };

        if substate == SubState::GetSize {
            let start_index = self.index;
            if let Some(n) = self.get_size(start_index)? {
                size = n;
                substate = SubState::CheckLF;
                self.get_array_change(|sm| {
                    sm.size = size;
                    sm.substate = substate;
                });
            }
        }

        if substate == SubState::CheckLF {
            if self.check_lf()?.is_some() {
                substate = SubState::GetElements;
                self.get_array_change(|sm| {
                    sm.substate = substate;
                });
            }
        }

        if substate == SubState::GetElements {
            if size > 0 {
                self.get_array_change(|sm| {
                    sm.size -= 1;
                });
                self.stack.push(State::GetType);
            } else {
                let v = match self.stack.last_mut() {
                    Some(State::GetArray(ga)) => ga.pop_value(),
                    _ => unreachable!(),
                };
                self.set_value(v);
                self.stack.pop();
            }
            return Ok(Some(()));
        }

        Ok(None)
    }

    fn get_array_change<T: FnMut(&mut GetArray)>(&mut self, mut change: T) {
        let state_machine = match self.stack.last_mut() {
            Some(State::GetArray(ga)) => ga,
            _ => unreachable!(),
        };
        change(state_machine);
    }

    fn get_size(&mut self, start_index: usize) -> Result<Option<usize>> {
        if let Some(i) = self.find_break(start_index) {
            let n = self.parse_int(start_index, i)?;
            self.index = i + 1;
            return Ok(Some(n as usize));
        }

        Ok(None)
    }

    fn check_lf(&mut self) -> Result<Option<()>> {
        if let Some(&byte) = self.message.get(self.index) {
            if byte == b'\n' {
                self.index += 1;
                return Ok(Some(()));
            } else {
                // CR not followed by LF
                return Err(RespError::InvalidTerminator);
            }
        }

        Ok(None)
    }

    fn build_string(&mut self, size: usize) -> Result<Option<()>> {
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
                // Does not end with CRLF
                return Err(RespError::InvalidTerminator);
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

    fn parse_int(&self, start_index: usize, end_index: usize) -> Result<i64> {
        match str::from_utf8(&self.message[start_index..end_index]) {
            Ok(s) => s.parse::<i64>().map_err(|_| RespError::NotInteger),
            Err(_) => Err(RespError::NotInteger),
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
    GetArray(GetArray),
}

#[derive(Debug)]
struct GetArray {
    substate: SubState,
    size: usize,
    elements: Vec<Value>,
}

impl GetArray {
    fn new() -> Self {
        GetArray {
            substate: SubState::GetSize,
            size: 0,
            elements: vec![],
        }
    }

    fn pop_value(&mut self) -> Value {
        let v = &mut self.elements;
        let mut c = vec![];
        mem::swap(v, &mut c);
        Value::Array(c)
    }
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
    use super::RespError;
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

    fn check_invalid(s: &str, e: RespError) {
        let mut reader = RespReader::new();
        let mut stream = MockStream::from(s);

        let result = reader.frame_message(&mut stream);

        assert_eq!(result, Err(e));
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
        let err = RespError::InvalidTerminator;
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
        let err = RespError::NotInteger;
        check_invalid(no_size, err);

        let s = "$12\r\nHello World!\r\r";
        let err = RespError::InvalidTerminator;
        check_invalid(s, err);

        let incomplete = "$12\r\nHello";
        let err = RespError::UnexpectedEof;
        check_invalid(incomplete, err);
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

    #[test]
    fn check_invalid_type() {
        let invalid = "&hello";
        let err = RespError::InvalidType;
        check_invalid(invalid, err);
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

        let simple = "*1\r\n$12\r\nHello World!\r\n";
        let v = get_value(simple);
        assert_eq!(v, Value::Array(vec![Value::BulkString("Hello World!".to_string())]));

        // [[$"A", [-ERR]], +OK, :25]
        let nested_array = "*3\r\n*2\r\n$1\r\nA\r\n*1\r\n-ERR\r\n+OK\r\n:25\r\n";
        let expected = Value::Array(vec![
            Value::Array(vec![
                Value::BulkString("A".to_string()),
                Value::Array(vec![Value::Error("ERR".to_string())]),
            ]),
            Value::SimpleString("OK".to_string()),
            Value::Integer(25),
        ]);
        let v = get_value(nested_array);
        assert_eq!(v, expected);
    }
}
