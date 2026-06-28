//! View handlers — HTML page responses for the web UI (HTMX/DaisyUI).
#![allow(clippy::unused_async, clippy::module_name_repetitions)]

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    state::AppState,
    views::{IndexTemplate, JobCardTemplate, PrinterDetailTemplate},
};

/// `GET /ui/` — printer list index page.
pub async fn index(_state: State<AppState>) -> impl IntoResponse {
    IndexTemplate { printers: vec![] }
}

/// `GET /ui/printers/{name}` — per-printer detail page.
pub async fn printer_detail(
    Path(name): Path<String>,
    _state: State<AppState>,
) -> impl IntoResponse {
    PrinterDetailTemplate {
        printer_name: name,
        online: false,
    }
}

/// `GET /ui/jobs/{printer}/{job_id}` — HTMX job-card partial.
pub async fn job_card(
    Path((printer_name, job_id)): Path<(String, u64)>,
    _state: State<AppState>,
) -> impl IntoResponse {
    JobCardTemplate {
        printer_name,
        job_id,
        job_end: false,
        canceled: false,
        progress: None,
    }
}
