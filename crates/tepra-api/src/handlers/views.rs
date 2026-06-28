//! View handlers — HTML page responses for the web UI (HTMX/DaisyUI).
#![allow(clippy::module_name_repetitions)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    state::AppState,
    views::{IndexTemplate, JobCardTemplate, PrinterDetailTemplate},
};

/// `GET /ui/` — printer list index page.
///
/// # Errors
///
/// Returns `502 Bad Gateway` when the Creator API client fails.
pub async fn index(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let items = state
        .client
        .list_printers()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let printers = items.into_iter().map(|p| p.printer_name).collect();
    Ok(IndexTemplate { printers })
}

/// `GET /ui/printers/{name}` — per-printer detail page.
///
/// # Errors
///
/// Returns `502 Bad Gateway` when the Creator API client fails.
pub async fn printer_detail(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let resp = state
        .client
        .online_status(&name)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(PrinterDetailTemplate {
        printer_name: name,
        online: resp.online,
    })
}

/// `GET /ui/jobs/{printer}/{job_id}` — HTMX job-card partial.
///
/// # Errors
///
/// Returns `502 Bad Gateway` when the Creator API client fails.
pub async fn job_card(
    Path((printer_name, job_id)): Path<(String, u64)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let resp = state
        .client
        .job_progress(&printer_name, job_id)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let progress = if resp.job_end || resp.canceled {
        None
    } else {
        Some(resp.data_progress)
    };

    Ok(JobCardTemplate {
        printer_name,
        job_id,
        job_end: resp.job_end,
        canceled: resp.canceled,
        progress,
    })
}
