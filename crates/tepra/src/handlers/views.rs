//! View handlers — HTML page responses for the web UI (HTMX/DaisyUI).
#![allow(
    clippy::module_name_repetitions,
    clippy::literal_string_with_formatting_args
)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use opentelemetry_semantic_conventions::attribute as semconv;
use tracing::{Span, instrument};
use utoipa::OpenApi as _;

use crate::{
    handlers::openapi::ApiDoc,
    state::AppState,
    views::{
        ApiDocsTemplate, Breadcrumb, HtmlTemplate, IndexTemplate, JobCardTemplate, NAV_API,
        NAV_PRINTERS, PrinterDetailTemplate, build_endpoint_views,
    },
};

const CREATOR_API_ERROR: &str = "Cannot connect to TEPRA Creator WebAPI";
const API_DOC_SERIALIZE_ERROR: &str = "Failed to build the OpenAPI document";

/// `GET /ui/` — printer list index page.
#[instrument(
    name = "handler.index",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let result = state.client.list_printers().await;
    let (printers, error) = result.map_or_else(
        |_| (vec![], Some(CREATOR_API_ERROR.to_owned())),
        |items| (items.into_iter().map(|p| p.printer_name).collect(), None),
    );
    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    HtmlTemplate(IndexTemplate {
        nav_active: NAV_PRINTERS.to_owned(),
        breadcrumbs: vec![Breadcrumb {
            label: "Printers".into(),
            href: None,
        }],
        printers,
        error,
    })
}

/// `GET /ui/printers/{name}` — per-printer detail page.
#[instrument(
    name = "handler.printer_detail",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/printers/{name}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn printer_detail(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = state.client.online_status(&name).await;
    let (online, error) = result.map_or_else(
        |_| (false, Some(CREATOR_API_ERROR.to_owned())),
        |resp| (resp.online, None),
    );
    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    HtmlTemplate(PrinterDetailTemplate {
        nav_active: NAV_PRINTERS.to_owned(),
        breadcrumbs: vec![
            Breadcrumb {
                label: "Printers".into(),
                href: Some("/ui/".into()),
            },
            Breadcrumb {
                label: name.clone(),
                href: None,
            },
        ],
        printer_name: name,
        online,
        error,
    })
}

/// `GET /ui/jobs/{printer}/{job_id}` — HTMX job-card partial.
///
/// # Errors
///
/// Returns `502 Bad Gateway` when the Creator API client fails.
#[instrument(
    name = "handler.job_card",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/jobs/{printer}/{job_id}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
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

    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    Ok(HtmlTemplate(JobCardTemplate {
        printer_name,
        job_id,
        job_end: resp.job_end,
        canceled: resp.canceled,
        progress,
    }))
}

/// `GET /ui/api` — read-only API reference page listing every built-in
/// `/api/*` endpoint with its request/response schema.
#[instrument(
    name = "handler.api_docs",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/api",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn api_docs() -> impl IntoResponse {
    let (endpoints, error) = serde_json::to_value(ApiDoc::openapi()).map_or_else(
        |_| (Vec::new(), Some(API_DOC_SERIALIZE_ERROR.to_owned())),
        |openapi| (build_endpoint_views(&openapi), None),
    );

    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    HtmlTemplate(ApiDocsTemplate {
        nav_active: NAV_API.to_owned(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints,
        error,
    })
}
