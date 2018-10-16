use std::net::IpAddr;
use std::path::PathBuf;

use clap::{ArgMatches, ErrorKind};

#[derive(Debug)]
pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
    pub pidfile: Option<PathBuf>,
}

impl Config {
    pub fn new(matches: ArgMatches) -> Self {
        let port = value_t!(matches, "PORT", u16).unwrap_or_else(|e| {
            if e.kind == ErrorKind::ValueValidation {
                println!("Specified port value is invalid, using default 6363.");
            }
            6363
        });

        let ip = value_t!(matches, "IP", IpAddr).unwrap_or_else(|e| {
            if e.kind == ErrorKind::ValueValidation {
                println!("Specified IP address is invalid, using default 127.0.0.1.");
            }
            "127.0.0.1".parse().unwrap()
        });

        let pidfile = matches
            .value_of("PIDFILE")
            .and_then(|v| Some(PathBuf::from(v)));

        Config { ip, port, pidfile }
    }
}
