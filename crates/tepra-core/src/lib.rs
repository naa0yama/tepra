//! tepra-core: domain types and KING JIM `WebAPI` client.

#![warn(missing_docs)]

pub mod dto;
pub mod otel;

/// Crate version baked at compile time.
#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
