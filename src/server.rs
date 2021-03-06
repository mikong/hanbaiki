use std::io;
use std::io::Write;
use std::collections::HashMap;

use std::fs::File;
use std::path::PathBuf;
use std::process;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::{RwLock, Arc};

use config::Config;
use respreader::RespReader;
use respwriter::RespWriter;
use response::Response;
use value::Value;

type KvStore = Arc<RwLock<HashMap<String, Vec<u8>>>>;

pub struct Server;

impl Server {
    pub fn run(config: Config) {
        create_pidfile(config.pidfile);

        let data = Arc::new(RwLock::new(HashMap::new()));

        let addr = SocketAddr::new(config.ip, config.port);
        let listener = match TcpListener::bind(addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Couldn't bind to address: {:?}", e);
                return;
            }
        };
        println!("Listening on {}", addr);

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

/// Attempts to create a PID file if the pidfile option was provided.
///
/// This function fails silently if it's unable to create or write to the file.
fn create_pidfile(pidfile: Option<PathBuf>) {
    if let Some(p) = pidfile {
        if let Ok(mut f) = File::create(p) {
            let _ = f.write_all(process::id().to_string().as_bytes());
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
        _ => return Response::build_error("ERROR: Command must be an array"),
    };

    if v.len() == 0 {
        return Response::build_error("ERROR: Missing command");
    }

    if v.iter().any(|value| match value {
        Value::BulkString(_) => false,
        _ => true,
    }) {
        return Response::build_error("ERROR: Command must be an array of BulkString");
    }

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
    fn invalid_command() {
        let data = Arc::new(RwLock::new(HashMap::new()));

        let command = Value::BulkString("DESTROY".to_string());
        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_error("ERROR: Command must be an array");
        assert_eq!(response, expected);

        let command = Vec::new().into();
        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_error("ERROR: Missing command");
        assert_eq!(response, expected);

        let command = Value::Array(vec![
            Value::BulkString("EXISTS".to_string()),
            Value::SimpleString("hello".to_string()),
        ]);
        let response = process_command(Arc::clone(&data), command);
        let expected = Response::build_error("ERROR: Command must be an array of BulkString");
        assert_eq!(response, expected);
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
