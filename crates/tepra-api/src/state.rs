//! Shared application state threaded through all Axum handlers.

use std::sync::Arc;

use tepra_core::client::traits::TepraClient;

use crate::actor::registry::PrinterRegistry;

/// Axum application state: Creator API client + per-printer actor registry.
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AppState {
    /// Low-level Creator `WebAPI` client (shared, thread-safe).
    pub client: Arc<dyn TepraClient>,
    /// Per-printer actor registry (lazy spawn on first use).
    pub registry: Arc<PrinterRegistry>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Construct a new `AppState` backed by `client`.
    pub fn new(client: Arc<dyn TepraClient>) -> Self {
        let registry = Arc::new(PrinterRegistry::new(Arc::clone(&client)));
        Self { client, registry }
    }
}
