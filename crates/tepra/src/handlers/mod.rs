//! HTTP request handlers.

use axum::http::StatusCode;

pub mod jobs;
pub mod openapi;
pub mod printers;
pub mod templates;
pub mod views;

/// Map any [`tepra_core::error::TepraError`] from the upstream Creator API to
/// `502 Bad Gateway`. Used by every `/api/*` handler that proxies to the client.
pub(crate) fn err_502(_: tepra_core::error::TepraError) -> StatusCode {
    StatusCode::BAD_GATEWAY
}
