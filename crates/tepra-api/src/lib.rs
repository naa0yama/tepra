//! tepra-api: REST API layer.

/// Returns the crate version from Cargo metadata.
#[must_use]
pub const fn router_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
