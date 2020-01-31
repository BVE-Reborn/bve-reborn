//! Logger and associated tooling.
//!
//! BVE-Reborn uses a fully async logger. All log events are sent to a different thread to be serialized and written to
//! file. The logger backend is implemented using the `tracing` library.

pub use data::*;
pub use method::*;
pub use subscriber::*;

mod common;
mod data;
mod method;
mod subscriber;
mod writer;
