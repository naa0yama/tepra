//! Domain error type for the TEPRA Creator `WebAPI` client.

use thiserror::Error;

/// Errors that can occur when communicating with the Creator `WebAPI`.
#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum TepraError {
    /// The Creator API returned a non-success `errcode` in the response body.
    #[error("Creator API error (errcode={errcode})")]
    Creator {
        /// Raw error code from `{ "errcode": N }`.
        errcode: u32,
    },

    /// An HTTP-level transport failure (connection refused, timeout, TLS, etc.).
    #[error("HTTP transport error: {source}")]
    Transport {
        /// Underlying transport error.
        #[source]
        source: anyhow::Error,
    },

    /// The response body could not be parsed into the expected DTO.
    #[error("Response parse error: {source}")]
    Parse {
        /// Underlying parse error.
        #[source]
        source: anyhow::Error,
    },

    /// The server returned a non-success HTTP status code.
    #[error("HTTP error {status}")]
    Http {
        /// HTTP status code (e.g. 404, 500).
        status: u16,
    },

    /// The per-printer actor worker has shut down; the request cannot be processed.
    #[error("Printer actor has shut down")]
    ActorShutdown,
}
