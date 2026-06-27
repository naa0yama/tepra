//! `PrinterActor` — per-printer single-worker FIFO job queue.

use std::sync::Arc;

use tepra_core::{
    client::TepraClient,
    dto::job::{PrintRequest, PrintResponse},
    error::TepraError,
};
use tokio::sync::{mpsc, oneshot};

use super::job::{JobId, JobState};

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// Messages dispatched to the per-printer worker task.
#[allow(clippy::large_enum_variant)] // PrintRequest is boxed; Shutdown is ZST
#[allow(dead_code)] // Submit/Cancel/Status fields are stubs; T12d will read them
#[derive(Debug)]
pub(crate) enum Msg {
    /// Enqueue a print job; reply channel returns the Creator API response.
    Print {
        req: Box<PrintRequest>,
        reply: oneshot::Sender<Result<PrintResponse, TepraError>>,
    },
    /// Submit a job to the FIFO queue; reply carries the actor-assigned [`JobId`].
    Submit {
        req: Box<PrintRequest>,
        reply: oneshot::Sender<Result<JobId, TepraError>>,
    },
    /// Cancel the job identified by `jobid`.
    Cancel {
        jobid: JobId,
        reply: oneshot::Sender<Result<(), TepraError>>,
    },
    /// Query the currently executing job's actor-assigned ID.
    CurrentJob {
        reply: oneshot::Sender<Option<JobId>>,
    },
    /// Query the state of a previously submitted job.
    Status {
        jobid: JobId,
        reply: oneshot::Sender<Option<JobState>>,
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
            Msg::Submit { reply, .. } => {
                // T12d: track job in queue and return actor-assigned JobId.
                let _ = reply.send(Err(TepraError::ActorShutdown));
            }
            Msg::Cancel { reply, .. } => {
                // T12d: implement cancel.
                let _ = reply.send(Err(TepraError::ActorShutdown));
            }
            Msg::CurrentJob { reply } => {
                // T12d: return current in-flight JobId.
                let _ = reply.send(None);
            }
            Msg::Status { reply, .. } => {
                // T12d: look up job state.
                let _ = reply.send(None);
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

    /// Submit a print job without awaiting its completion; returns an actor-assigned [`JobId`].
    ///
    /// # Errors
    /// Returns [`TepraError`] if the worker has shut down.
    pub async fn submit(&self, req: PrintRequest) -> Result<JobId, TepraError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = Msg::Submit {
            req: Box::new(req),
            reply: reply_tx,
        };
        if self.tx.send(msg).await.is_err() {
            return Err(TepraError::ActorShutdown);
        }
        reply_rx.await.map_err(|_| TepraError::ActorShutdown)?
    }

    /// Cancel the job identified by `jobid`.
    ///
    /// Returns `Ok(())` if the cancellation was accepted (or the job was already done).
    ///
    /// # Errors
    /// Returns [`TepraError`] if the worker has shut down or the jobid is unknown.
    pub async fn cancel(&self, jobid: JobId) -> Result<(), TepraError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = Msg::Cancel {
            jobid,
            reply: reply_tx,
        };
        if self.tx.send(msg).await.is_err() {
            return Err(TepraError::ActorShutdown);
        }
        reply_rx.await.map_err(|_| TepraError::ActorShutdown)?
    }

    /// Return the actor-assigned [`JobId`] of the job currently being submitted, if any.
    pub async fn current_job(&self) -> Option<JobId> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = Msg::CurrentJob { reply: reply_tx };
        if self.tx.send(msg).await.is_err() {
            return None;
        }
        reply_rx.await.ok().flatten()
    }

    /// Return the [`JobState`] of a previously submitted job, or `None` if unknown.
    pub async fn status(&self, jobid: JobId) -> Option<JobState> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = Msg::Status {
            jobid,
            reply: reply_tx,
        };
        if self.tx.send(msg).await.is_err() {
            return None;
        }
        reply_rx.await.ok().flatten()
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
