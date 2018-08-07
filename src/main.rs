extern crate hanbaiki;

use std::io;
use std::io::Write;
use std::collections::HashMap;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{RwLock, Arc};

use hanbaiki::{RespWriter, RespReader};
use hanbaiki::Value;

type KvStore = Arc<RwLock<HashMap<String, Vec<u8>>>>;

fn main() {
    let data = Arc::new(RwLock::new(HashMap::new()));

    data_server("127.0.0.1:6363", data);
}

fn data_server(addr: &str, data: KvStore) {
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
