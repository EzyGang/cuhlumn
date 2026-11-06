//! Request and response types of the cuhlumn HTTP API.
//!
//! The server, the CLI, and the console share these types.
//! Value types validate their input when they are constructed or deserialized, so a value that exists is valid.
//! With the `ts` feature, every type derives [`ts_rs::TS`](https://docs.rs/ts-rs) for the console.

#[macro_use]
mod macros;

pub mod post;
pub mod site;
pub mod text;
pub mod wire;
