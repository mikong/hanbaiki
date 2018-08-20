use std::fmt::Display;
use std::io;
use std::io::Write;
use std::collections::HashMap;

use std::net::{ToSocketAddrs, TcpListener, TcpStream};
use std::thread;
use std::sync::{RwLock, Arc};

use respreader::RespReader;
use respwriter::RespWriter;
use response::Response;
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
                match process_command(data, command) {
                    Response::KeepAlive(response) => write_stream.write(response.as_bytes())?,
                    Response::Close(response) => {
                        write_stream.write(response.as_bytes())?;
                        return Ok(());
                    },
                };

                
            },
            Err(e) => {
                println!("{:?}", e);
                return Ok(())
            },
        }
    }
}

fn process_command(data: KvStore, command: Value) -> Response {
    let mut v = match command {
        Value::Array(values) => values,
        _ => panic!("Expected command to be Value::Array"),
    };

    let command = v[0].take().to_string().to_ascii_uppercase();

    match command.as_ref() {

        "SET" if v.len() == 3 => {
            let mut data = data.write().unwrap();
            data.insert(v[1].take().to_string(), v[2].take().to_string().into_bytes());
            Response::build_ok()
        },

        "GET" if v.len() == 2 => {
            let data = data.read().unwrap();
            if let Some(value) = data.get(&v[1].take().to_string()) {
                let value = String::from_utf8_lossy(value).into_owned();
                Response::KeepAlive(RespWriter::to_bulk_string(&value))
            } else {
                Response::build_error("ERROR: Key not found")
            }
        },

        "DELETE" if v.len() == 2 => {
            let mut data = data.write().unwrap();
            if let Some(_) = data.remove(&v[1].take().to_string()) {
                Response::build_ok()
            } else {
                Response::build_error("ERROR: Key not found")
            }
        },

        "EXISTS" if v.len() == 2 => {
            let data = data.read().unwrap();
            if data.contains_key(&v[1].take().to_string()) {
                Response::KeepAlive(RespWriter::to_integer(1))
            } else {
                Response::KeepAlive(RespWriter::to_integer(0))
            }
        },

        "COUNT" if v.len() == 1 => {
            let data = data.read().unwrap();
            Response::KeepAlive(RespWriter::to_integer(data.len()))
        },

        "DESTROY" if v.len() == 1 => {
            let mut data = data.write().unwrap();
            data.clear();
            Response::build_ok()
        },

        "QUIT" | "EXIT" if v.len() == 1 => {
            Response::build_close_ok()
        },

        _ => {
            Response::build_error("ERROR: Command not recognized")
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
        let expected = Response::build_ok();
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
        let expected = Response::KeepAlive(RespWriter::to_bulk_string("world"));
        assert_eq!(response, expected);
    }

    #[test]
    fn lowercase_get_set() {
        let command = vec!["set".to_string(), "hello".to_string(), "world".to_string()].into();
        let data = Arc::new(RwLock::new(HashMap::new()));

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_ok();
        assert_eq!(response, expected);

        let command = vec!["get".to_string(), "hello".to_string()].into();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::KeepAlive(RespWriter::to_bulk_string("world"));
        assert_eq!(response, expected);
    }

    #[test]
    fn delete_command() {
        let command = vec!["DELETE".to_string(), "hello".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_ok();
        assert_eq!(response, expected);

        let command = vec!["DELETE".to_string(), "hello".to_string()].into();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_error("ERROR: Key not found");
        assert_eq!(response, expected);
    }

    #[test]
    fn exists_command() {
        let command = vec!["EXISTS".to_string(), "hello".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::KeepAlive(RespWriter::to_integer(1));
        assert_eq!(response, expected);

        let command = vec!["EXISTS".to_string(), "nonexistent".to_string()].into();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::KeepAlive(RespWriter::to_integer(0));
        assert_eq!(response, expected);
    }

    #[test]
    fn count_command() {
        let command = vec!["COUNT".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::KeepAlive(RespWriter::to_integer(1));
        assert_eq!(response, expected);
    }

    #[test]
    fn destroy_command() {
        let command = vec!["DESTROY".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_ok();
        assert_eq!(response, expected);

        let r = data.read().unwrap();
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn quit_command() {
        let command = vec!["QUIT".to_string()].into();
        let data = init_data();

        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_close_ok();
        assert_eq!(response, expected);
    }
}
