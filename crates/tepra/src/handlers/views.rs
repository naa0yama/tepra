//! View handlers — HTML page responses for the web UI (HTMX/DaisyUI).
#![allow(
    clippy::module_name_repetitions,
    clippy::literal_string_with_formatting_args
)]

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use opentelemetry_semantic_conventions::attribute as semconv;
use serde::Deserialize;
use tepra_core::dto::tape_label::{tape_id_label, tape_kind_label};
use tracing::{Span, instrument, warn};
use utoipa::OpenApi as _;

use crate::{
    handlers::{
        merge_print::{MergePrintRequest, fetch_template_and_frames, merge_print},
        openapi::ApiDoc,
    },
    state::AppState,
    views::{
        ApiDocsTemplate, Breadcrumb, ErrorAlertTemplate, HtmlTemplate, IndexTemplate,
        JobCardTemplate, MergeFramesTemplate, NAV_API, NAV_PRINT, NAV_PRINTERS, PrintPageTemplate,
        PrinterStatusCardTemplate, build_endpoint_views,
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

/// `GET /ui/printers/{name}/status-card` — HTMX printer status-card partial.
///
/// Lazy-loaded by each card in the printer list index page so that one
/// offline/slow printer cannot block the rest of the grid from rendering.
#[instrument(
    name = "printer.status_card",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/printers/{name}/status-card",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
        printer.name = %name,
    )
)]
pub async fn status_card(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let online_result = state.client.online_status(&name).await;
    let lw_result = state.client.lw_status(&name).await;

    let (online, tape_width, tape_kind, error) = match (online_result, lw_result) {
        (Ok(online_resp), Ok(lw_resp)) => (
            online_resp.online,
            tape_id_label(lw_resp.tape_id),
            tape_kind_label(lw_resp.tape_kind),
            None,
        ),
        (online_result, lw_result) => {
            if let Err(err) = &online_result {
                warn!(printer_name = %name, error = %err, "failed to fetch online status");
            }
            if let Err(err) = &lw_result {
                warn!(printer_name = %name, error = %err, "failed to fetch lw status");
            }
            (
                false,
                String::new(),
                "不明",
                Some(CREATOR_API_ERROR.to_owned()),
            )
        }
    };

    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    HtmlTemplate(PrinterStatusCardTemplate {
        printer_name: name,
        online,
        tape_width,
        tape_kind,
        error,
    })
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

/// `GET /ui/print` — merge-print page: template picker, frame table,
/// print-settings form.
#[instrument(
    name = "handler.print_page",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/print",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn print_page(State(state): State<AppState>) -> impl IntoResponse {
    let (templates, error) = crate::templates::list_templates(&state.template_dir)
        .map_or_else(|e| (vec![], Some(e.to_string())), |items| (items, None));

    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    HtmlTemplate(PrintPageTemplate {
        nav_active: NAV_PRINT.to_owned(),
        breadcrumbs: vec![Breadcrumb {
            label: "Print".into(),
            href: None,
        }],
        templates,
        error,
    })
}

/// Query parameters for `GET /ui/print/frames`.
#[derive(Debug, Deserialize)]
pub struct PrintFramesQuery {
    /// Template path relative to the configured template directory.
    pub template: String,
}

/// `GET /ui/print/frames?template=<rel>` — HTMX frame-table + print-settings
/// form partial, lazy-loaded when the template selector changes.
#[instrument(
    name = "handler.print_frames",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/ui/print/frames",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
        tepra.template_path = %q.template,
    )
)]
pub async fn print_frames(
    State(state): State<AppState>,
    Query(q): Query<PrintFramesQuery>,
) -> impl IntoResponse {
    let (frames, error) = match fetch_template_and_frames(&state, &q.template).await {
        Ok((_, frames)) => (frames, None),
        Err(e) => (vec![], Some(e.to_string())),
    };

    Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
    HtmlTemplate(MergeFramesTemplate {
        template: q.template,
        frames,
        error,
    })
}

/// `POST /ui/print/{printer}` — submit the merge-print form.
///
/// Reuses the same `merge_print` orchestration as the REST API and returns
/// the job-card partial on success (or an error banner partial on failure).
#[instrument(
    name = "handler.print_submit",
    skip_all,
    fields(
        http.request.method = "POST",
        http.route = "/ui/print/{printer}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
        tepra.template_path = tracing::field::Empty,
        tepra.row_count = tracing::field::Empty,
    )
)]
pub async fn print_submit(
    State(state): State<AppState>,
    Path(printer): Path<String>,
    Json(req): Json<MergePrintRequest>,
) -> impl IntoResponse {
    Span::current().record("tepra.template_path", req.template.as_str());
    Span::current().record(
        "tepra.row_count",
        i64::try_from(req.rows.len()).unwrap_or(i64::MAX),
    );
    match merge_print(&state, &printer, req).await {
        Ok(resp) => {
            Span::current().record(semconv::HTTP_RESPONSE_STATUS_CODE, 200_i64);
            HtmlTemplate(JobCardTemplate {
                printer_name: printer,
                job_id: resp.jobid,
                job_end: false,
                canceled: false,
                progress: None,
            })
            .into_response()
        }
        Err(e) => {
            warn!(error = %e, "merge-print submit failed");
            // WHY-NOT: record the actual wire status (always 200) — this
            // records the semantic outcome for observability; the response
            // itself stays 200 so htmx can swap the error partial into the DOM.
            Span::current().record(
                semconv::HTTP_RESPONSE_STATUS_CODE,
                i64::from(e.status().as_u16()),
            );
            HtmlTemplate(ErrorAlertTemplate {
                message: e.to_string(),
            })
            .into_response()
        }
    }
}
