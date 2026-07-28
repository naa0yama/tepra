//! `PrinterRegistry` — single-process map of printer name → `PrinterHandle`.

use std::sync::Arc;

use dashmap::DashMap;
use tepra_core::client::TepraClient;

use super::{PrinterActor, PrinterHandle};

/// Single-process registry mapping printer names to their running actors.
///
/// Guarantees at most one `PrinterActor` per printer name.
#[allow(clippy::module_name_repetitions)]
pub struct PrinterRegistry {
    client: Arc<dyn TepraClient>,
    actors: DashMap<String, Arc<PrinterHandle>>,
}

impl std::fmt::Debug for PrinterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrinterRegistry")
            .field("actors_count", &self.actors.len())
            .finish_non_exhaustive()
    }
}

impl PrinterRegistry {
    /// Create a new empty registry backed by `client`.
    pub fn new(client: Arc<dyn TepraClient>) -> Self {
        Self {
            client,
            actors: DashMap::new(),
        }
    }

    /// Return the handle for `name`, spawning a new actor if none exists yet.
    #[must_use]
    pub fn get_or_spawn(&self, name: &str) -> Arc<PrinterHandle> {
        let entry = self.actors.entry(name.to_owned()).or_insert_with(|| {
            Arc::new(PrinterActor::spawn(
                Arc::clone(&self.client),
                name.to_owned(),
            ))
        });
        Arc::clone(&entry)
    }

    /// Send the shutdown signal to every registered actor.
    pub async fn shutdown_all(self) {
        let handles: Vec<Arc<PrinterHandle>> = self.actors.into_iter().map(|(_, h)| h).collect();
        for h in handles {
            h.send_shutdown().await;
        }
    }
}
