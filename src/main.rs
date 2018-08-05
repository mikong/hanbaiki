extern crate hanbaiki;

use std::io;
use std::io::Write;
use std::collections::HashMap;

use std::net::TcpListener;

use hanbaiki::{RespWriter, RespReader};
use hanbaiki::Value;

fn main() {
    let mut data = HashMap::new();

    data_server("127.0.0.1:6363", &mut data).expect("error: ");
}

fn data_server(addr: &str, data: &mut HashMap<String, Vec<u8>>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("listening on {}", addr);

    // Wait for a client to connect.
    let (mut stream, addr) = listener.accept()?;
    println!("connection received from {}", addr);

    let mut write_stream = stream.try_clone()?;
    write_stream.set_nodelay(true)?;

    loop {
        // Read command.
        let mut reader = RespReader::new();
        reader.frame_message(&mut stream).unwrap();
        let command = reader.value;

        let response = process_command(data, command);

        // Return response.
        write_stream.write(response.as_bytes())?;
    }
}

fn process_command(data: &mut HashMap<String, Vec<u8>>, command: Value) -> String {
    let mut v = match command {
        Value::Array(values) => values,
        _ => panic!("Expected command to be Value::Array"),
    };

    let command = v[0].take().to_string();

    match command.as_ref() {

        "SET" if v.len() == 3 => {
            data.insert(v[1].take().to_string(), v[2].take().to_string().into_bytes());
            RespWriter::to_simple_string("OK").unwrap()
        },


        "GET" if v.len() == 2 => {
            if let Some(value) = data.get(&v[1].take().to_string()) {
                let value = String::from_utf8_lossy(value).into_owned();
                RespWriter::to_bulk_string(&value)
            } else {
                RespWriter::to_error("ERROR: Key not found").unwrap()
            }
        },

        "DELETE" if v.len() == 2 => {
            if let Some(_) = data.remove(&v[1].take().to_string()) {
                RespWriter::to_simple_string("OK").unwrap()
            } else {
                RespWriter::to_error("ERROR: Key not found").unwrap()
            }
        },

        "EXISTS" if v.len() == 2 => {
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
