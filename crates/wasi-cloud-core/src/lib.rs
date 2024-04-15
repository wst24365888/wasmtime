//! wasmCloud runtime library

#![allow(clippy::module_name_repetitions)]
#![warn(missing_docs)]
#![forbid(clippy::unwrap_used)]

pub mod component;

/// Capability provider implementations and adaptors
pub mod capability;

/// wasmCloud I/O functionality
pub mod io;

pub use async_trait::async_trait;
pub use tokio;
