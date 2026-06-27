//! `PrinterActor` — per-printer single-worker FIFO job queue.
//!
//! T12a stub: types compile; all methods are `todo!()`.
//! T12b will replace the stubs with the real `tokio::sync::mpsc` implementation.
#![allow(clippy::todo)] // stub — real impl lives in T12b

use std::sync::Arc;

use tepra_core::{
    client::TepraClient,
    dto::job::{PrintRequest, PrintResponse},
    error::TepraError,
};
use tokio::sync::oneshot;

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// Messages dispatched to the per-printer worker task.
#[allow(dead_code)] // used in T12b worker loop
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
// PrinterHandle
// ---------------------------------------------------------------------------

/// Cloneable handle to a running `PrinterActor` worker task.
///
/// Obtained from [`PrinterActor::spawn`].
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct PrinterHandle {
    _private: (),
}

impl PrinterHandle {
    /// Submit a print job to the FIFO queue and await its result.
    ///
    /// # Errors
    /// Returns [`TepraError`] if the Creator API call fails.
    #[allow(clippy::unused_async)] // T12b will add mpsc send + oneshot recv
    pub async fn print(&self, _req: PrintRequest) -> Result<PrintResponse, TepraError> {
        todo!("T12b: implement PrinterHandle::print via mpsc send + oneshot recv")
    }

    /// Signal the worker to drain remaining jobs then exit, consuming the handle.
    #[allow(clippy::unused_async)] // T12b will add Msg::Shutdown send + task join
    pub async fn shutdown(self) {
        todo!("T12b: implement PrinterHandle::shutdown via Msg::Shutdown")
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
    pub fn spawn(_client: Arc<dyn TepraClient>, _printer_name: String) -> PrinterHandle {
        todo!("T12b: spawn tokio task with mpsc channel and return PrinterHandle")
    }
}
