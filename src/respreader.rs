use std::net::TcpStream;
use std::io::Read;

#[derive(Debug)]
pub struct RespReader {
    pub message: Vec<u8>,
}

impl RespReader {
    pub fn new() -> Self {
        RespReader { message: vec![] }
    }

    pub fn frame_message(&mut self, mut stream: &mut TcpStream) -> Result<(), String> {

        let mut type_buf = vec![0; 1];
        let _length = stream.read(&mut type_buf).unwrap();

        self.message.push(type_buf[0]);

        match type_buf[0] {
            b'+' | b'-' => self.get_simple_message(&mut stream)?,
            b':' => {
                self.get_simple_message(&mut stream)?
                // TODO: return Err if value doesn't pass as i64
            },
            // TODO: b'$' => ,
            // TODO: b'*' => ,
            _ => return Err("Invalid RESP type".to_string()),
        }

        Ok(())
    }

    fn get_simple_message(&mut self, stream: &mut TcpStream) -> Result<(), String> {
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

}
