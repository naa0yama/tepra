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

use super::err_502;
use crate::state::AppState;

/// `POST /api/printer/print/{name}` — enqueue a print job via the Creator API.
#[axum::debug_handler]
pub async fn print(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<PrintRequest>,
) -> Result<Json<PrintResponse>, StatusCode> {
    state
        .client
        .print(&name, req)
        .await
        .map(Json)
        .map_err(err_502)
}

/// Query parameters for `GET /api/printer/tapefeed/{name}`.
#[derive(Debug, Deserialize)]
pub struct TapefeedQuery {
    /// Cut tape after feed when `true`.
    pub cutflag: bool,
}

/// `GET /api/printer/tapefeed/{name}?cutflag=<bool>` — advance tape.
#[axum::debug_handler]
pub async fn tapefeed(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(q): Query<TapefeedQuery>,
) -> Result<StatusCode, StatusCode> {
    state
        .client
        .tapefeed(&name, q.cutflag)
        .await
        .map(|()| StatusCode::OK)
        .map_err(err_502)
}

/// Query parameters for job progress and info endpoints.
#[derive(Debug, Deserialize)]
pub struct JobIdQuery {
    /// Creator API job identifier returned by `/print`.
    pub jobid: u64,
}

/// `GET /api/printer/job/progress/{name}?jobid=N` — poll print job progress.
#[axum::debug_handler]
pub async fn job_progress(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(q): Query<JobIdQuery>,
) -> Result<Json<JobProgressResponse>, StatusCode> {
    state
        .client
        .job_progress(&name, q.jobid)
        .await
        .map(Json)
        .map_err(err_502)
}

/// `GET /api/printer/job/info/{name}?jobid=N` — Win32 job status bitmask.
#[axum::debug_handler]
pub async fn job_info(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(q): Query<JobIdQuery>,
) -> Result<Json<JobInfoResponse>, StatusCode> {
    state
        .client
        .job_info(&name, q.jobid)
        .await
        .map(Json)
        .map_err(err_502)
}

/// `POST /api/printer/job/control/{name}` — pause / resume / cancel a job.
#[axum::debug_handler]
pub async fn job_control(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<JobControlRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .client
        .job_control(&name, req)
        .await
        .map(|()| StatusCode::OK)
        .map_err(err_502)
}
