//! Handlers for template-related endpoints.
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::literal_string_with_formatting_args
)]

use axum::{Json, extract::State, http::StatusCode};
use opentelemetry_semantic_conventions::attribute as semconv;
use tepra_core::dto::template::{ImportFrameItem, ImportFrameRequest};
use tracing::{Span, instrument};

use super::err_502;
use crate::{state::AppState, templates::TemplateEntry};

/// `POST /api/printer/template/importframe` — extract frame list from a template file.
#[axum::debug_handler]
#[instrument(
    name = "handler.import_frame",
    skip_all,
    fields(
        http.request.method = "POST",
        http.route = "/api/printer/template/importframe",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn import_frame(
    State(state): State<AppState>,
    Json(req): Json<ImportFrameRequest>,
) -> Result<Json<Vec<ImportFrameItem>>, StatusCode> {
    let result = state
        .client
        .import_frame(req)
        .await
        .map(Json)
        .map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `GET /api/templates` — list template files in the configured template directory.
#[axum::debug_handler]
#[instrument(
    name = "handler.list_template_files",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/templates",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn list_template_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<TemplateEntry>>, StatusCode> {
    let result = crate::templates::list_templates(&state.template_dir)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 500_i64 },
    );
    result
}
