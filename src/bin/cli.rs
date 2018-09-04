extern crate hanbaiki;

#[macro_use]
extern crate clap;

use std::net::{TcpStream, SocketAddr};
use std::io;
use std::io::{Write};

use hanbaiki::{RespWriter, RespReader};
use hanbaiki::Value;
use hanbaiki::client::config::Config;

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
        .arg(Arg::with_name("IP")
            .help("Specify a custom IP to bind to. Default: 127.0.0.1")
            .takes_value(true)
            .long("bind")
            .short("b"))
        .get_matches();

    let config = Config::new(matches);

    let address = SocketAddr::new(config.ip, config.port);
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

    if v.len() == 0 { return }

    let serialized = RespWriter::to_array(&v);

    stream.write_all(serialized.as_bytes())
        .expect("Could not write");
    stream.flush().expect("Could not flush");

    read(stream);

    if v.len() == 1 {
        let cmd = v[0].to_ascii_uppercase();
        if cmd == "EXIT" || cmd == "QUIT" {
            std::process::exit(0);
        }
    }
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
