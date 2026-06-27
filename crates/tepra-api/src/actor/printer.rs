//! `PrinterActor` — per-printer single-worker FIFO job queue.

use std::sync::Arc;

use tepra_core::{
    client::TepraClient,
    dto::job::{PrintRequest, PrintResponse},
    error::TepraError,
};
use tokio::sync::{mpsc, oneshot};

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// Messages dispatched to the per-printer worker task.
#[allow(clippy::large_enum_variant)] // PrintRequest is boxed; Shutdown is ZST
#[derive(Debug)]
pub(crate) enum Msg {
    /// Enqueue a print job; reply channel returns the Creator API response.
    Print {
        req: Box<PrintRequest>,
        reply: oneshot::Sender<Result<PrintResponse, TepraError>>,
    },
    /// Drain the queue and terminate the worker task.
    Shutdown,
}

// ---------------------------------------------------------------------------
// Worker loop
// ---------------------------------------------------------------------------

async fn run_worker(
    client: Arc<dyn TepraClient>,
    printer_name: String,
    mut rx: mpsc::Receiver<Msg>,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            Msg::Print { req, reply } => {
                let result = client.print(&printer_name, *req).await;
                // Ignore send error: caller may have dropped the receiver (timeout).
                let _ = reply.send(result);
            }
            Msg::Shutdown => {
                // Drain remaining messages without processing, then exit.
                rx.close();
                while rx.recv().await.is_some() {}
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// PrinterHandle
// ---------------------------------------------------------------------------

/// Cloneable handle to a running `PrinterActor` worker task.
///
/// Obtained from [`PrinterActor::spawn`].
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct PrinterHandle {
    tx: mpsc::Sender<Msg>,
    task: tokio::task::JoinHandle<()>,
}

impl PrinterHandle {
    /// Submit a print job to the FIFO queue and await its result.
    ///
    /// # Errors
    /// Returns [`TepraError`] if the Creator API call fails or the worker has shut down.
    pub async fn print(&self, req: PrintRequest) -> Result<PrintResponse, TepraError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = Msg::Print {
            req: Box::new(req),
            reply: reply_tx,
        };
        if self.tx.send(msg).await.is_err() {
            return Err(TepraError::ActorShutdown);
        }
        reply_rx.await.map_err(|_| TepraError::ActorShutdown)?
    }

    /// Signal the worker to drain remaining jobs then exit, consuming the handle.
    pub async fn shutdown(self) {
        // Best-effort: if the channel is already closed, the worker already exited.
        let _ = self.tx.send(Msg::Shutdown).await;
        let _ = self.task.await;
    }
}

// ---------------------------------------------------------------------------
// PrinterActor
// ---------------------------------------------------------------------------

/// Spawns and owns a per-printer tokio task that processes jobs one at a time.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct PrinterActor;

impl PrinterActor {
    /// Spawn a worker task for `printer_name` backed by `client`, returning a [`PrinterHandle`].
    pub fn spawn(client: Arc<dyn TepraClient>, printer_name: String) -> PrinterHandle {
        let (tx, rx) = mpsc::channel(64);
        let task = tokio::spawn(run_worker(client, printer_name, rx));
        PrinterHandle { tx, task }
    }
}
