use std::fmt::Display;
use std::io;
use std::io::Write;
use std::collections::HashMap;

use std::net::{ToSocketAddrs, TcpListener, TcpStream};
use std::thread;
use std::sync::{RwLock, Arc};

use respreader::RespReader;
use respwriter::RespWriter;
use value::Value;

type KvStore = Arc<RwLock<HashMap<String, Vec<u8>>>>;

pub struct Server;

impl Server {
    pub fn run<T>(addr: &T)
        where T: ToSocketAddrs + Display
    {
        let data = Arc::new(RwLock::new(HashMap::new()));

        let listener = TcpListener::bind(addr).unwrap();
        println!("listening on {}", addr);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let data = Arc::clone(&data);
                    thread::spawn(move || {
                        handle_client(stream, data)
                    });
                },
                Err(e) => println!("connection failed: {:?}", e),
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, data: KvStore) -> io::Result<()> {
    let mut write_stream = stream.try_clone()?;
    write_stream.set_nodelay(true)?;

    loop {
        // Read command.
        let mut reader = RespReader::new();
        match reader.frame_message(&mut stream) {
            Ok(_) => {
                let command = reader.value;

                let data = Arc::clone(&data);
                let response = process_command(data, command);

                write_stream.write(response.as_bytes())?;
            },
            Err(e) => {
                println!("{:?}", e);
                return Ok(())
            },
        }
    }
}

fn process_command(data: KvStore, command: Value) -> String {
    let mut v = match command {
        Value::Array(values) => values,
        _ => panic!("Expected command to be Value::Array"),
    };

    let command = v[0].take().to_string();

    match command.as_ref() {

        "SET" if v.len() == 3 => {
            let mut data = data.write().unwrap();
            data.insert(v[1].take().to_string(), v[2].take().to_string().into_bytes());
            RespWriter::to_simple_string("OK").unwrap()
        },

        "GET" if v.len() == 2 => {
            let data = data.read().unwrap();
            if let Some(value) = data.get(&v[1].take().to_string()) {
                let value = String::from_utf8_lossy(value).into_owned();
                RespWriter::to_bulk_string(&value)
            } else {
                RespWriter::to_error("ERROR: Key not found").unwrap()
            }
        },

        "DELETE" if v.len() == 2 => {
            let mut data = data.write().unwrap();
            if let Some(_) = data.remove(&v[1].take().to_string()) {
                RespWriter::to_simple_string("OK").unwrap()
            } else {
                RespWriter::to_error("ERROR: Key not found").unwrap()
            }
        },

        "EXISTS" if v.len() == 2 => {
            let data = data.read().unwrap();
            if data.contains_key(&v[1].take().to_string()) {
                RespWriter::to_integer(1)
            } else {
                RespWriter::to_integer(0)
            }
        },

        _ => {
            RespWriter::to_error("ERROR: Command not recognized").unwrap()
        },
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn init_data() -> KvStore {
        let mut data = HashMap::new();
        data.insert("hello".to_string(), "world".to_string().into_bytes());
        Arc::new(RwLock::new(data))
    }

    #[test]
    fn set_command() {
        let command = vec!["SET".to_string(), "hello".to_string(), "world".to_string()].into();
        let data = Arc::new(RwLock::new(HashMap::new()));

        let response = process_command(Arc::clone(&data), command);
        let expected = RespWriter::to_simple_string("OK").unwrap();
        assert_eq!(response, expected);

        let r = data.read().unwrap();
        let value = r.get("hello").unwrap();
        let expected = &"world".to_string().into_bytes();
        assert_eq!(value, expected);
    }

    #[test]
    fn get_command() {
        let command = vec!["GET".to_string(), "hello".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = RespWriter::to_bulk_string("world");
        assert_eq!(response, expected);
    }

    #[test]
    fn delete_command() {
        let command = vec!["DELETE".to_string(), "hello".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = RespWriter::to_simple_string("OK").unwrap();
        assert_eq!(response, expected);

        let command = vec!["DELETE".to_string(), "hello".to_string()].into();

        let response = process_command(Arc::clone(&data), command);
        let expected = RespWriter::to_error("ERROR: Key not found").unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn exists_command() {
        let command = vec!["EXISTS".to_string(), "hello".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = RespWriter::to_integer(1);
        assert_eq!(response, expected);

        let command = vec!["EXISTS".to_string(), "nonexistent".to_string()].into();

        let response = process_command(Arc::clone(&data), command);
        let expected = RespWriter::to_integer(0);
        assert_eq!(response, expected);
    }
}
