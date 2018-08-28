# Benchmark

The benchmark only simulates one TCP client connecting to the Hanbaiki server on 127.0.0.1:6363. It runs each command 10,000 times per iteration. These parameters (number of clients, hostname, port, executions per iteration) are currently hardcoded.

## Usage

Hanbaiki uses the unstable `test` crate for benchmarking, so you'll need to install nightly Rust.

Install nightly:

```
$ rustup install nightly
```

Check *toolchains*:

```
$ rustup toolchain list
```

To compile the benchmark code without running the tests:

```
$ rustup run nightly cargo bench --no-run
```

Before running the benchmark, you need to run the Hanbaiki server at 127.0.0.1:6363. Compile the code with the `--release` option to get the optimizations, and then run the binary in the release directory:

```
$ cargo build --release
$ ./target/release/hanbaiki
```

**Warning**: Running the benchmark will connect to 127.0.0.1:6363 and clear the data as it calls `DESTROY` on every benchmark test.

To run `cargo bench` with nightly Rust, use:

```
$ rustup run nightly cargo bench
```
