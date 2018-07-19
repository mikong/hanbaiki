use std::io::Read;
use std::str;

#[derive(Debug)]
pub struct RespReader {
    pub message: Vec<u8>,
    index: usize,
}

impl RespReader {
    pub fn new() -> Self {
        RespReader {
            message: vec![],
            index: 0,
        }
    }

    pub fn frame_message<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {

        let mut type_buf = vec![0; 1];
        let _length = stream.read(&mut type_buf).unwrap();

        self.message.push(type_buf[0]);

        match type_buf[0] {
            b'+' | b'-' => self.get_simple_message(stream)?,
            b':' => self.get_integer(stream)?,
            b'$' => self.get_bulk_string(stream)?,
            // TODO: b'*' => ,
            _ => return Err("Invalid RESP type".to_string()),
        }

        Ok(())
    }

    fn get_simple_message<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {
        let mut buf = vec![0; 20];
        let mut has_cr = false;

        loop {
            let length = stream.read(&mut buf).unwrap();

            if length == 0 {
                return Err("EOF before end of frame".to_string());
            }

            for byte in buf[0..length].iter() {

                if has_cr && *byte != b'\n' {
                    return Err("CR not followed by LF".to_string());
                }

                self.message.push(*byte);

                if *byte == b'\r' {
                    has_cr = true;
                } else if *byte == b'\n' {
                    if has_cr {
                        return Ok(());
                    } else {
                        return Err("LF before CR".to_string());
                    }
                }
            }
        }
    }

    fn get_integer<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {
        self.get_simple_message(stream)?;

        match self.parse_int(self.message.len() - 2) {
            Some(_) => Ok(()),
            None => Err("Not an integer".to_string()),
        }
    }

    fn get_bulk_string<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {
        let mut buf = vec![0; 20];
        let mut state = BulkStringState::GetSize;
        let mut size = 0;

        self.index = self.message.len();

        loop {
            let length = stream.read(&mut buf).unwrap();

            if length == 0 {
                return Err("EOF before end of frame".to_string());
            }

            for byte in buf[0..length].iter() {
                self.message.push(*byte)
            }

            if state == BulkStringState::GetSize {
                let start_index = self.index;
                if let Some(n) = self.get_size(start_index)? {
                    size = n;
                    state = BulkStringState::CheckEOL;
                }
            }

            if state == BulkStringState::CheckEOL {
                if self.check_eol()?.is_some() {
                    state = BulkStringState::BuildString;
                }
            }

            if state == BulkStringState::BuildString {
                if self.build_string(size as usize)?.is_some() {
                    return Ok(());
                }
            }

        }

    }

    fn get_size(&mut self, start_index: usize) -> Result<Option<usize>, String> {
        let mut size = None;

        if let Some(i) = self.find_break(start_index) {
            match self.parse_int(i) {
                Some(n) => {
                    size = Some(n as usize);
                    self.index = i + 1;
                },
                None => return Err("Not an integer".to_string()),
            }
        } else {
            self.index = self.message.len();
        }

        Ok(size)
    }

    fn check_eol(&mut self) -> Result<Option<()>, String> {
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
            self.index += size;
            if self.message[self.index] == b'\r' && self.message[self.index + 1] == b'\n' {
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

    fn parse_int(&self, end_index: usize) -> Option<i64> {
        match str::from_utf8(&self.message[1..end_index]) {
            Ok(s) => s.parse::<i64>().ok(),
            Err(_) => None,
        }
    }

}

#[derive(Debug, PartialEq)]
enum BulkStringState {
    GetSize,
    CheckEOL,
    BuildString,
}

#[cfg(test)]
mod test {

    use super::RespReader;
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
        let split_crlf = "+1234567890123456789\r\n";
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
}
