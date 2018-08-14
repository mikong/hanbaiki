extern crate hanbaiki;

#[macro_use]
extern crate clap;

use hanbaiki::Server;

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
        .get_matches();

    let port = if matches.is_present("PORT") {
        value_t!(matches, "PORT", u16).unwrap_or_else(|_| {
            println!("Specified port value is invalid, using default 6363.");
            6363
        })
    } else {
        6363
    };

    let address = format!("127.0.0.1:{}", port);

    Server::run(&address);
}
