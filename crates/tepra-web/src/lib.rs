#![warn(missing_docs)]
//! tepra-web: HTTP server and templates.

pub mod assets;
pub mod cli;
pub mod trace;

/// Crate version baked at compile time.
#[must_use]
pub const fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
