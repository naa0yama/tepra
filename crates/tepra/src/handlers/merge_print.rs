//! Handler for `POST /api/rest/merge-print/{printer}` — orchestrates a
//! template + CSV merge print job from a minimal, `curl`-friendly body.
// WHY-NOT: rename types/fns to drop the `MergePrint`/`merge_print` prefix —
// matches sibling `merge.rs` naming convention (`MergeField`, `merge_print_parameter`).
#![allow(clippy::module_name_repetitions)]
// WHY-NOT: add `# Errors` docs to every fallible fn — variants are already
// documented on `MergePrintError`; per-fn docs would just repeat that enum.
#![allow(clippy::missing_errors_doc)]
// WHY-NOT: rename `http.route` field literal — clippy misreads the route's
// `{printer}` path segment as a format-string placeholder; it is a literal.
#![allow(clippy::literal_string_with_formatting_args)]

use anyhow::Context as _;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use opentelemetry_semantic_conventions::attribute as semconv;
use serde::{Deserialize, Serialize};
use tepra_core::{
    dto::{
        job::{FilePayload, PrintFiles, PrintRequest, PrintResponse},
        template::{ImportFrameItem, ImportFrameRequest},
    },
    error::TepraError,
};
use tracing::{Span, instrument};

use crate::{
    merge::{
        CsvEncoding, MergeField, MergePrintOverrides, SerialSpec, build_merge_csv, expand_serial,
        merge_print_parameter, sort_frames_by_column,
    },
    state::AppState,
    templates::resolve_template_path,
};

/// Request body for `POST /api/rest/merge-print/{printer}`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MergePrintRequest {
    /// Template file path, relative to the configured template directory.
    pub template: String,
    /// One entry per tape (label); each entry is that tape's `{title, value}` fields.
    pub rows: Vec<Vec<MergeField>>,
    /// Optional serial-number generation, merged into `rows` per tape.
    pub serial: Option<SerialSpec>,
    /// Print settings; unset fields keep the SDK default.
    #[serde(flatten)]
    pub overrides: MergePrintOverrides,
}

/// Failure modes of the merge-print orchestration, mapped 1:1 to HTTP status
/// codes by the handler.
#[derive(Debug)]
pub(crate) enum MergePrintError {
    /// Template file not found (also covers path-traversal attempts).
    TemplateNotFound(anyhow::Error),
    /// Malformed `rows`/`serial` (unknown/duplicate title, or row/serial count mismatch).
    BadRequest(anyhow::Error),
    /// Creator `WebAPI` call failed.
    Upstream(TepraError),
}

impl MergePrintError {
    pub(crate) const fn status(&self) -> StatusCode {
        match self {
            Self::TemplateNotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl std::fmt::Display for MergePrintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateNotFound(e) | Self::BadRequest(e) => write!(f, "{e}"),
            Self::Upstream(e) => write!(f, "{e}"),
        }
    }
}

/// `POST /api/rest/merge-print/{printer}` — merge `rows`/`serial` data into
/// `template` and submit the print job.
#[utoipa::path(
    post,
    path = "/api/rest/merge-print/{printer}",
    tag = "merge-print",
    params(
        ("printer" = String, Path, description = "Printer name"),
    ),
    request_body = MergePrintRequest,
    responses(
        (status = 200, description = "Print job enqueued", body = PrintResponse),
        (status = 400, description = "Unknown/duplicate title, or rows/serial count mismatch"),
        (status = 404, description = "Template not found"),
        (status = 502, description = "Creator API error"),
    ),
)]
#[axum::debug_handler]
#[instrument(
    name = "handler.merge_print",
    skip_all,
    fields(
        http.request.method = "POST",
        http.route = "/api/rest/merge-print/{printer}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
        tepra.template_path = tracing::field::Empty,
        tepra.row_count = tracing::field::Empty,
    )
)]
pub async fn merge_print_handler(
    State(state): State<AppState>,
    Path(printer): Path<String>,
    Json(req): Json<MergePrintRequest>,
) -> Result<Json<PrintResponse>, StatusCode> {
    Span::current().record("tepra.template_path", req.template.as_str());
    Span::current().record(
        "tepra.row_count",
        i64::try_from(req.rows.len()).unwrap_or(i64::MAX),
    );
    let result = merge_print(&state, &printer, req).await;
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        i64::from(result.as_ref().map_or_else(
            |e: &MergePrintError| e.status().as_u16(),
            |_| StatusCode::OK.as_u16(),
        )),
    );
    result.map(Json).map_err(|e| {
        match &e {
            MergePrintError::TemplateNotFound(err) => {
                tracing::warn!(error = %err, "merge-print template not found");
            }
            MergePrintError::BadRequest(err) => {
                tracing::warn!(error = %err, "merge-print bad request");
            }
            MergePrintError::Upstream(err) => {
                tracing::warn!(error = %err, "merge-print upstream error");
            }
        }
        e.status()
    })
}

/// Reads `template_rel` under the configured template directory and calls
/// `import_frame` on it, returning both the encoded file payload (reused for
/// the print request) and the extracted frame list.
///
/// Extracted so the `/ui/print/frames` handler can reuse the same logic
/// without re-reading the template a second time in `merge_print`.
pub(crate) async fn fetch_template_and_frames(
    state: &AppState,
    template_rel: &str,
) -> Result<(FilePayload, Vec<ImportFrameItem>), MergePrintError> {
    let template_path = resolve_template_path(&state.template_dir, template_rel)
        .map_err(MergePrintError::TemplateNotFound)?;
    let read_result = std::fs::read(&template_path)
        .with_context(|| format!("failed to read template: {}", template_path.display()));
    let template_bytes = read_result.map_err(MergePrintError::TemplateNotFound)?;
    let file_name = template_path.file_name().map_or_else(
        || template_rel.to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    let template_file = FilePayload {
        file_name,
        base64_str: STANDARD.encode(&template_bytes),
    };

    let mut frames = state
        .client
        .import_frame(ImportFrameRequest {
            template_file: template_file.clone(),
        })
        .await
        .map_err(MergePrintError::Upstream)?;
    // import_frame's array order need not match cell reference order; the
    // printer binds CSV columns to frames by cell reference, so normalize
    // here once for both CSV assembly and UI rendering downstream.
    sort_frames_by_column(&mut frames);

    Ok((template_file, frames))
}

/// Shared orchestration: template read → `import_frame` → CSV build → `print`.
///
/// Extracted so the `/ui/print` form handler can reuse the same logic.
pub(crate) async fn merge_print(
    state: &AppState,
    printer: &str,
    req: MergePrintRequest,
) -> Result<PrintResponse, MergePrintError> {
    let (template_file, frames) = fetch_template_and_frames(state, &req.template).await?;

    let rows = merge_rows_with_serial(req.rows, req.serial.as_ref())
        .map_err(MergePrintError::BadRequest)?;
    if rows.is_empty() {
        return Err(MergePrintError::BadRequest(anyhow::anyhow!(
            "no rows to print"
        )));
    }
    Span::current().record(
        "tepra.row_count",
        i64::try_from(rows.len()).unwrap_or(i64::MAX),
    );
    let csv_bytes = build_merge_csv(&frames, &rows, CsvEncoding::default())
        .map_err(MergePrintError::BadRequest)?;
    let csv_file = FilePayload {
        file_name: "merge.csv".to_owned(),
        base64_str: STANDARD.encode(&csv_bytes),
    };

    let print_request = PrintRequest {
        print_file: PrintFiles {
            template_file: Some(template_file),
            csv_file: Some(csv_file),
            image_file: None,
        },
        print_parameter: merge_print_parameter(&req.overrides),
    };

    state
        .client
        .print(printer, print_request)
        .await
        .map_err(MergePrintError::Upstream)
}

/// Combines `rows` (tapes) with a generated `serial` sequence, one serial
/// field merged into each tape.
///
/// - No `serial`: `rows` returned unchanged.
/// - `rows` empty: each generated serial value becomes its own single-field tape.
/// - `rows` has exactly one tape: that tape's fields are broadcast to every
///   generated tape (the common case — shared fields + a varying serial).
/// - `rows` has exactly `serial.count` tapes: zipped 1:1 by index.
/// - Any other length mismatch is ambiguous → error.
fn merge_rows_with_serial(
    rows: Vec<Vec<MergeField>>,
    serial: Option<&SerialSpec>,
) -> anyhow::Result<Vec<Vec<MergeField>>> {
    let Some(spec) = serial else {
        return Ok(rows);
    };
    let generated = expand_serial(spec);

    if rows.is_empty() {
        return Ok(generated.into_iter().map(|f| vec![f]).collect());
    }
    if let [base] = rows.as_slice() {
        return Ok(generated
            .into_iter()
            .map(|f| {
                let mut tape = base.clone();
                tape.push(f);
                tape
            })
            .collect());
    }
    anyhow::ensure!(
        rows.len() == generated.len(),
        "rows length ({}) must be 1 or match serial.count ({})",
        rows.len(),
        generated.len()
    );
    Ok(rows
        .into_iter()
        .zip(generated)
        .map(|(mut tape, f)| {
            tape.push(f);
            tape
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]

    use super::*;

    fn field(title: &str, value: &str) -> MergeField {
        MergeField {
            title: title.to_owned(),
            value: value.to_owned(),
        }
    }

    fn spec() -> SerialSpec {
        SerialSpec {
            title: "Bち".to_owned(),
            start: 1,
            count: 3,
            step: 1,
            pad: 3,
        }
    }

    #[test]
    fn no_serial_returns_rows_unchanged() {
        let rows = vec![vec![field("AA", "田中")]];
        let result = merge_rows_with_serial(rows.clone(), None).unwrap();
        assert_eq!(result, rows);
    }

    #[test]
    fn empty_rows_with_serial_generates_one_tape_per_value() {
        let result = merge_rows_with_serial(vec![], Some(&spec())).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec![field("Bち", "001")]);
    }

    #[test]
    fn single_row_broadcasts_and_merges_serial() {
        let rows = vec![vec![field("AA", "資産")]];
        let result = merge_rows_with_serial(rows, Some(&spec())).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[1], vec![field("AA", "資産"), field("Bち", "002")]);
    }

    #[test]
    fn matching_row_count_zips_by_index() {
        let rows = vec![
            vec![field("AA", "a")],
            vec![field("AA", "b")],
            vec![field("AA", "c")],
        ];
        let result = merge_rows_with_serial(rows, Some(&spec())).unwrap();
        assert_eq!(result[2], vec![field("AA", "c"), field("Bち", "003")]);
    }

    #[test]
    fn mismatched_row_count_errors() {
        let rows = vec![vec![field("AA", "a")], vec![field("AA", "b")]];
        let err = merge_rows_with_serial(rows, Some(&spec())).unwrap_err();
        assert!(err.to_string().contains("must be 1 or match"));
    }
}
