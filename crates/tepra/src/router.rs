//! Axum router builder for tepra.
#![allow(clippy::module_name_repetitions)]

use std::sync::Arc;

use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};
use tepra_core::client::traits::TepraClient;

use crate::{
    handlers::{jobs, openapi, printers, templates, views},
    state::AppState,
};

/// Build the main API router, wiring all `/api/printer*` routes.
pub fn build_router(client: Arc<dyn TepraClient>) -> Router {
    Router::new()
        .route("/api/printer", get(printers::list_printers))
        .route("/api/printer/version", get(printers::version))
        .route("/api/printer/autoselect", get(printers::autoselect))
        .route("/api/printer/info/{name}", get(printers::printer_info))
        .route(
            "/api/printer/onlinestatus/{name}",
            get(printers::online_status),
        )
        .route("/api/printer/lwstatus/{name}", get(printers::lw_status))
        .route("/api/printer/getmargin/{name}", post(printers::get_margin))
        .route("/api/openapi.json", get(openapi::openapi_json))
        .with_state(client)
}

/// Build the jobs API router for job-related `/api/printer/*` routes.
pub fn build_jobs_router(state: AppState) -> Router {
    Router::new()
        .route("/api/printer/print/{name}", post(jobs::print))
        .route("/api/printer/tapefeed/{name}", get(jobs::tapefeed))
        .route("/api/printer/job/progress/{name}", get(jobs::job_progress))
        .route("/api/printer/job/info/{name}", get(jobs::job_info))
        .route("/api/printer/job/control/{name}", post(jobs::job_control))
        .with_state(state)
}

/// Build the templates API router.
pub fn build_templates_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/printer/template/importframe",
            post(templates::import_frame),
        )
        .route("/api/templates", get(templates::list_template_files))
        .with_state(state)
}

/// Build the web UI router (`/ui/*` routes, HTML + HTMX).
pub fn build_ui_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/ui/") }))
        .route("/ui/", get(views::index))
        .route("/ui/printers/{name}/status-card", get(views::status_card))
        .route("/ui/jobs/{printer}/{job_id}", get(views::job_card))
        .route("/ui/api", get(views::api_docs))
        .with_state(state)
}
