//! Askama HTML templates for the web UI.
//!
//! Template files live under `templates/` (Askama default search path).

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Newtype that renders an askama template as an HTML response.
///
/// Required because askama 0.13+ removed framework integration crates.
#[derive(Debug)]
pub struct HtmlTemplate<T: Template>(pub T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        self.0.render().map_or_else(
            |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            |html| Html(html).into_response(),
        )
    }
}

// ---------------------------------------------------------------------------
// Index page — printer list
// ---------------------------------------------------------------------------

/// Sidebar section key for the printers page (`nav_active` field below).
///
/// Matched by string equality in `templates/components/sidebar.html`
/// (`{% if active == "printers" %}`); defined once here so the two handlers
/// that set `nav_active` (`handlers::views::index`, `::printer_detail`)
/// cannot drift out of sync with each other.
pub const NAV_PRINTERS: &str = "printers";

/// One entry in a navbar breadcrumb trail (`shells/dashboard.html`).
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    /// Display label.
    pub label: String,
    /// Link target; `None` renders the current page as plain text.
    pub href: Option<String>,
}

/// Context for the top-level index page (`GET /`).
#[derive(Debug, Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
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
#[template(path = "pages/printer_detail.html")]
pub struct PrinterDetailTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
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
#[template(path = "partials/job_card.html")]
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
