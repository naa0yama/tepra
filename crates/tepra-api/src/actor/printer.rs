//! `PrinterActor` — per-printer single-worker FIFO job queue.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use tepra_core::{
    client::TepraClient,
    dto::job::{PrintRequest, PrintResponse},
    error::TepraError,
};
use tokio::sync::{mpsc, oneshot};

use super::job::{JobId, JobState, JobStatus};

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// Messages dispatched to the per-printer worker task.
#[allow(clippy::large_enum_variant)]
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
// Worker state
// ---------------------------------------------------------------------------

struct WorkerState {
    queue: VecDeque<(JobId, Box<PrintRequest>)>,
    jobs: HashMap<JobId, JobState>,
    next_id: JobId,
    current: Option<JobId>,
    cancelled: HashSet<JobId>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            jobs: HashMap::new(),
            next_id: 1,
            current: None,
            cancelled: HashSet::new(),
        }
    }

    fn enqueue(&mut self, req: Box<PrintRequest>) -> JobId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.jobs.insert(
            id,
            JobState {
                id,
                status: JobStatus::Pending,
            },
        );
        self.queue.push_back((id, req));
        id
    }

    /// Cancel a pending (queued) job. Returns true if found and cancelled.
    fn cancel_pending(&mut self, jobid: JobId) -> bool {
        if let Some(pos) = self.queue.iter().position(|(id, _)| *id == jobid) {
            self.queue.remove(pos);
            if let Some(js) = self.jobs.get_mut(&jobid) {
                js.status = JobStatus::Canceled;
            }
            true
        } else {
            false
        }
    }

    /// Pop the next non-cancelled job from the queue, skipping any that were
    /// cancelled while waiting.
    fn pop_next_job(&mut self) -> Option<(JobId, Box<PrintRequest>)> {
        loop {
            let &(front_id, _) = self.queue.front()?;
            if self.cancelled.remove(&front_id) {
                self.queue.pop_front();
                if let Some(js) = self.jobs.get_mut(&front_id) {
                    js.status = JobStatus::Canceled;
                }
            } else {
                return self.queue.pop_front();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Worker loop
// ---------------------------------------------------------------------------

/// Handle a non-Print message in the idle state. Returns true if the worker
/// should shut down.
fn handle_idle_msg(msg: Msg, state: &mut WorkerState) -> bool {
    match msg {
        Msg::Submit { req, reply } => {
            let id = state.enqueue(req);
            let _ = reply.send(Ok(id));
        }
        Msg::Cancel { jobid, reply } => {
            state.cancel_pending(jobid);
            let _ = reply.send(Ok(()));
        }
        Msg::CurrentJob { reply } => {
            let _ = reply.send(state.current);
        }
        Msg::Status { jobid, reply } => {
            let _ = reply.send(state.jobs.get(&jobid).cloned());
        }
        Msg::Shutdown => return true,
        Msg::Print { reply, .. } => {
            // Print via direct path is handled in the caller; this branch is unreachable.
            let _ = reply.send(Err(TepraError::ActorShutdown));
        }
    }
    false
}

async fn run_worker(
    client: Arc<dyn TepraClient>,
    printer_name: String,
    mut rx: mpsc::Receiver<Msg>,
) {
    let mut state = WorkerState::new();

    loop {
        if let Some((jobid, req)) = state.pop_next_job() {
            // --- Executing a queued job ---
            state.current = Some(jobid);
            if let Some(js) = state.jobs.get_mut(&jobid) {
                js.status = JobStatus::InProgress;
            }

            let print_fut = client.print(&printer_name, *req);
            tokio::pin!(print_fut);

            // Run job while accepting incoming messages. If cancel arrives for
            // THIS job, drop print_fut (break None) so inner.print is never called.
            let result = 'job: loop {
                tokio::select! {
                    r = &mut print_fut => break 'job Some(r),
                    msg = rx.recv() => {
                        match msg {
                            None => return,
                            Some(Msg::Cancel { jobid: cid, reply }) => {
                                if cid == jobid {
                                    // Abort the in-flight print by breaking (drops print_fut).
                                    state.cancelled.insert(cid);
                                    let _ = reply.send(Ok(()));
                                    break 'job None;
                                }
                                // Cancel a pending or unknown job.
                                state.cancel_pending(cid);
                                let _ = reply.send(Ok(()));
                            }
                            Some(Msg::Submit { req, reply }) => {
                                let id = state.enqueue(req);
                                let _ = reply.send(Ok(id));
                            }
                            Some(Msg::CurrentJob { reply }) => {
                                let _ = reply.send(state.current);
                            }
                            Some(Msg::Status { jobid: sid, reply }) => {
                                let _ = reply.send(state.jobs.get(&sid).cloned());
                            }
                            Some(Msg::Shutdown) => {
                                rx.close();
                                return;
                            }
                            Some(Msg::Print { reply, .. }) => {
                                // Direct print while queue is running is not supported.
                                let _ = reply.send(Err(TepraError::ActorShutdown));
                            }
                        }
                    }
                }
            };

            state.current = None;
            let was_cancelled = state.cancelled.remove(&jobid) || result.is_none();
            if let Some(js) = state.jobs.get_mut(&jobid) {
                js.status = if was_cancelled {
                    JobStatus::Canceled
                } else if let Some(r) = result {
                    match r {
                        Ok(_) => JobStatus::Completed,
                        Err(e) => JobStatus::Failed(e.to_string()),
                    }
                } else {
                    JobStatus::Canceled
                };
            }
        } else {
            // --- Idle: no queued jobs ---
            match rx.recv().await {
                None => return,
                Some(Msg::Print { req, reply }) => {
                    // Direct (non-queued) print: used by handle.print().
                    let result = client.print(&printer_name, *req).await;
                    let _ = reply.send(result);
                }
                Some(msg) => {
                    if handle_idle_msg(msg, &mut state) {
                        return;
                    }
                }
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
        let _ = self.tx.send(Msg::Shutdown).await;
        let _ = self.task.await;
    }

    /// Send the shutdown signal without consuming the handle or joining the task.
    pub(super) async fn send_shutdown(&self) {
        let _ = self.tx.send(Msg::Shutdown).await;
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
    /// Returns [`TepraError`] if the worker has shut down.
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
