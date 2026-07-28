//! In-memory print job history.
//!
//! Ephemeral: all records are lost on process restart. See ADR 0011.

use std::{
    collections::VecDeque,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::{Deserialize, Serialize};

use crate::handlers::merge_print::MergePrintRequest;

/// Maximum records retained; the oldest entry is evicted once exceeded.
pub const MAX_RECORDS: usize = 100;

/// Default page size for [`JobStore::page`].
pub const DEFAULT_PAGE_SIZE: usize = 20;

/// Outcome of a submitted print job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
    /// Creator API accepted the job.
    Accepted {
        /// ID assigned to the enqueued job by the Creator API.
        jobid: u64,
    },
    /// Creator API call failed.
    Failed {
        /// Upstream error message (`Display` of the originating error).
        message: String,
    },
}

impl JobOutcome {
    /// Short label used as the `outcome` metric/log attribute value.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Accepted { .. } => "accepted",
            Self::Failed { .. } => "failed",
        }
    }
}

impl std::fmt::Display for JobOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted { jobid } => write!(f, "accepted(jobid={jobid})"),
            Self::Failed { message } => write!(f, "failed({message})"),
        }
    }
}

/// A single recorded print submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRecord {
    /// Monotonic internal ID (distinct from the Creator API `jobid`).
    pub record_id: u64,
    /// Printer name the job was submitted to.
    pub printer: String,
    /// Submission time, epoch seconds (UTC).
    pub submitted_at: u64,
    /// Template path used, copied from `request.template` for list display.
    pub template: String,
    /// Full request payload, kept for re-print and parameter display.
    pub request: MergePrintRequest,
    /// Submission outcome.
    pub outcome: JobOutcome,
}

/// In-memory, bounded, newest-first print job history.
///
/// Capped at [`MAX_RECORDS`] entries (oldest evicted on overflow). Ephemeral —
/// lost on process restart (see ADR 0011).
#[derive(Debug, Default)]
pub struct JobStore {
    inner: Mutex<VecDeque<JobRecord>>,
    next_id: AtomicU64,
}

impl JobStore {
    /// Create an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a submission and return its assigned `record_id`.
    ///
    /// `submitted_at` is taken as a parameter (rather than sampled internally)
    /// so tests can assert on a fixed value instead of `SystemTime::now()`.
    pub fn record(
        &self,
        printer: String,
        template: String,
        request: MergePrintRequest,
        outcome: JobOutcome,
        submitted_at: u64,
    ) -> u64 {
        let record_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let record = JobRecord {
            record_id,
            printer,
            submitted_at,
            template,
            request,
            outcome,
        };

        let mut records = self.lock_inner();
        records.push_front(record);
        if records.len() > MAX_RECORDS {
            records.pop_back();
        }
        record_id
    }

    /// Look up a single record by `record_id`.
    #[must_use]
    pub fn get(&self, record_id: u64) -> Option<JobRecord> {
        self.lock_inner()
            .iter()
            .find(|r| r.record_id == record_id)
            .cloned()
    }

    /// Return one newest-first page plus the total record count.
    ///
    /// `page` is 1-indexed; out-of-range pages clamp to an empty slice.
    #[must_use]
    pub fn page(&self, page: usize, per: usize) -> (Vec<JobRecord>, usize) {
        let records = self.lock_inner();
        let total = records.len();
        let per = per.max(1);
        let start = page.saturating_sub(1).saturating_mul(per);
        let items = records.iter().skip(start).take(per).cloned().collect();
        drop(records);
        (items, total)
    }

    // WHY-NOT: propagate the poison error — a job-history write racing a panic
    // elsewhere is best-effort bookkeeping, not a correctness-critical path;
    // recovering keeps the store usable instead of poisoning every future call.
    fn lock_inner(&self) -> std::sync::MutexGuard<'_, VecDeque<JobRecord>> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]

    use super::*;

    fn request() -> MergePrintRequest {
        MergePrintRequest {
            template: "label.lw1".to_owned(),
            ..Default::default()
        }
    }

    fn accepted(jobid: u64) -> JobOutcome {
        JobOutcome::Accepted { jobid }
    }

    #[test]
    fn record_then_get_round_trips() {
        let store = JobStore::new();
        let id = store.record(
            "printer1".to_owned(),
            "label.lw1".to_owned(),
            request(),
            accepted(42),
            1_000,
        );

        let got = store.get(id).unwrap();
        assert_eq!(got.record_id, id);
        assert_eq!(got.printer, "printer1");
        assert_eq!(got.outcome, accepted(42));
        assert_eq!(got.submitted_at, 1_000);
    }

    #[test]
    fn get_unknown_record_id_returns_none() {
        let store = JobStore::new();
        assert!(store.get(999).is_none());
    }

    #[test]
    fn record_id_is_monotonic() {
        let store = JobStore::new();
        let first = store.record("p".to_owned(), "t".to_owned(), request(), accepted(1), 0);
        let second = store.record("p".to_owned(), "t".to_owned(), request(), accepted(2), 0);
        assert!(second > first);
    }

    #[test]
    fn overflow_evicts_oldest_record() {
        let store = JobStore::new();
        for i in 0..u64::try_from(MAX_RECORDS).unwrap() {
            store.record("p".to_owned(), "t".to_owned(), request(), accepted(i), i);
        }
        // Store is now exactly full; the oldest record (jobid=0) is still present.
        let (all, total) = store.page(1, MAX_RECORDS);
        assert_eq!(total, MAX_RECORDS);
        assert_eq!(all.last().unwrap().outcome, accepted(0));

        // One more push must evict the oldest (jobid=0).
        store.record(
            "p".to_owned(),
            "t".to_owned(),
            request(),
            accepted(u64::try_from(MAX_RECORDS).unwrap()),
            0,
        );
        let (all, total) = store.page(1, MAX_RECORDS);
        assert_eq!(total, MAX_RECORDS);
        assert!(all.iter().all(|r| r.outcome != accepted(0)));
    }

    #[test]
    fn page_returns_newest_first_slice() {
        let store = JobStore::new();
        for i in 0..5_u64 {
            store.record("p".to_owned(), "t".to_owned(), request(), accepted(i), i);
        }
        let (page1, total) = store.page(1, 2);
        assert_eq!(total, 5);
        // Newest first: record for jobid=4 was pushed last.
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].outcome, accepted(4));
        assert_eq!(page1[1].outcome, accepted(3));
    }

    #[test]
    fn page_out_of_range_clamps_to_empty() {
        let store = JobStore::new();
        store.record("p".to_owned(), "t".to_owned(), request(), accepted(0), 0);
        let (page, total) = store.page(99, 20);
        assert!(page.is_empty());
        assert_eq!(total, 1);
    }

    #[test]
    fn label_matches_outcome_variant() {
        assert_eq!(accepted(1).label(), "accepted");
        assert_eq!(
            JobOutcome::Failed {
                message: "boom".to_owned()
            }
            .label(),
            "failed"
        );
    }
}
