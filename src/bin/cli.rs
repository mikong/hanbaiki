extern crate hanbaiki;
extern crate clap;

use std::net::TcpStream;
use std::io;
use std::io::{Write};

use hanbaiki::{RespWriter, RespReader};
use hanbaiki::Value;

use clap::{App};

fn main() {
    let _matches = App::new("Hanbaiki CLI")
        .get_matches();

    let mut stream = TcpStream::connect("127.0.0.1:6363")
        .expect("Couldn't connect to the server...");

    stream.set_nodelay(true).expect("set_nodelay failed");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        process_command(&command, &mut stream);
    }
}

fn process_command(command: &str, stream: &mut TcpStream) {
    let v: Vec<&str> = command.split_whitespace().collect();

    let serialized = RespWriter::to_array(&v);

    stream.write_all(serialized.as_bytes())
        .expect("Could not write");
    stream.flush().expect("Could not flush");

    read(stream);
}

fn read(stream: &mut TcpStream) {
    let mut reader = RespReader::new();
    reader.frame_message(stream).unwrap();
    let response = reader.value;

    match response {
        Value::SimpleString(s) => println!("{}", s),
        Value::Error(s) => println!("(error) {}", s),
        Value::Integer(i) => println!("(integer) {}", i),
        Value::BulkString(s) => println!("\"{}\"", s),
        _ => panic!("Unexpected Value type"),
    }
}
