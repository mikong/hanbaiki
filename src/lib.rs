//! A simple key-value store
//!
//! Hanbaiki uses the same protocol as Redis for its client-server communication called RESP (REdis
//! Serialization Protocol). In practice, you can use any client that supports the Redis protocol to
//! communicate with a Hanbaiki server.

#[macro_use]
extern crate clap;

mod config;
pub mod resp_error;
mod respreader;
mod respwriter;
mod server;
pub mod client;
mod response;
mod value;

pub use config::Config;
pub use respreader::RespReader;
pub use respwriter::RespWriter;
pub use server::Server;
pub use value::Value;
