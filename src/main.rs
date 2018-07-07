use std::io;
use std::io::{Write, Read};
use std::collections::HashMap;

use std::net::TcpListener;

fn main() {
    let mut data = HashMap::new();

    data_server("127.0.0.1:6363", &mut data).expect("error: ");
}

fn data_server(addr: &str, data: &mut HashMap<String, Vec<u8>>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("listening on {}", addr);

    loop {
        // Wait for a client to connect.
        let (mut stream, addr) = listener.accept()?;
        println!("connection received from {}", addr);

        let mut write_stream = stream.try_clone()?;

        // Read command.
        let mut buf = vec![];
        stream.read_to_end(&mut buf)?;
        let command = String::from_utf8(buf).unwrap();
        let response = process_command(data, &command);

        // Return response.
        write_stream.write(response.as_bytes())?;
        println!("connection closed");
    }
}

fn process_command(data: &mut HashMap<String, Vec<u8>>, command: &str) -> String {
    let v: Vec<&str> = command.split_whitespace().collect();

    let command = v[0];

    match command {

        "SET" if v.len() == 3 => {
            data.insert(v[1].to_string(), v[2].as_bytes().to_vec());
            "OK".to_string()
        },


        "GET" if v.len() == 2 => {
            if let Some(value) = data.get(v[1]) {
                String::from_utf8_lossy(value).into_owned()
            } else {
                "ERROR: KEY NOT FOUND".to_string()
            }
        },

        "DELETE" if v.len() == 2 => {
            if let Some(_) = data.remove(v[1]) {
                "OK".to_string()
            } else {
                "ERROR: KEY NOT FOUND".to_string()
            }
        },

        "EXISTS" if v.len() == 2 => {
            if data.contains_key(v[1]) {
                "1".to_string()
            } else {
                "0".to_string()
            }
        },

        _ => {
            "Command not recognized. Try again".to_string()
        },
    }
}
