use std::io::Read;
use std::str;

#[derive(Debug)]
pub struct RespReader {
    pub message: Vec<u8>,
}

impl RespReader {
    pub fn new() -> Self {
        RespReader { message: vec![] }
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

        let s = str::from_utf8(&self.message[1..self.message.len()-2]).unwrap();
        match s.parse::<i64>() {
            Ok(_) => Ok(()),
            Err(_) => Err("Not an integer".to_string()),
        }
    }

    fn get_bulk_string<T: Read>(&mut self, stream: &mut T) -> Result<(), String> {
        let mut buf = vec![0; 20];
        let mut state = BulkStringState::GetSize;
        let mut size = 0;

        let mut index = self.message.len();

        loop {
            let length = stream.read(&mut buf).unwrap();

            if length == 0 {
                return Err("EOF before end of frame".to_string());
            }

            for byte in buf[0..length].iter() {
                self.message.push(*byte)
            }

            if state == BulkStringState::GetSize {
                if let Some(i) = self.find_break(index) {
                    match self.parse_int(i) {
                        Some(n) => {
                            size = n;
                            state = BulkStringState::CheckEOL;
                            index = i + 1;
                        },
                        None => return Err("Not an integer".to_string()),
                    }
                } else {
                    index = self.message.len();
                }
            }

            if state == BulkStringState::CheckEOL {
                if let Some(byte) = self.message.get(index) {
                    if *byte == b'\n' {
                        state = BulkStringState::BuildString;
                        index += 1;
                    } else {
                        return Err("CR not followed by LF".to_string());
                    }
                }
            }

            if state == BulkStringState::BuildString {
                if self.message.len() > index + size as usize {
                    index += size as usize;
                    if self.message[index] == b'\r' && self.message[index + 1] == b'\n' {
                        return Ok(());
                    } else {
                        return Err("Does not end with CRLF".to_string());
                    }
                }
            }

        }

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

    #[test]
    fn check_empty_string() {
        let mut reader = RespReader::new();

        let empty = "+\r\n".to_string();
        let mut stream = MockStream::from(&empty);

        reader.frame_message(&mut stream).unwrap();

        assert_eq!(empty, String::from_utf8(reader.message).unwrap());
    }

    #[test]
    fn check_simple_string() {
        let mut reader = RespReader::new();

        let simple = "+OK\r\n".to_string();
        let mut stream = MockStream::from(&simple);

        reader.frame_message(&mut stream).unwrap();

        assert_eq!(simple, String::from_utf8(reader.message).unwrap());
    }

    #[test]
    fn check_split_crlf() {
        let mut reader = RespReader::new();

        // with reader buf size of 20, \n is on the next read
        let split_crlf = "+1234567890123456789\r\n".to_string();
        let mut stream = MockStream::from(&split_crlf);

        reader.frame_message(&mut stream).unwrap();

        assert_eq!(split_crlf, String::from_utf8(reader.message).unwrap());
    }

    #[test]
    fn check_nonconsecutive_crlf() {
        let mut reader = RespReader::new();

        let s = "+123\r4\n".to_string();
        let mut stream = MockStream::from(&s);

        let result = reader.frame_message(&mut stream);

        assert_eq!(result, Err("CR not followed by LF".to_string()));
    }
}
