extern crate hanbaiki;

#[macro_use]
extern crate clap;

use hanbaiki::Server;
use hanbaiki::Config;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Hanbaiki")
        .version(crate_version!())
        .about("A simple key-value store.")
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
        .arg(Arg::with_name("PIDFILE")
            .help("Generate a pidfile at the specified path. Example: /var/run/hanbaiki.pid")
            .takes_value(true)
            .long("pidfile"))
        .get_matches();

    let config = Config::new(matches);

    Server::run(config);

    println!("Exiting");
}
