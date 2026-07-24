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
/// (`{% if active == "printers" %}`).
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
// Printer status card partial (HTMX lazy-load target)
// ---------------------------------------------------------------------------

/// Context for the printer status-card partial
/// (`GET /ui/printers/{name}/status-card`).
#[derive(Debug, Template)]
#[template(path = "partials/printer_status_card.html")]
pub struct PrinterStatusCardTemplate {
    /// Printer identifier.
    pub printer_name: String,
    /// Whether the printer is currently reachable.
    pub online: bool,
    /// Loaded tape width label (e.g. `"12mm"`), from `tape_id_label`.
    pub tape_width: String,
    /// Loaded tape kind label (e.g. `"標準ラベル"`), from `tape_kind_label`.
    pub tape_kind: &'static str,
    /// Creator API error message, if the status fetch failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// API reference page
// ---------------------------------------------------------------------------

/// Sidebar section key for the API reference page (`nav_active` field below).
///
/// Matched by string equality in `templates/components/sidebar.html`
/// (`{% if active == "api" %}`).
pub const NAV_API: &str = "api";

/// One path or query parameter, extracted from an operation's `parameters`
/// array for display in the API reference property table.
#[derive(Debug, Clone)]
pub struct ParamView {
    /// Parameter name (e.g. `name`).
    pub name: String,
    /// Display type (e.g. `string`, `array<Printer>`).
    pub type_name: String,
    /// Whether the request is rejected if this parameter is absent.
    pub required: bool,
    /// Human-readable description, when the operation declares one.
    pub description: Option<String>,
}

/// One request or response body property, extracted from a JSON-Schema
/// object (or the item schema of an array body) for display in the API
/// reference property table.
#[derive(Debug, Clone)]
pub struct PropertyView {
    /// Property name (e.g. `printerName`).
    pub name: String,
    /// Display type (e.g. `integer`, `array<Printer>`).
    pub type_name: String,
    /// Whether the schema's `required` list includes this property.
    pub required: bool,
    /// Human-readable description, when the DTO field declares one.
    pub description: Option<String>,
}

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
    /// Path and query parameters declared on the operation.
    pub params: Vec<ParamView>,
    /// Request body properties; empty for endpoints with no body.
    pub request_properties: Vec<PropertyView>,
    /// `200` response body properties; empty for empty-body responses.
    pub response_properties: Vec<PropertyView>,
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
    /// Path parameter names in declaration order (e.g. `["name"]` for
    /// `/api/printer/info/{name}`), used by the Try-it-out form to render
    /// one input per placeholder.
    pub path_params: Vec<String>,
    /// Query parameters declared on the operation (`in == "query"`), used by
    /// the Try-it-out form to render one input each (e.g. `jobid`, `cutflag`).
    /// Separate from `path_params` because query inputs must NOT carry the
    /// `data-path-param` marker — htmx serializes them into the GET query
    /// string, whereas path params are substituted into `{...}` placeholders.
    pub query_params: Vec<ParamView>,
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

/// Extract `{name}`-style path parameter names, in declaration order
/// (e.g. `/api/printer/info/{name}` -> `["name"]`).
fn extract_path_params(path: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        let after_brace = &rest[start.saturating_add(1)..];
        let Some(end) = after_brace.find('}') else {
            break;
        };
        params.push(after_brace[..end].to_owned());
        rest = &after_brace[end.saturating_add(1)..];
    }
    params
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

/// Last path segment of a JSON-Schema `$ref` pointer
/// (`#/components/schemas/Printer` -> `Printer`).
fn ref_name(pointer: &str) -> String {
    pointer.rsplit('/').next().unwrap_or(pointer).to_owned()
}

/// `schema`'s `type` keyword as a non-null type-name string. utoipa emits
/// `Option<T>` in two different shapes depending on `T`: a `oneOf: [{"type":
/// "null"}, T]` wrapper for `Option<Struct>`/`Option<Enum>` (handled by the
/// `oneOf` branch in [`schema_type_name`]), and a bare `"type": [T, "null"]`
/// array for `Option<primitive>` (handled here). Returns `None` when `type`
/// is absent or is only `"null"`.
fn nullable_type_str(schema: &Value) -> Option<&str> {
    match schema.get("type") {
        Some(Value::String(type_name)) => Some(type_name.as_str()),
        Some(Value::Array(variants)) => variants
            .iter()
            .filter_map(Value::as_str)
            .find(|type_name| *type_name != "null"),
        _ => None,
    }
}

/// Display type for a (possibly `$ref`'d, possibly nullable) JSON Schema
/// (e.g. `string`, `Printer`, `array<Printer>`). The display type is the
/// non-null variant's, since optionality is already carried by `required`.
fn schema_type_name(schema: &Value) -> String {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return ref_name(reference);
    }
    if let Some(variants) = schema.get("oneOf").and_then(Value::as_array) {
        return variants
            .iter()
            .find(|v| v.get("type").and_then(Value::as_str) != Some("null"))
            .map_or_else(|| "any".to_owned(), schema_type_name);
    }
    match nullable_type_str(schema) {
        Some("array") => {
            let item_type = schema
                .get("items")
                .map_or_else(|| "any".to_owned(), schema_type_name);
            format!("array<{item_type}>")
        }
        Some(other) => other.to_owned(),
        None => "any".to_owned(),
    }
}

/// Description for a (possibly `oneOf`-wrapped) JSON Schema. For
/// `Option<Struct>`/`Option<Enum>` fields (`oneOf: [{"type": "null"}, T]`)
/// utoipa attaches the field-level `description` to the non-null variant
/// rather than the `oneOf` wrapper itself, so that variant is checked as a
/// fallback. `Option<primitive>` fields (`"type": [T, "null"]`) keep their
/// `description` at the top level, so the first check already covers them.
fn schema_description(schema: &Value) -> Option<String> {
    if let Some(description) = schema.get("description").and_then(Value::as_str) {
        return Some(description.to_owned());
    }
    let variants = schema.get("oneOf").and_then(Value::as_array)?;
    variants
        .iter()
        .find(|v| v.get("type").and_then(Value::as_str) != Some("null"))
        .and_then(|v| v.get("description").and_then(Value::as_str))
        .map(str::to_owned)
}

/// Map one entry of an operation's `parameters` array to a [`ParamView`].
fn param_view(param: &Value) -> ParamView {
    ParamView {
        name: param
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        type_name: param
            .get("schema")
            .map_or_else(|| "any".to_owned(), schema_type_name),
        required: param
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        description: param
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_owned),
    }
}

/// Extract an operation's path/query parameters (`parameters` array) for
/// the property table. Returns an empty `Vec` for operations without one.
fn extract_params(operation: &Value) -> Vec<ParamView> {
    operation
        .get("parameters")
        .and_then(Value::as_array)
        .map(|params| params.iter().map(param_view).collect())
        .unwrap_or_default()
}

/// Extract only the query parameters (`in == "query"`) from an operation's
/// `parameters` array, for the Try-it-out form's query-string inputs.
/// Returns an empty `Vec` for operations without any query parameter.
fn extract_query_params(operation: &Value) -> Vec<ParamView> {
    operation
        .get("parameters")
        .and_then(Value::as_array)
        .map(|params| {
            params
                .iter()
                .filter(|param| param.get("in").and_then(Value::as_str) == Some("query"))
                .map(param_view)
                .collect()
        })
        .unwrap_or_default()
}

/// Extract the properties of a (possibly `$ref`'d) request/response body
/// schema for the property table. Array bodies (e.g. `GET /api/printer`
/// returning `Vec<PrinterListItem>`) list the item type's properties, since
/// that is what the caller actually reads/sends per element. Returns an
/// empty `Vec` when the schema has no `properties` (e.g. a bare scalar).
fn extract_properties(schema: &Value, openapi: &Value) -> Vec<PropertyView> {
    let resolved = resolve_ref(schema, openapi);
    if resolved.get("type").and_then(Value::as_str) == Some("array") {
        return resolved
            .get("items")
            .map(|items| extract_properties(items, openapi))
            .unwrap_or_default();
    }

    let Some(properties) = resolved.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    let required: Vec<&str> = resolved
        .get("required")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    properties
        .iter()
        .map(|(name, prop)| PropertyView {
            name: name.clone(),
            type_name: schema_type_name(prop),
            required: required.contains(&name.as_str()),
            description: schema_description(prop),
        })
        .collect()
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

            let params = extract_params(operation);
            let query_params = extract_query_params(operation);

            let request_schema = operation.pointer("/requestBody/content/application~1json/schema");
            let request_schema_json = request_schema
                .map(|schema| resolve_ref(schema, openapi))
                .and_then(|schema| serde_json::to_string_pretty(schema).ok());
            let request_properties = request_schema
                .map(|schema| extract_properties(schema, openapi))
                .unwrap_or_default();

            let response_schema =
                operation.pointer("/responses/200/content/application~1json/schema");
            let response_schema_json = response_schema
                .map(|schema| resolve_ref(schema, openapi))
                .and_then(|schema| serde_json::to_string_pretty(schema).ok());
            let response_properties = response_schema
                .map(|schema| extract_properties(schema, openapi))
                .unwrap_or_default();
            let sample_json = response_schema
                .map(|schema| sample_value(schema, openapi, 8))
                .and_then(|value| serde_json::to_string_pretty(&value).ok());

            endpoints.push(EndpointView {
                method: method.to_uppercase(),
                path: path.clone(),
                summary,
                params,
                request_properties,
                response_properties,
                request_schema_json,
                response_schema_json,
                sample_json,
                is_destructive: is_destructive_path(path),
                path_params: extract_path_params(path),
                query_params,
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

    fn fixture_paths() -> Value {
        json!({
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
                    "parameters": [
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "description": "Printer name",
                            "schema": {"type": "string"}
                        }
                    ],
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
        })
    }

    fn fixture_schemas() -> Value {
        json!({
            "PrinterListItem": {
                "type": "object",
                "properties": {
                    "printerName": {"type": "string", "description": "Printer display name."}
                },
                "required": ["printerName"]
            },
            "PrintRequest": {
                "type": "object",
                "properties": {
                    "copies": {"type": "integer", "description": "Number of copies."},
                    "note": {
                        "oneOf": [
                            {"type": "null"},
                            {"type": "string", "description": "Optional operator note."}
                        ]
                    }
                },
                "required": ["copies"]
            },
            "PrintResponse": {
                "type": "object",
                "properties": {
                    "jobId": {"type": "integer"},
                    "warningCode": {
                        "type": ["integer", "null"],
                        "description": "Non-fatal warning code; absent on success."
                    }
                },
                "required": ["jobId"]
            }
        })
    }

    fn fixture_openapi() -> Value {
        json!({
            "paths": fixture_paths(),
            "components": {
                "schemas": fixture_schemas()
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

    #[test]
    fn build_endpoint_views_extracts_path_params() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();
        assert_eq!(print.path_params, vec!["name".to_owned()]);

        let list = endpoints.iter().find(|e| e.path == "/api/printer").unwrap();
        assert!(list.path_params.is_empty());
    }

    #[test]
    fn extract_path_params_finds_single_placeholder() {
        assert_eq!(
            extract_path_params("/api/printer/info/{name}"),
            vec!["name".to_owned()]
        );
    }

    #[test]
    fn extract_path_params_finds_multiple_placeholders() {
        assert_eq!(
            extract_path_params("/api/{a}/foo/{b}"),
            vec!["a".to_owned(), "b".to_owned()]
        );
    }

    #[test]
    fn extract_path_params_returns_empty_for_no_placeholder() {
        assert!(extract_path_params("/api/printer").is_empty());
    }

    #[test]
    fn build_endpoint_views_extracts_path_param_metadata() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        assert_eq!(print.params.len(), 1);
        let name_param = print.params.first().unwrap();
        assert_eq!(name_param.name, "name");
        assert_eq!(name_param.type_name, "string");
        assert!(name_param.required);
        assert_eq!(name_param.description.as_deref(), Some("Printer name"));
    }

    #[test]
    fn extract_query_params_keeps_only_query_location_params() {
        let operation = json!({
            "parameters": [
                {
                    "name": "name",
                    "in": "path",
                    "required": true,
                    "schema": {"type": "string"}
                },
                {
                    "name": "jobid",
                    "in": "query",
                    "required": true,
                    "description": "Creator API job identifier.",
                    "schema": {"type": "integer"}
                }
            ]
        });

        let query = extract_query_params(&operation);
        assert_eq!(query.len(), 1);
        let jobid = query.first().unwrap();
        assert_eq!(jobid.name, "jobid");
        assert_eq!(jobid.type_name, "integer");
        assert!(jobid.required);
        assert_eq!(
            jobid.description.as_deref(),
            Some("Creator API job identifier.")
        );
    }

    #[test]
    fn extract_query_params_returns_empty_when_operation_declares_none() {
        let operation = json!({
            "parameters": [
                {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}
            ]
        });
        assert!(extract_query_params(&operation).is_empty());
    }

    #[test]
    fn build_endpoint_views_returns_no_params_when_operation_declares_none() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let list = endpoints.iter().find(|e| e.path == "/api/printer").unwrap();
        assert!(list.params.is_empty());
    }

    #[test]
    fn build_endpoint_views_extracts_request_properties_with_required_flag() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        let copies = print
            .request_properties
            .iter()
            .find(|p| p.name == "copies")
            .unwrap();
        assert_eq!(copies.type_name, "integer");
        assert!(copies.required);
        assert_eq!(copies.description.as_deref(), Some("Number of copies."));
    }

    #[test]
    fn build_endpoint_views_marks_optional_property_not_required_with_variant_description() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        let note = print
            .request_properties
            .iter()
            .find(|p| p.name == "note")
            .unwrap();
        assert_eq!(note.type_name, "string");
        assert!(!note.required);
        assert_eq!(note.description.as_deref(), Some("Optional operator note."));
    }

    #[test]
    fn build_endpoint_views_extracts_response_properties() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        let job_id = print
            .response_properties
            .iter()
            .find(|p| p.name == "jobId")
            .unwrap();
        assert_eq!(job_id.type_name, "integer");
        assert!(job_id.required);
    }

    #[test]
    fn build_endpoint_views_extracts_response_properties_from_array_item_schema() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let list = endpoints.iter().find(|e| e.path == "/api/printer").unwrap();

        assert_eq!(list.response_properties.len(), 1);
        let printer_name = list.response_properties.first().unwrap();
        assert_eq!(printer_name.name, "printerName");
        assert_eq!(printer_name.type_name, "string");
        assert!(printer_name.required);
        assert_eq!(
            printer_name.description.as_deref(),
            Some("Printer display name.")
        );
    }

    #[test]
    fn build_endpoint_views_returns_empty_properties_for_endpoints_without_body() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let tapefeed = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/tapefeed/{name}")
            .unwrap();

        assert!(tapefeed.request_properties.is_empty());
        assert!(tapefeed.response_properties.is_empty());
    }

    #[test]
    fn schema_type_name_formats_ref_as_bare_name() {
        assert_eq!(
            schema_type_name(&json!({"$ref": "#/components/schemas/Printer"})),
            "Printer"
        );
    }

    #[test]
    fn schema_type_name_formats_array_of_ref_with_item_type() {
        let schema = json!({
            "type": "array",
            "items": {"$ref": "#/components/schemas/Printer"}
        });
        assert_eq!(schema_type_name(&schema), "array<Printer>");
    }

    #[test]
    fn schema_type_name_unwraps_option_to_inner_type() {
        let schema = json!({"oneOf": [{"type": "null"}, {"type": "string"}]});
        assert_eq!(schema_type_name(&schema), "string");
    }

    #[test]
    fn schema_type_name_unwraps_nullable_primitive_type_array() {
        let schema = json!({"type": ["integer", "null"]});
        assert_eq!(schema_type_name(&schema), "integer");
    }

    #[test]
    fn build_endpoint_views_resolves_nullable_primitive_response_property() {
        let endpoints = build_endpoint_views(&fixture_openapi());
        let print = endpoints
            .iter()
            .find(|e| e.path == "/api/printer/print/{name}")
            .unwrap();

        let warning_code = print
            .response_properties
            .iter()
            .find(|p| p.name == "warningCode")
            .unwrap();
        assert_eq!(warning_code.type_name, "integer");
        assert!(!warning_code.required);
        assert_eq!(
            warning_code.description.as_deref(),
            Some("Non-fatal warning code; absent on success.")
        );
    }
}
