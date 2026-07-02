//! Shared application state threaded through all Axum handlers.

use std::{path::PathBuf, sync::Arc};

use tepra_core::{client::traits::TepraClient, otel::metrics::Meters};

use crate::actor::registry::PrinterRegistry;

/// Axum application state: Creator API client + per-printer actor registry.
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AppState {
    /// Low-level Creator `WebAPI` client (shared, thread-safe).
    pub client: Arc<dyn TepraClient>,
    /// Per-printer actor registry (lazy spawn on first use).
    pub registry: Arc<PrinterRegistry>,
    /// Directory that holds label template files served by `GET /api/templates`.
    pub template_dir: PathBuf,
    /// `OTel` metric instruments. `None` when `OTEL_EXPORTER_OTLP_ENDPOINT` is unset.
    meters: Option<Arc<Meters>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("registry", &self.registry)
            .field("template_dir", &self.template_dir)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Construct a new `AppState` backed by `client` using the given template directory.
    pub fn new_with_template_dir(client: Arc<dyn TepraClient>, template_dir: PathBuf) -> Self {
        let registry = Arc::new(PrinterRegistry::new(Arc::clone(&client)));
        Self {
            client,
            registry,
            template_dir,
            meters: None,
        }
    }

    /// Construct a new `AppState` backed by `client` with an empty template directory.
    pub fn new(client: Arc<dyn TepraClient>) -> Self {
        Self::new_with_template_dir(client, PathBuf::new())
    }

    /// Attach metric instruments to the state (builder pattern).
    #[must_use]
    pub fn with_meters(mut self, meters: Arc<Meters>) -> Self {
        self.meters = Some(meters);
        self
    }

    /// Return the metric instruments, if any.
    #[must_use]
    pub const fn meters(&self) -> Option<&Arc<Meters>> {
        self.meters.as_ref()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use std::sync::Arc;

    use tepra_core::client::MockTepraClient;

    use super::*;

    fn mock_client() -> Arc<dyn TepraClient> {
        Arc::new(MockTepraClient::new())
    }

    #[test]
    fn new_creates_state_with_empty_template_dir() {
        let state = AppState::new(mock_client());
        assert_eq!(state.template_dir, PathBuf::new());
    }

    #[test]
    fn new_with_template_dir_stores_path() {
        let path = PathBuf::from("/tmp/labels");
        let state = AppState::new_with_template_dir(mock_client(), path.clone());
        assert_eq!(state.template_dir, path);
    }

    #[test]
    fn clone_preserves_template_dir() {
        let path = PathBuf::from("/tmp/labels");
        let state = AppState::new_with_template_dir(mock_client(), path);
        let cloned = state.clone();
        assert_eq!(cloned.template_dir, state.template_dir);
    }

    #[test]
    fn debug_impl_contains_struct_name() {
        let state = AppState::new(mock_client());
        let s = format!("{state:?}");
        assert!(s.contains("AppState"));
    }

    #[test]
    fn with_meters_stores_meters_and_accessor_returns_some() {
        let meters = Arc::new(tepra_core::otel::metrics::Meters::default());
        let state = AppState::new(mock_client()).with_meters(Arc::clone(&meters));
        assert!(state.meters().is_some());
    }

    #[test]
    fn new_state_has_no_meters() {
        let state = AppState::new(mock_client());
        assert!(state.meters().is_none());
    }
}
