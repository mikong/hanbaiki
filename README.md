# Hanbaiki (販売機) [![Build Status][travis-image]][travis]

A simple key-value store written in Rust.

If you're interested to know the meaning of the name, see [project name page](https://mikong.github.io/hanbaiki/name.html).

## Installation

**Warning**: This project is still in the pre-alpha stage.

Precompiled binaries are available for Linux, macOS, and Windows in [Releases](https://github.com/mikong/hanbaiki/releases). Simply download and extract the archive to get the binaries for the server (hanbaiki) and interactive client (cli).

## Usage

### Running the server

```
$ ./hanbaiki
listening on 127.0.0.1:6363
```

### Running the client

```
$ ./cli
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
