//! Job types for the per-printer actor queue.

/// Opaque identifier for a print job assigned by the actor (monotonically increasing).
#[allow(clippy::module_name_repetitions)]
pub type JobId = u64;

/// Lifecycle status of a queued print job.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum JobStatus {
    /// Waiting in the FIFO queue; not yet submitted to the printer.
    Pending,
    /// Currently being submitted to the Creator API.
    InProgress,
    /// Creator API accepted the job successfully.
    Completed,
    /// Creator API returned an error or the actor encountered an I/O failure.
    Failed(String),
    /// Cancelled by a call to [`PrinterHandle::cancel`].
    Canceled,
}

/// Snapshot of a single print job's state.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct JobState {
    /// Actor-assigned identifier.
    pub id: JobId,
    /// Current lifecycle status.
    pub status: JobStatus,
}
