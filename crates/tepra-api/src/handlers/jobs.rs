//! Handlers for job-related `/api/printer/*` endpoints.
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use tepra_core::dto::job::{
    JobControlRequest, JobInfoResponse, JobProgressResponse, PrintRequest, PrintResponse,
};

use crate::state::AppState;

#[allow(dead_code)]
pub(crate) fn err_502(_: tepra_core::error::TepraError) -> StatusCode {
    StatusCode::BAD_GATEWAY
}

/// `POST /api/printer/print/{name}` — enqueue a print job via `PrinterActor`.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn print(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_req): Json<PrintRequest>,
) -> Result<Json<PrintResponse>, StatusCode> {
    todo!("T14d")
}

/// Query parameters for `GET /api/printer/tapefeed/{name}`.
#[derive(Debug, Deserialize)]
pub struct TapefeedQuery {
    /// Cut tape after feed when `true`.
    pub cutflag: bool,
}

/// `GET /api/printer/tapefeed/{name}?cutflag=<bool>` — advance tape.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn tapefeed(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Query(_q): Query<TapefeedQuery>,
) -> Result<StatusCode, StatusCode> {
    todo!("T14d")
}

/// Query parameters for job progress and info endpoints.
#[derive(Debug, Deserialize)]
pub struct JobIdQuery {
    /// Creator API job identifier returned by `/print`.
    pub jobid: u64,
}

/// `GET /api/printer/job/progress/{name}?jobid=N` — poll print job progress.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn job_progress(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Query(_q): Query<JobIdQuery>,
) -> Result<Json<JobProgressResponse>, StatusCode> {
    todo!("T14d")
}

/// `GET /api/printer/job/info/{name}?jobid=N` — Win32 job status bitmask.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn job_info(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Query(_q): Query<JobIdQuery>,
) -> Result<Json<JobInfoResponse>, StatusCode> {
    todo!("T14d")
}

/// `POST /api/printer/job/control/{name}` — pause / resume / cancel a job.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn job_control(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_req): Json<JobControlRequest>,
) -> Result<StatusCode, StatusCode> {
    todo!("T14d")
}
