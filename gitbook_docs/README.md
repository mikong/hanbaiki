# Hanbaiki (販売機) [![Build Status][travis-image]][travis]

A simple key-value store written in Rust.

If you're interested to know the meaning of the name, see [project name page](https://mikong.github.io/hanbaiki/name.html).

## Usage

**Warning**: This project is still in the pre-alpha stage.

Download the source:

```
$ git clone git@github.com:mikong/hanbaiki.git
```

### Running the server

```
$ ./target/debug/hanbaiki
listening on 127.0.0.1:6363
```

### Running the client

```
$ ./target/debug/cli
> SET hello world
OK
> GET hello
"world"
```

For a work-in-progress list of commands, check the [commands page](https://mikong.github.io/hanbaiki/commands.html).

## License

This software is distributed under the [MIT License](https://github.com/mikong/hanbaiki/blob/master/LICENSE).

[travis-image]: https://travis-ci.org/mikong/hanbaiki.svg?branch=master
[travis]: https://travis-ci.org/mikong/hanbaiki
