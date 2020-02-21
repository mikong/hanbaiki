<p align="center">
  <a href="https://mikong.github.io/hanbaiki">
    <img src="assets/hanbaiki-logo.png" alt="Hanbaiki logo" width="115">
  </a>
</p>

<h3 align="center">Hanbaiki (販売機)</h3>

<p align="center">
  <a href="https://travis-ci.org/mikong/hanbaiki"><img src="https://travis-ci.org/mikong/hanbaiki.svg?branch=master" alt="Build Status"></a>
</p>

<p align="center">
  A simple key-value store written in Rust.
</p>

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
