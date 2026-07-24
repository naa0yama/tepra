//! tepra: REST API layer.

pub mod actor;
pub mod handlers;
pub mod router;
pub mod state;
pub mod templates;
pub mod views;

/// Returns the crate version from Cargo metadata.
#[must_use]
pub const fn router_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
