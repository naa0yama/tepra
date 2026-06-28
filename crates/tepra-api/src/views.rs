//! Askama HTML templates for the web UI.
//!
//! Template files live under `templates/` (Askama default search path).
//! T15b creates the actual `.html` files; until then `cargo build` fails
//! with "no template found" — this is the intentional RED state for T15a.

use askama::Template;

// ---------------------------------------------------------------------------
// Index page — printer list
// ---------------------------------------------------------------------------

/// Context for the top-level index page (`GET /`).
#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Display names of all known printers.
    pub printers: Vec<String>,
    /// Creator API error message, if the API call failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Printer detail page
// ---------------------------------------------------------------------------

/// Context for the per-printer detail page (`GET /printers/{name}`).
#[derive(Debug, Template)]
#[template(path = "printer_detail.html")]
pub struct PrinterDetailTemplate {
    /// Printer identifier.
    pub printer_name: String,
    /// Whether the printer is currently reachable.
    pub online: bool,
    /// Creator API error message, if the API call failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Job card partial (HTMX polling target)
// ---------------------------------------------------------------------------

/// Context for the job-status card partial (`GET /jobs/{printer}/{id}`).
#[derive(Debug, Template)]
#[template(path = "job_card.html")]
pub struct JobCardTemplate {
    /// Printer that owns the job.
    pub printer_name: String,
    /// Job sequence ID (display only).
    pub job_id: u64,
    /// `true` when the job has finished (stops HTMX polling).
    pub job_end: bool,
    /// `true` when the job was canceled.
    pub canceled: bool,
    /// Print progress percentage (0–100), `None` while queued.
    pub progress: Option<u32>,
}
