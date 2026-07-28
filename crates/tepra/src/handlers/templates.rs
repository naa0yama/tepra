//! Handlers for template-related endpoints.
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::literal_string_with_formatting_args
)]

use anyhow::Context as _;
use axum::{
    Json,
    extract::{Query, State},
    http::{StatusCode, header},
};
use opentelemetry_semantic_conventions::attribute as semconv;
use serde::Deserialize;
use tepra_core::dto::template::{ImportFrameItem, ImportFrameRequest};
use tracing::{Span, instrument};

use super::err_502;
use crate::{
    state::AppState,
    templates::{TemplateEntry, resolve_template_path},
};

/// `POST /api/printer/template/importframe` — extract frame list from a template file.
#[utoipa::path(
    post,
    path = "/api/printer/template/importframe",
    tag = "template",
    request_body = ImportFrameRequest,
    responses(
        (status = 200, description = "Frame list extracted from template", body = Vec<ImportFrameItem>),
        (status = 502, description = "Creator API error"),
    ),
)]
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

/// `GET /api/rest/templates` — list template files in the configured template directory.
#[utoipa::path(
    get,
    path = "/api/rest/templates",
    tag = "template",
    responses(
        (status = 200, description = "Template files in the configured directory", body = Vec<TemplateEntry>),
        (status = 500, description = "Template directory read error"),
    ),
)]
#[axum::debug_handler]
#[instrument(
    name = "handler.list_template_files",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/rest/templates",
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

/// Query parameters for `GET /api/rest/templates/preview`.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TemplatePreviewQuery {
    /// Template path relative to the configured template directory.
    pub path: String,
}

/// BMP content-type header paired with the image bytes.
type BmpResponse = ([(header::HeaderName, &'static str); 1], Vec<u8>);

/// `GET /api/rest/templates/preview?path=<rel>` — extract the leading BMP
/// segment (self-contained, `bfSize` bytes) from a `.lw1` template as a
/// static reference image (frame positions/names, unrotated).
#[utoipa::path(
    get,
    path = "/api/rest/templates/preview",
    tag = "template",
    params(TemplatePreviewQuery),
    responses(
        (status = 200, description = "BMP preview image", content_type = "image/bmp"),
        (status = 404, description = "Template not found"),
    ),
)]
#[axum::debug_handler]
#[instrument(
    name = "handler.template_preview",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/rest/templates/preview",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
        tepra.template_path = tracing::field::Empty,
    )
)]
pub async fn template_preview(
    State(state): State<AppState>,
    Query(q): Query<TemplatePreviewQuery>,
) -> Result<BmpResponse, StatusCode> {
    Span::current().record("tepra.template_path", q.path.as_str());
    let result = extract_bmp_preview(&state.template_dir, &q.path);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 404_i64 },
    );
    result
        .map(|bmp| ([(header::CONTENT_TYPE, "image/bmp")], bmp))
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Reads `rel` under `dir` and slices out the leading BMP segment (`bfSize` bytes).
fn extract_bmp_preview(dir: &std::path::Path, rel: &str) -> anyhow::Result<Vec<u8>> {
    let path = resolve_template_path(dir, rel)?;
    let bytes = std::fs::read(&path)
        .with_context(|| format!("failed to read template: {}", path.display()))?;
    let header_bytes: &[u8; 6] = bytes
        .get(..6)
        .and_then(|s| s.try_into().ok())
        .with_context(|| format!("template too small for a BMP header: {}", path.display()))?;
    anyhow::ensure!(
        header_bytes[0..2] == *b"BM",
        "template does not start with a BMP header: {}",
        path.display()
    );
    let bf_size_bytes: [u8; 4] = header_bytes[2..6]
        .try_into()
        .with_context(|| "unreachable: header_bytes has exactly 6 bytes".to_owned())?;
    let bf_size = usize::try_from(u32::from_le_bytes(bf_size_bytes))
        .with_context(|| "unreachable: usize is at least 32-bit on supported targets".to_owned())?;
    let bmp = bytes
        .get(..bf_size)
        .with_context(|| {
            format!(
                "BMP bfSize ({bf_size}) exceeds file length ({}): {}",
                bytes.len(),
                path.display()
            )
        })?
        .to_vec();
    Ok(bmp)
}
