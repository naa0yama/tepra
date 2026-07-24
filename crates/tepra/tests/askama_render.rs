//! RED unit tests for Askama template rendering.
//!
//! These tests verify that `IndexTemplate` and `JobCardTemplate` render to
//! the expected HTML. They are committed in the
//! RED phase: `cargo build` fails with "template not found" because the
//! `.html` files do not exist yet (created in T15b GREEN).
//!
//! Snapshot files are also absent until the first `cargo test` pass in T15b,
//! at which point `insta` writes them to `tests/snapshots/`.
// insta uses `cargo metadata` subprocess which miri isolation blocks;
// snapshot tests are not the target of UB detection.
#![cfg(not(miri))]
#![allow(
    clippy::unwrap_used,
    clippy::missing_const_for_fn,
    clippy::items_after_statements,
    clippy::needless_pass_by_value
)]

use askama::Template as _;
use tepra::views::{
    ApiDocsTemplate, Breadcrumb, EndpointView, IndexTemplate, JobCardTemplate, ParamView,
    PropertyView,
};

// ---------------------------------------------------------------------------
// IndexTemplate
// ---------------------------------------------------------------------------

#[test]
fn test_index_render_empty_printers() {
    let tmpl = IndexTemplate {
        nav_active: "printers".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "Printers".into(),
            href: None,
        }],
        printers: vec![],
        error: None,
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("<!DOCTYPE html") || html.contains("<html"));
    insta::assert_snapshot!("index_empty", html);
}

#[test]
fn test_index_render_multiple_printers() {
    let tmpl = IndexTemplate {
        nav_active: "printers".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "Printers".into(),
            href: None,
        }],
        printers: vec!["PT-P710BT".into(), "QL-800".into()],
        error: None,
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("PT-P710BT"));
    assert!(html.contains("QL-800"));
    insta::assert_snapshot!("index_two_printers", html);
}

// ---------------------------------------------------------------------------
// JobCardTemplate
// ---------------------------------------------------------------------------

#[test]
fn test_job_card_in_progress() {
    let tmpl = JobCardTemplate {
        printer_name: "PT-P710BT".into(),
        job_id: 1,
        job_end: false,
        canceled: false,
        progress: Some(42),
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("PT-P710BT"));
    // Polling must be active: hx-trigger present
    assert!(html.contains("hx-trigger"));
    insta::assert_snapshot!("job_card_in_progress", html);
}

#[test]
fn test_job_card_completed() {
    let tmpl = JobCardTemplate {
        printer_name: "PT-P710BT".into(),
        job_id: 2,
        job_end: true,
        canceled: false,
        progress: Some(100),
    };
    let html = tmpl.render().unwrap();
    // Polling must stop when job_end=true: no hx-trigger on polling interval
    assert!(!html.contains("hx-trigger=\"every 1s\""));
    insta::assert_snapshot!("job_card_completed", html);
}

#[test]
fn test_job_card_canceled() {
    let tmpl = JobCardTemplate {
        printer_name: "PT-P710BT".into(),
        job_id: 3,
        job_end: true,
        canceled: true,
        progress: None,
    };
    let html = tmpl.render().unwrap();
    insta::assert_snapshot!("job_card_canceled", html);
}

// ---------------------------------------------------------------------------
// ApiDocsTemplate
// ---------------------------------------------------------------------------

#[test]
fn test_api_docs_render_lists_endpoints() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![
            EndpointView {
                method: "GET".into(),
                path: "/api/printer".into(),
                summary: "List printers".into(),
                params: vec![],
                request_properties: vec![],
                response_properties: vec![],
                request_schema_json: None,
                response_schema_json: Some("{\"type\":\"array\"}".into()),
                sample_json: Some("[]".into()),
                is_destructive: false,
                path_params: vec![],
                query_params: vec![],
            },
            EndpointView {
                method: "POST".into(),
                path: "/api/printer/print/{name}".into(),
                summary: "Print a label".into(),
                params: vec![],
                request_properties: vec![],
                response_properties: vec![],
                request_schema_json: Some("{\"type\":\"object\"}".into()),
                response_schema_json: None,
                sample_json: None,
                is_destructive: true,
                path_params: vec!["name".into()],
                query_params: vec![],
            },
        ],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains("/api/printer"));
    assert!(html.contains("/api/printer/print/{name}"));
    assert!(html.contains("List printers"));
    assert!(html.contains("Print a label"));
    assert!(html.contains("destructive"));

    // Both the non-destructive `GET /api/printer` and the destructive
    // `print` endpoint get a Try it out form (execute button + result
    // <pre>); only the destructive one is gated behind the confirm modal.
    assert_eq!(html.matches(r#"class="try-it-out-form"#).count(), 2);
    assert!(html.contains(r#"hx-get="/api/printer""#));
    assert!(html.contains("Execute"));
    assert!(html.contains(r#"id="try-it-out-result-1""#));
    assert!(html.contains(r#"id="try-it-out-result-2""#));

    // The destructive endpoint's form carries the modal gate marker and its
    // Execute button is a plain button (not type="submit"), routed through
    // the shared confirm modal instead of submitting directly.
    assert_eq!(html.matches("data-destructive-form").count(), 1);
    assert!(html.contains("data-destructive-trigger"));
    assert!(html.contains(r#"id="destructive-confirm-modal""#));

    assert!(html.contains(r"badge badge-outline w-16 justify-center badge-info"));
    assert!(html.contains(r"badge badge-outline w-16 justify-center badge-warning"));
    assert!(html.contains(r"badge badge-outline badge-error badge-sm"));

    insta::assert_snapshot!("api_docs_two_endpoints", html);
}

#[test]
fn api_docs_renders_property_table_when_endpoint_declares_properties() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "POST".into(),
            path: "/api/printer/print/{name}".into(),
            summary: "Print a label".into(),
            params: vec![ParamView {
                name: "name".into(),
                type_name: "string".into(),
                required: true,
                description: Some("Printer name.".into()),
            }],
            request_properties: vec![PropertyView {
                name: "templateName".into(),
                type_name: "string".into(),
                required: true,
                description: Some("Template file to print.".into()),
            }],
            response_properties: vec![PropertyView {
                name: "jobId".into(),
                type_name: "integer".into(),
                required: false,
                description: Some("Queued job identifier.".into()),
            }],
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: Some("{\"type\":\"object\"}".into()),
            sample_json: Some("{}".into()),
            is_destructive: true,
            path_params: vec!["name".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains("Printer name."));
    assert!(html.contains("templateName"));
    assert!(html.contains("Template file to print."));
    assert!(html.contains("jobId"));
    assert!(html.contains("Queued job identifier."));
}

#[test]
fn api_docs_renders_placeholder_when_property_has_no_description() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "POST".into(),
            path: "/api/printer/print/{name}".into(),
            summary: "Print a label".into(),
            params: vec![],
            request_properties: vec![PropertyView {
                name: "templateName".into(),
                type_name: "string".into(),
                required: true,
                description: None,
            }],
            response_properties: vec![],
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: None,
            sample_json: None,
            is_destructive: true,
            path_params: vec!["name".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains("&mdash;"));
}

#[test]
fn api_docs_places_raw_json_after_property_table_when_endpoint_declares_properties() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "POST".into(),
            path: "/api/printer/print/{name}".into(),
            summary: "Print a label".into(),
            params: vec![ParamView {
                name: "name".into(),
                type_name: "string".into(),
                required: true,
                description: Some("Printer name.".into()),
            }],
            request_properties: vec![PropertyView {
                name: "templateName".into(),
                type_name: "string".into(),
                required: true,
                description: Some("Template file to print.".into()),
            }],
            response_properties: vec![],
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: None,
            sample_json: None,
            is_destructive: true,
            path_params: vec!["name".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains("<details open>"));
    assert!(html.contains("Raw JSON schema (request)"));
    let table_pos = html.find("templateName").unwrap();
    let request_json_pos = html.find("{&#34;type&#34;:&#34;object&#34;}").unwrap();
    assert!(request_json_pos > table_pos);
}

#[test]
fn api_docs_omits_property_table_when_endpoint_declares_no_properties() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "GET".into(),
            path: "/api/printer".into(),
            summary: "List printers".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: None,
            response_schema_json: Some("{\"type\":\"array\"}".into()),
            sample_json: Some("[]".into()),
            is_destructive: false,
            path_params: vec![],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(!html.contains("<h3 class=\"font-semibold text-sm mb-1\">Parameters</h3>"));
    assert!(!html.contains("<h3 class=\"font-semibold text-sm mb-1\">Request body</h3>"));
    assert!(!html.contains("<h3 class=\"font-semibold text-sm mb-1\">Response body</h3>"));
}

#[test]
fn api_docs_non_destructive_form_has_no_destructive_gate_marker() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "GET".into(),
            path: "/api/printer".into(),
            summary: "List printers".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: None,
            response_schema_json: Some("{\"type\":\"array\"}".into()),
            sample_json: Some("[]".into()),
            is_destructive: false,
            path_params: vec![],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    // The click-handler script always references the `data-destructive-trigger`
    // selector (it's static markup shared across all endpoint rows), so only
    // the per-form marker distinguishes a destructive form from this one.
    assert!(!html.contains("data-destructive-form"));
    assert!(html.contains(r#"type="submit""#));
}

fn destructive_endpoint_view() -> EndpointView {
    EndpointView {
        method: "POST".into(),
        path: "/api/printer/print/{name}".into(),
        summary: "Print a label".into(),
        params: vec![],
        request_properties: vec![],
        response_properties: vec![],
        request_schema_json: Some("{\"type\":\"object\"}".into()),
        response_schema_json: None,
        sample_json: None,
        is_destructive: true,
        path_params: vec!["name".into()],
        query_params: vec![],
    }
}

#[test]
fn api_docs_renders_destructive_form_marker_when_endpoint_is_destructive() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![destructive_endpoint_view()],
        error: None,
    };
    let html = tmpl.render().unwrap();

    // The click-handler script always references the `data-destructive-trigger`
    // selector, so only the form-specific marker proves the destructive form
    // itself was rendered (not just the shared script).
    assert!(html.contains("data-destructive-form"));
}

#[test]
fn api_docs_uses_button_type_trigger_when_endpoint_is_destructive() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![destructive_endpoint_view()],
        error: None,
    };
    let html = tmpl.render().unwrap();

    // The trailing `>` distinguishes the button's own attribute from the
    // script's `closest("[data-destructive-trigger]")` selector string.
    assert!(html.contains("data-destructive-trigger>Execute"));
    // Destructive Execute button must not be type="submit" — direct
    // submission would bypass the confirm modal.
    assert!(!html.contains(r#"type="submit""#));
}

#[test]
fn api_docs_renders_path_param_input_when_endpoint_has_path_param() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "POST".into(),
            path: "/api/printer/getmargin/{name}".into(),
            summary: "Get printer margin".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: Some("{\"type\":\"object\"}".into()),
            sample_json: Some("{}".into()),
            is_destructive: false,
            path_params: vec!["name".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains(r#"data-path-param="name""#));
}

#[test]
fn api_docs_renders_name_path_param_as_select_when_param_is_printer_name() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "GET".into(),
            path: "/api/printer/info/{name}".into(),
            summary: "Get printer info".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: None,
            response_schema_json: None,
            sample_json: None,
            is_destructive: false,
            path_params: vec!["name".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains(r#"<select name="name" data-path-param="name" data-printer-select"#));
    assert!(!html.contains(r#"<input type="text" name="name""#));
}

#[test]
fn api_docs_renders_non_name_path_param_as_text_input_when_param_is_not_printer_name() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "GET".into(),
            path: "/api/job/{jobid}".into(),
            summary: "Get job".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: None,
            response_schema_json: None,
            sample_json: None,
            is_destructive: false,
            path_params: vec!["jobid".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains(r#"<input type="text" name="jobid" data-path-param="jobid""#));
    // The populate script always references [data-printer-select] as a
    // selector string; only assert no such *element* was rendered.
    assert!(!html.contains(r#"data-path-param="jobid" data-printer-select"#));
}

#[test]
fn api_docs_renders_query_param_input_without_path_param_marker() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "GET".into(),
            path: "/api/printer/job/progress/{name}".into(),
            summary: "Get job progress".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: None,
            response_schema_json: None,
            sample_json: None,
            is_destructive: false,
            path_params: vec!["name".into()],
            query_params: vec![ParamView {
                name: "jobid".into(),
                type_name: "integer".into(),
                required: true,
                description: Some("Creator API job identifier.".into()),
            }],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains(r#"<input type="text" name="jobid""#));
    // It must NOT carry data-path-param — otherwise the configRequest handler
    // would strip it from the outgoing query string instead of sending it.
    assert!(!html.contains(r#"name="jobid" data-path-param"#));
}

#[test]
fn api_docs_uses_json_submit_when_endpoint_has_request_body() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![EndpointView {
            method: "POST".into(),
            path: "/api/printer/getmargin/{name}".into(),
            summary: "Get printer margin".into(),
            params: vec![],
            request_properties: vec![],
            response_properties: vec![],
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: Some("{\"type\":\"object\"}".into()),
            sample_json: Some("{}".into()),
            is_destructive: false,
            path_params: vec!["name".into()],
            query_params: vec![],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    // The form falls back to the plain-JS submit handler (no hx-post)
    // because it carries a JSON body.
    assert!(html.contains("data-json-body"));
    assert!(html.contains("data-json-body-form"));
    assert!(!html.contains(r#"hx-post="/api/printer/getmargin/{name}""#));
}

#[test]
fn test_api_docs_render_marks_sidebar_active() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![],
        error: None,
    };
    let html = tmpl.render().unwrap();

    // Sidebar active branch (`components/sidebar.html`) picks up `nav_active`.
    assert!(html.contains(r#"href="/ui/api" class="menu-active" aria-current="page""#));
}

#[test]
fn test_api_docs_render_empty_endpoints() {
    let tmpl = ApiDocsTemplate {
        nav_active: "api".into(),
        breadcrumbs: vec![Breadcrumb {
            label: "API".into(),
            href: None,
        }],
        endpoints: vec![],
        error: None,
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("No endpoints found"));
    insta::assert_snapshot!("api_docs_empty", html);
}
