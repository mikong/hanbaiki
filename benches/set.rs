#![feature(test)]

extern crate test;
extern crate rand;

#[macro_use]
extern crate lazy_static;

extern crate hanbaiki;

use std::net::TcpStream;
use std::io::{Write, Read};

use rand::prelude::*;

use hanbaiki::RespWriter;

lazy_static! {
    static ref KEYS: Vec<String> = random_int();
    static ref VALUES: Vec<String> = random_int();
}

fn random_int() -> Vec<String> {
    let mut v = Vec::new();
    for _ in 0..10_000 {
        let n: u32 = random();
        // add 15 characters for min length of 16
        let s = format!("abcdefghijklmno{}", n);
        v.push(s);
    }
    v
}

pub fn set(i: usize, stream: &mut TcpStream) {
    let v = vec!["SET", &KEYS[i], &VALUES[i]];
    let serialized = RespWriter::to_array(&v);

    stream.write_all(serialized.as_bytes()).expect("Could not write");
    stream.flush().expect("Could not flush");

    let mut buf = vec![0; 40];
    stream.read(&mut buf).expect("Could not read");
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_set(b: &mut Bencher) {
        let mut stream = TcpStream::connect("127.0.0.1:6363")
            .expect("Couldn't connect to the server...");
        stream.set_nodelay(true).expect("set_nodelay failed");

        b.iter(|| {
            for i in 0..10_000 {
                set(i, &mut stream);
            }
        });
    }
}
