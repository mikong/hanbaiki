extern crate hanbaiki;

#[macro_use]
extern crate clap;

use std::net::TcpStream;
use std::io;
use std::io::{Write};

use hanbaiki::{RespWriter, RespReader};
use hanbaiki::Value;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Hanbaiki CLI")
        .version(crate_version!())
        .about("This is a CLI for the simple key-value store Hanbaiki.")
        .arg(Arg::with_name("PORT")
            .help("Specify a custom port. Default: 6363")
            .takes_value(true)
            .long("port")
            .short("p"))
        .get_matches();

    let port = value_t!(matches, "PORT", u16).unwrap_or_else(|_| {
        println!("Specified port value is invalid, using default 6363.");
        6363
    });

    let address = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(address)
        .expect("Couldn't connect to the server...");

    stream.set_nodelay(true).expect("set_nodelay failed");

    start_repl(&mut stream);
}

fn start_repl(stream: &mut TcpStream) {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        process_command(&command, stream);
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
        _ => unreachable!(),
    }
}
