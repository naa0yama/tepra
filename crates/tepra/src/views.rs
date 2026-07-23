//! Askama HTML templates for the web UI.
//!
//! Template files live under `templates/` (Askama default search path).

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::Value;

/// Newtype that renders an askama template as an HTML response.
///
/// Required because askama 0.13+ removed framework integration crates.
#[derive(Debug)]
pub struct HtmlTemplate<T: Template>(pub T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        self.0.render().map_or_else(
            |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            |html| Html(html).into_response(),
        )
    }
}

// ---------------------------------------------------------------------------
// Index page — printer list
// ---------------------------------------------------------------------------

/// Sidebar section key for the printers page (`nav_active` field below).
///
/// Matched by string equality in `templates/components/sidebar.html`
/// (`{% if active == "printers" %}`); defined once here so the two handlers
/// that set `nav_active` (`handlers::views::index`, `::printer_detail`)
/// cannot drift out of sync with each other.
pub const NAV_PRINTERS: &str = "printers";

/// One entry in a navbar breadcrumb trail (`shells/dashboard.html`).
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    /// Display label.
    pub label: String,
    /// Link target; `None` renders the current page as plain text.
    pub href: Option<String>,
}

/// Context for the top-level index page (`GET /`).
#[derive(Debug, Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
    /// Display names of all known printers.
    pub printers: Vec<String>,
    /// Creator API error message, if the API call failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Printer detail page
// ---------------------------------------------------------------------------

/// Context for the per-printer detail page (`GET /printers/{name}`).
#[derive(Debug, Template)]
#[template(path = "pages/printer_detail.html")]
pub struct PrinterDetailTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
    /// Printer identifier.
    pub printer_name: String,
    /// Whether the printer is currently reachable.
    pub online: bool,
    /// Creator API error message, if the API call failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Job card partial (HTMX polling target)
// ---------------------------------------------------------------------------

/// Context for the job-status card partial (`GET /jobs/{printer}/{id}`).
#[derive(Debug, Template)]
#[template(path = "partials/job_card.html")]
pub struct JobCardTemplate {
    /// Printer that owns the job.
    pub printer_name: String,
    /// Job sequence ID (display only).
    pub job_id: u64,
    /// `true` when the job has finished (stops HTMX polling).
    pub job_end: bool,
    /// `true` when the job was canceled.
    pub canceled: bool,
    /// Print progress percentage (0–100), `None` while queued.
    pub progress: Option<u32>,
}

// ---------------------------------------------------------------------------
// API reference page
// ---------------------------------------------------------------------------

/// Sidebar section key for the API reference page (`nav_active` field below).
///
/// Matched by string equality in `templates/components/sidebar.html`
/// (`{% if active == "api" %}`).
pub const NAV_API: &str = "api";

/// One endpoint entry rendered by `pages/api.html`, derived from the
/// code-generated `OpenAPI` document (`handlers::openapi::ApiDoc`).
#[derive(Debug, Clone)]
pub struct EndpointView {
    /// HTTP method, upper case (`GET`, `POST`, ...).
    pub method: String,
    /// Route path, as declared in `router.rs` (e.g. `/api/printer/{name}`).
    pub path: String,
    /// Operation summary (utoipa derives this from the handler's doc comment).
    pub summary: String,
    /// Pretty-printed JSON request body schema; `None` for endpoints with no body.
    pub request_schema_json: Option<String>,
    /// Pretty-printed JSON `200` response schema; `None` for empty-body responses.
    pub response_schema_json: Option<String>,
    /// Pretty-printed placeholder JSON instance of the `200` response schema.
    pub sample_json: Option<String>,
    /// `true` for endpoints with a physical side effect (print / tape feed /
    /// job cancel). Consumed by the Try-it-out confirm modal added in a
    /// later task; this page only displays the flag.
    pub is_destructive: bool,
}

/// Context for the API reference page (`GET /ui/api`).
#[derive(Debug, Template)]
#[template(path = "pages/api.html")]
pub struct ApiDocsTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
    /// One entry per `OpenAPI` operation, in path/method order.
    pub endpoints: Vec<EndpointView>,
    /// Set when the `OpenAPI` document could not be turned into view-models.
    pub error: Option<String>,
}

/// Route markers that identify a physical side effect on the printer.
/// Matched by substring since path params (`{name}`, `{id}`) vary per route.
const DESTRUCTIVE_PATH_MARKERS: [&str; 3] = ["/print/", "/tapefeed/", "/job/control/"];

fn is_destructive_path(path: &str) -> bool {
    DESTRUCTIVE_PATH_MARKERS
        .iter()
        .any(|marker| path.contains(marker))
}

/// Resolve a JSON-Schema `$ref` (`#/components/schemas/Name`) against the
/// root `OpenAPI` document. Returns `schema` unchanged when it is not a
/// `$ref` or the pointer does not resolve.
fn resolve_ref<'a>(schema: &'a Value, openapi: &'a Value) -> &'a Value {
    schema
        .get("$ref")
        .and_then(Value::as_str)
        .map_or(schema, |pointer| {
            pointer
                .strip_prefix('#')
                .and_then(|p| openapi.pointer(p))
                .unwrap_or(schema)
        })
}

/// Build a placeholder JSON instance from a (possibly `$ref`'d) JSON Schema,
/// recursively resolving nested `$ref`s. `depth` bounds recursion; none of
/// the current DTOs are self-referential, so this never triggers in practice.
fn sample_value(schema: &Value, openapi: &Value, depth: u8) -> Value {
    let Some(depth) = depth.checked_sub(1) else {
        return Value::Null;
    };
    let schema = resolve_ref(schema, openapi);

    // `Option<T>` fields render as `oneOf: [{"type": "null"}, T]`.
    if let Some(variants) = schema.get("oneOf").and_then(Value::as_array) {
        return variants
            .iter()
            .find(|v| v.get("type").and_then(Value::as_str) != Some("null"))
            .map_or(Value::Null, |v| sample_value(v, openapi, depth));
    }

    match schema.get("type").and_then(Value::as_str) {
        Some("object") => {
            let map = schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|props| {
                    props
                        .iter()
                        .map(|(name, prop)| (name.clone(), sample_value(prop, openapi, depth)))
                        .collect()
                })
                .unwrap_or_default();
            Value::Object(map)
        }
        Some("array") => {
            let item = schema
                .get("items")
                .map_or(Value::Null, |items| sample_value(items, openapi, depth));
            Value::Array(vec![item])
        }
        Some("integer") => Value::from(0),
        Some("number") => Value::from(0.0),
        Some("boolean") => Value::from(false),
        Some("string") => Value::from("string"),
        _ => Value::Null,
    }
}

/// Build one [`EndpointView`] per `OpenAPI` operation, in path/method order.
///
/// `openapi` is `ApiDoc::openapi()` (`handlers::openapi`) serialized via
/// `serde_json::to_value`. Pure function: no I/O, no template access, so it
/// is unit-testable against fixture `OpenAPI` JSON.
// WHY-NOT: renaming to drop the module-name repetition — the spec
// (2026-07-24-builtin-api-reference-page.md) fixes this exact name for the
// `EndpointView` view-model builder; kept as-is for traceability.
#[allow(clippy::module_name_repetitions)]
pub fn build_endpoint_views(openapi: &Value) -> Vec<EndpointView> {
    let Some(paths) = openapi.get("paths").and_then(Value::as_object) else {
        return Vec::new();
    };

    let mut endpoints = Vec::new();
    for (path, operations) in paths {
        let Some(operations) = operations.as_object() else {
            continue;
        };
        for (method, operation) in operations {
            let summary = operation
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();

            let request_schema_json = operation
                .pointer("/requestBody/content/application~1json/schema")
                .map(|schema| resolve_ref(schema, openapi))
                .and_then(|schema| serde_json::to_string_pretty(schema).ok());

            let response_schema =
                operation.pointer("/responses/200/content/application~1json/schema");
            let response_schema_json = response_schema
                .map(|schema| resolve_ref(schema, openapi))
                .and_then(|schema| serde_json::to_string_pretty(schema).ok());
            let sample_json = response_schema
                .map(|schema| sample_value(schema, openapi, 8))
                .and_then(|value| serde_json::to_string_pretty(&value).ok());

            endpoints.push(EndpointView {
                method: method.to_uppercase(),
                path: path.clone(),
                summary,
                request_schema_json,
                response_schema_json,
                sample_json,
                is_destructive: is_destructive_path(path),
            });
        }
    }
    endpoints
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use super::*;

    fn fixture_openapi() -> Value {
        json!({
            "paths": {
                "/api/printer": {
                    "get": {
                        "summary": "List printers",
                        "responses": {
                            "200": {
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "array",
                                            "items": {"$ref": "#/components/schemas/PrinterListItem"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/api/printer/print/{name}": {
                    "post": {
                        "summary": "Print a label",
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/PrintRequest"}
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "content": {
                                    "application/json": {
                                        "schema": {"$ref": "#/components/schemas/PrintResponse"}
                                    }
                                }
                            }
                        }
                    }
                },
                "/api/printer/tapefeed/{name}": {
                    "get": {
                        "summary": "Feed tape",
                        "responses": {
                            "200": {"description": "OK"}
                        }
                    }
                },
                "/api/printer/job/control/{id}": {
                    "post": {
                        "summary": "Control a print job",
                        "responses": {
                            "200": {"description": "OK"}
                        }
                    }
                },
                "/api/printer/getmargin": {
                    "post": {
                        "summary": "Get printer margin",
                        "responses": {
                            "200": {"description": "OK"}
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "PrinterListItem": {
                        "type": "object",
                        "properties": {"printerName": {"type": "string"}},
                        "required": ["printerName"]
                    },
                    "PrintRequest": {
                        "type": "object",
                        "properties": {
                            "copies": {"type": "integer"},
                            "note": {"oneOf": [{"type": "null"}, {"type": "string"}]}
                        },
                        "required": ["copies"]
                    },
                    "PrintResponse": {
                        "type": "object",
                        "properties": {"jobId": {"type": "integer"}},
                        "required": ["jobId"]
                    }
                }
            }
        })
    }

    #[test]
    fn build_endpoint_views_enumerates_every_operation() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        assert_eq!(endpoints.len(), 5);
    }

    #[test]
    fn build_endpoint_views_flags_destructive_paths() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();
        assert!(print.is_destructive);

        let tapefeed = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/tapefeed/{name}")
            .unwrap();
        assert!(tapefeed.is_destructive);

        let job_control = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/job/control/{id}")
            .unwrap();
        assert!(job_control.is_destructive);

        let list = endpoints.iter().find(|e| e.path == "/api/printer").unwrap();
        assert!(!list.is_destructive);

        // POST alone must not trigger the flag — it is path-marker driven, not method-driven.
        let getmargin = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/getmargin")
            .unwrap();
        assert!(!getmargin.is_destructive);
    }

    #[test]
    fn build_endpoint_views_resolves_request_and_response_schema() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        let request = print.request_schema_json.as_ref().unwrap();
        assert!(request.contains("copies"));
        let response = print.response_schema_json.as_ref().unwrap();
        assert!(response.contains("jobId"));
    }

    #[test]
    fn build_endpoint_views_builds_placeholder_sample_for_response() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        let sample: Value = serde_json::from_str(print.sample_json.as_ref().unwrap()).unwrap();
        assert_eq!(sample.get("jobId"), Some(&json!(0)));
    }

    #[test]
    fn build_endpoint_views_handles_missing_response_body() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let tapefeed = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/tapefeed/{name}")
            .unwrap();

        assert!(tapefeed.response_schema_json.is_none());
        assert!(tapefeed.sample_json.is_none());
    }

    #[test]
    fn build_endpoint_views_resolves_optional_field_to_non_null_variant() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();
        let request = print.request_schema_json.as_ref().unwrap();
        assert!(request.contains("oneOf"));
    }

    #[test]
    fn build_endpoint_views_returns_empty_for_missing_paths() {
        let endpoints = build_endpoint_views(&json!({}));
        assert!(endpoints.is_empty());
    }
}
