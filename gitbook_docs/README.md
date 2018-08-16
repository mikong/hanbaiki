# Hanbaiki

A simple key-value store written in Rust.

[![Build Status](https://travis-ci.org/mikong/hanbaiki.svg?branch=master)](https://travis-ci.org/mikong/hanbaiki)

## Usage

Warning: This project is still in the pre-alpha stage.

Download the source:

```
$ git clone git@github.com:mikong/hanbaiki.git
```

### Running the server

```
$ cargo run --bin hanbaiki
listening on 127.0.0.1:6363
```

### Running the client

```
$ cargo run --bin cli
> SET hello world
OK
> GET hello
"world"
```

## License

This software is distributed under the [MIT License](https://github.com/mikong/hanbaiki/blob/master/LICENSE).
