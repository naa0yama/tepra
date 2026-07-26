//! Askama HTML templates for the web UI.
//!
//! Template files live under `templates/` (Askama default search path).

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::Value;

use crate::{
    jobs::{JobOutcome, JobRecord},
    merge::MergeField,
};

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

/// App version for the sidebar footer, baked from Cargo at compile time.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Git short hash (7 chars) baked by build.rs; `unknown` if git is unavailable.
pub const GIT_HASH: &str = env!("GIT_HASH");

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
// Merge-print printer info panel (HTMX lazy-load target)
// ---------------------------------------------------------------------------

/// Context for the merge-print printer info panel partial
/// (`GET /ui/print/{printer}/panel`).
#[derive(Debug, Template)]
#[template(path = "partials/merge_printer_panel.html")]
pub struct MergePrinterPanelTemplate {
    /// Printer identifier.
    pub printer_name: String,
    /// Whether the printer is currently reachable.
    pub online: bool,
    /// Loaded tape width label (e.g. `"12mm"`), from `tape_id_label`.
    pub tape_width: String,
    /// Loaded tape kind label (e.g. `"標準ラベル"`), from `tape_kind_label`.
    pub tape_kind: &'static str,
    /// Top margin, pre-formatted (e.g. `"1.5mm"`); empty when unavailable.
    pub margin_top: String,
    /// Bottom margin, pre-formatted (e.g. `"1.5mm"`); empty when unavailable.
    pub margin_bottom: String,
    /// Left/right margin, pre-formatted (e.g. `"1.5mm"`); empty when unavailable.
    pub margin_left_right: String,
    /// Creator API error message when `onlinestatus`/`lwstatus` failed;
    /// independent of `margin_error` so a margin-only failure does not hide
    /// the online badge or tape info.
    pub status_error: Option<String>,
    /// Creator API error message when `getmargin` failed; independent of
    /// `status_error`.
    pub margin_error: Option<String>,
}

// ---------------------------------------------------------------------------
// Merge-print UI page
// ---------------------------------------------------------------------------

/// Sidebar section key for the merge-print page (`nav_active` field below).
///
/// Not yet matched by `templates/components/sidebar.html` — the page is not
/// linked from the sidebar until a later task, so this key currently never
/// highlights a nav item.
pub const NAV_PRINT: &str = "print";

/// Context for the merge-print page (`GET /ui/print`).
#[derive(Debug, Template)]
#[template(path = "pages/print.html")]
pub struct PrintPageTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
    /// Template files available for selection, from `list_templates`.
    pub templates: Vec<crate::templates::TemplateEntry>,
    /// Template directory read error, if any.
    pub error: Option<String>,
    /// `?from=<id>` resolved to a stored job: render `#print-frames-pane`
    /// inline (server-side, values populated) instead of the placeholder.
    pub prefill: bool,
    /// Reprint source template path; empty when not prefilled. Marks the
    /// matching `#template-select` `<option>` and is the inline frames
    /// form's `data-template`.
    pub selected_template: String,
    /// `true` when `selected_template` is absent from `templates` — shows
    /// a mismatch banner; best-effort fill still applies (task point E).
    pub template_mismatch: bool,
    /// Reprint source printer name; empty when not prefilled. Read by
    /// `populatePrinterSelects()` (client JS) to mark the matching
    /// `#printer-info-select` `<option>` selected after it rebuilds
    /// `innerHTML` from the fetched `/api/printer` list.
    pub selected_printer: String,
    /// Tape cards for the inline frames form; see [`build_tapes_view`].
    pub tapes: Vec<Vec<FrameFieldView>>,
    /// Serial section prefill for the inline frames form.
    pub serial: SerialPrefill,
    /// Right-column print-settings prefill.
    pub overrides: OverridePrefill,
}

/// Right-column setting-input prefill for `pages/print.html`.
#[derive(Debug, Clone)]
pub struct OverridePrefill {
    /// `#print-copies` value.
    pub copies: u32,
    /// `#setting-density` value; `None` leaves the input blank.
    pub density: Option<i32>,
    /// `#setting-tape-cut` selection; `0` = no selection (default option).
    pub tape_cut: u32,
    /// `#setting-half-cut` selection; `0` = no selection (default option).
    pub half_cut: u32,
    /// `#setting-half-cut-separate` selection; `0` = no selection (default option).
    pub half_cut_separate: u32,
    /// `#setting-print-speed` selection; `0` = no selection (default option).
    pub print_speed: u32,
    /// `#setting-margin-left-right` value; `None` leaves the input blank.
    pub margin_left_right: Option<u32>,
    /// `#setting-display-tape-width` checked state.
    pub display_tape_width_checked: bool,
    /// `#setting-display-print-setting` checked state.
    pub display_print_setting_checked: bool,
}

impl OverridePrefill {
    /// Values matching today's hardcoded markup exactly (no prefill).
    const fn default_form() -> Self {
        Self {
            copies: 1,
            density: None,
            tape_cut: 0,
            half_cut: 0,
            half_cut_separate: 0,
            print_speed: 0,
            margin_left_right: None,
            display_tape_width_checked: true,
            display_print_setting_checked: true,
        }
    }
}

/// Build [`OverridePrefill`] from a stored job's overrides.
///
/// `None` yields [`OverridePrefill::default_form`] exactly, so non-prefill
/// rendering is unchanged. Applies task point C's checkbox inversion:
/// `collectOverrides()` sends `displayTapeWidth`/`displayPrintSetting` as
/// `checked ? 1 : 2`, so an overrides value of `2` unchecks the box; `1` or
/// unset (matching the markup's current `checked` default) checks it.
#[must_use]
pub fn override_prefill_fields(
    overrides: Option<&crate::merge::MergePrintOverrides>,
) -> OverridePrefill {
    overrides.map_or_else(OverridePrefill::default_form, |o| OverridePrefill {
        copies: o.copies.unwrap_or(1),
        density: o.density,
        tape_cut: o.tape_cut.unwrap_or(0),
        half_cut: o.half_cut.unwrap_or(0),
        half_cut_separate: o.half_cut_separate.unwrap_or(0),
        print_speed: o.print_speed.unwrap_or(0),
        margin_left_right: o.margin_left_right,
        display_tape_width_checked: o.display_tape_width != Some(2),
        display_print_setting_checked: o.display_print_setting != Some(2),
    })
}

/// Context for the frame-table + print-settings-form partial
/// (`GET /ui/print/frames?template=<rel>`).
#[derive(Debug, Template)]
#[template(path = "partials/merge_frames.html")]
pub struct MergeFramesTemplate {
    /// Template path (relative to the template directory) the frames were
    /// extracted from; echoed back as a hidden form field for submit.
    pub template: String,
    /// Import frames extracted from the template, in column order.
    pub frames: Vec<tepra_core::dto::template::ImportFrameItem>,
    /// Set when the template could not be read or `import_frame` failed.
    pub error: Option<String>,
    /// Tape cards to render, one entry per tape; see [`build_tapes_view`].
    pub tapes: Vec<Vec<FrameFieldView>>,
    /// Serial-number section prefill; see [`serial_prefill_fields`].
    pub serial: SerialPrefill,
}

/// One `{title, value}` cell rendered inside a tape card in
/// `partials/frames_form.html`.
#[derive(Debug, Clone)]
pub struct FrameFieldView {
    /// Frame column title (`data-field-title` on the rendered input).
    pub title: String,
    /// Prefilled value; empty when there is no prefill or the title has no
    /// match in the stored row (frame-drift best-effort absorption).
    pub value: String,
}

/// Build the tape-card rows for `partials/frames_form.html`.
///
/// `rows` is `None` (or `Some(&[])`, treated the same way) for the
/// traditional empty form: renders exactly one tape card with empty-value
/// fields, so non-prefill output stays byte-identical to before prefill
/// existed. Otherwise renders one card per submitted row (task point B:
/// render ALL tapes, not just the first), joining each frame column by
/// `title` against the row's [`MergeField`]s — unmatched columns render
/// empty rather than erroring, absorbing frame drift best-effort.
#[must_use]
pub fn build_tapes_view(
    frames: &[tepra_core::dto::template::ImportFrameItem],
    rows: Option<&[Vec<MergeField>]>,
) -> Vec<Vec<FrameFieldView>> {
    rows.filter(|r| !r.is_empty()).map_or_else(
        || {
            vec![
                frames
                    .iter()
                    .map(|f| FrameFieldView {
                        title: f.title.clone(),
                        value: String::new(),
                    })
                    .collect(),
            ]
        },
        |rows| {
            rows.iter()
                .map(|row| {
                    frames
                        .iter()
                        .map(|f| FrameFieldView {
                            title: f.title.clone(),
                            value: row
                                .iter()
                                .find(|field| field.title == f.title)
                                .map_or_else(String::new, |field| field.value.clone()),
                        })
                        .collect()
                })
                .collect()
        },
    )
}

/// Flattened serial-number prefill for `partials/frames_form.html`, avoiding
/// nested `Option` handling in the template.
#[derive(Debug, Clone)]
pub struct SerialPrefill {
    /// Whether `#serial-enable` is checked and `#serial-fields` is visible.
    pub enabled: bool,
    /// `#serial-title` selection; empty when disabled.
    pub title: String,
    /// `#serial-start` value.
    pub start: i64,
    /// `#serial-count` value.
    pub count: u32,
    /// `#serial-step` value.
    pub step: i64,
    /// `#serial-pad` value.
    pub pad: u8,
}

/// Build [`SerialPrefill`] from a stored job's `SerialSpec`.
///
/// `None` yields the form's current hardcoded defaults exactly (disabled,
/// start/count/step = 1, pad = 0), so non-prefill rendering is unchanged.
#[must_use]
pub fn serial_prefill_fields(serial: Option<&crate::merge::SerialSpec>) -> SerialPrefill {
    serial.map_or(
        SerialPrefill {
            enabled: false,
            title: String::new(),
            start: 1,
            count: 1,
            step: 1,
            pad: 0,
        },
        |s| SerialPrefill {
            enabled: true,
            title: s.title.clone(),
            start: s.start,
            count: s.count,
            step: s.step,
            pad: s.pad,
        },
    )
}

/// Context for a standalone error banner partial, reused by the merge-print
/// HTMX/fetch handlers to report a failure inline without a full page reload.
#[derive(Debug, Template)]
#[template(path = "partials/error_alert.html")]
pub struct ErrorAlertTemplate {
    /// Human-readable error message.
    pub message: String,
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
    /// `true` for program-specific REST helpers (`/api/rest/*`); `false` for
    /// the official Creator `WebAPI` facade (`/api/printer/*`). Drives the
    /// two-section grouping in `pages/api.html`.
    pub is_custom: bool,
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

/// Path prefix for program-specific REST helpers, distinct from the
/// official Creator `WebAPI` facade (`/api/printer/*`).
const CUSTOM_PATH_PREFIX: &str = "/api/rest/";

fn is_custom_path(path: &str) -> bool {
    path.starts_with(CUSTOM_PATH_PREFIX)
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
                is_custom: is_custom_path(path),
            });
        }
    }
    endpoints
}

// ---------------------------------------------------------------------------
// Jobs history page
// ---------------------------------------------------------------------------

/// Sidebar section key for the jobs history page (`nav_active` field below).
///
/// Matched by string equality in `templates/components/sidebar.html`
/// (`{% if active == "jobs" %}`).
pub const NAV_JOBS: &str = "jobs";

/// One job entry rendered by `pages/jobs.html`, flattened from a
/// [`JobRecord`] so the template never needs to pattern-match [`JobOutcome`].
#[derive(Debug, Clone)]
pub struct JobEntryView {
    /// Internal monotonic record ID (distinct from the Creator API `jobid`).
    pub record_id: u64,
    /// Printer name the job was submitted to.
    pub printer: String,
    /// Template path used, for list display.
    pub template: String,
    /// Submission time, epoch seconds (UTC); formatted client-side via `data-epoch`.
    pub submitted_at: u64,
    /// `"accepted"` or `"failed"`, matching [`JobOutcome::label`].
    pub outcome_label: &'static str,
    /// `jobid={id}` for accepted jobs, the upstream error message for failed ones.
    pub outcome_detail: String,
    /// Creator API `jobid`, present only for accepted jobs; drives the
    /// live-progress lazy-load and the reprint link's `?from=` target.
    pub job_id: Option<u64>,
    /// Tape parameters submitted with the job, one entry per tape (label).
    pub rows: Vec<Vec<MergeField>>,
}

/// Build one [`JobEntryView`] from a stored [`JobRecord`], pre-flattening
/// its [`JobOutcome`] so `partials/job_entry.html` only deals with plain
/// fields.
fn job_entry_view(record: JobRecord) -> JobEntryView {
    let (outcome_label, outcome_detail, job_id) = match record.outcome {
        JobOutcome::Accepted { jobid } => ("accepted", format!("jobid={jobid}"), Some(jobid)),
        JobOutcome::Failed { message } => ("failed", message, None),
    };
    JobEntryView {
        record_id: record.record_id,
        printer: record.printer,
        template: record.template,
        submitted_at: record.submitted_at,
        outcome_label,
        outcome_detail,
        job_id,
        rows: record.request.rows,
    }
}

/// Build the [`JobEntryView`] list for one page of `JobStore::page`'s result.
// WHY-NOT: renaming to drop the module-name repetition — matches the
// existing `build_endpoint_views` naming convention for view-model builders.
#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn build_job_entry_views(records: Vec<JobRecord>) -> Vec<JobEntryView> {
    records.into_iter().map(job_entry_view).collect()
}

/// Context for the jobs history page (`GET /ui/jobs?page=N`).
#[derive(Debug, Template)]
#[template(path = "pages/jobs.html")]
pub struct JobsPageTemplate {
    /// Active sidebar section key (`shells/dashboard.html`).
    pub nav_active: String,
    /// Navbar breadcrumb trail (`shells/dashboard.html`).
    pub breadcrumbs: Vec<Breadcrumb>,
    /// One entry per job on the current page, newest first.
    pub jobs: Vec<JobEntryView>,
    /// Current page number (1-indexed).
    pub page: usize,
    /// Total record count across all pages.
    pub total: usize,
    /// Page size (`DEFAULT_PAGE_SIZE`).
    pub page_size: usize,
    /// Total number of pages, at least 1.
    pub total_pages: usize,
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
            },
            "/api/rest/templates": {
                "get": {
                    "summary": "List template files",
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
        assert_eq!(endpoints.len(), 6);
    }

    #[test]
    fn build_endpoint_views_flags_custom_paths() {
        let endpoints = build_endpoint_views(&fixture_openapi());

        let custom = endpoints
            .iter()
            .find(|e| e.path == "/api/rest/templates")
            .unwrap();
        assert!(custom.is_custom);

        let official = endpoints.iter().find(|e| e.path == "/api/printer").unwrap();
        assert!(!official.is_custom);
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

    fn job_record(outcome: JobOutcome) -> JobRecord {
        JobRecord {
            record_id: 1,
            printer: "printer1".to_owned(),
            submitted_at: 1_000,
            template: "label.lw1".to_owned(),
            request: crate::handlers::merge_print::MergePrintRequest::default(),
            outcome,
        }
    }

    #[test]
    fn job_entry_view_flattens_accepted_outcome() {
        let view = job_entry_view(job_record(JobOutcome::Accepted { jobid: 42 }));
        assert_eq!(view.outcome_label, "accepted");
        assert_eq!(view.outcome_detail, "jobid=42");
        assert_eq!(view.job_id, Some(42));
    }

    #[test]
    fn job_entry_view_flattens_failed_outcome() {
        let view = job_entry_view(job_record(JobOutcome::Failed {
            message: "boom".to_owned(),
        }));
        assert_eq!(view.outcome_label, "failed");
        assert_eq!(view.outcome_detail, "boom");
        assert_eq!(view.job_id, None);
    }

    #[test]
    fn build_job_entry_views_preserves_order() {
        let records = vec![
            job_record(JobOutcome::Accepted { jobid: 1 }),
            job_record(JobOutcome::Failed {
                message: "err".to_owned(),
            }),
        ];
        let views = build_job_entry_views(records);
        assert_eq!(views.len(), 2);
        assert_eq!(views.first().unwrap().outcome_label, "accepted");
        assert_eq!(views.get(1).unwrap().outcome_label, "failed");
    }
}
