//! `GET /api/openapi.json` — code-derived `OpenAPI` document for the built-in `/api/*` endpoints.
#![allow(clippy::module_name_repetitions)]

use axum::Json;
use opentelemetry_semantic_conventions::attribute as semconv;
use tepra_core::dto::{job, printer, template};
use tracing::{Span, instrument};
use utoipa::{OpenApi, openapi::OpenApi as OpenApiDoc};

use super::{jobs, printers, templates};
use crate::templates::TemplateEntry;

/// `OpenAPI` document aggregating every `/api/*` handler.
#[derive(OpenApi)]
#[openapi(
    paths(
        printers::list_printers,
        printers::version,
        printers::autoselect,
        printers::printer_info,
        printers::online_status,
        printers::lw_status,
        printers::get_margin,
        jobs::print,
        jobs::tapefeed,
        jobs::job_progress,
        jobs::job_info,
        jobs::job_control,
        templates::import_frame,
        templates::list_template_files,
    ),
    components(schemas(
        printer::PrinterListItem,
        printer::VersionResponse,
        printer::AutoselectResponse,
        printer::PrinterInfoResponse,
        printer::OnlineStatusResponse,
        printer::LwStatusResponse,
        template::GetMarginRequest,
        template::GetMarginResponse,
        job::PrintRequest,
        job::PrintResponse,
        job::JobProgressResponse,
        job::JobInfoResponse,
        job::JobControlRequest,
        template::ImportFrameRequest,
        template::ImportFrameItem,
        TemplateEntry,
    )),
    tags(
        (name = "printer", description = "Printer discovery and status"),
        (name = "job", description = "Print job lifecycle"),
        (name = "template", description = "Template file handling"),
    ),
)]
#[derive(Debug)]
pub struct ApiDoc;

/// `GET /api/openapi.json` — serve the code-derived `OpenAPI` document as JSON.
#[axum::debug_handler]
#[instrument(
    name = "handler.openapi_json",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/openapi.json",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn openapi_json() -> Json<OpenApiDoc> {
    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    Json(ApiDoc::openapi())
}
