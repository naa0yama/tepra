//! RED unit tests for Askama template rendering.
//!
//! These tests verify that `IndexTemplate`, `PrinterDetailTemplate`, and
//! `JobCardTemplate` render to the expected HTML. They are committed in the
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
    ApiDocsTemplate, Breadcrumb, EndpointView, IndexTemplate, JobCardTemplate,
    PrinterDetailTemplate,
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
// PrinterDetailTemplate
// ---------------------------------------------------------------------------

#[test]
fn test_printer_detail_online() {
    let tmpl = PrinterDetailTemplate {
        nav_active: "printers".into(),
        breadcrumbs: vec![
            Breadcrumb {
                label: "Printers".into(),
                href: Some("/ui/".into()),
            },
            Breadcrumb {
                label: "PT-P710BT".into(),
                href: None,
            },
        ],
        printer_name: "PT-P710BT".into(),
        online: true,
        error: None,
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("PT-P710BT"));
    insta::assert_snapshot!("printer_detail_online", html);
}

#[test]
fn test_printer_detail_offline() {
    let tmpl = PrinterDetailTemplate {
        nav_active: "printers".into(),
        breadcrumbs: vec![
            Breadcrumb {
                label: "Printers".into(),
                href: Some("/ui/".into()),
            },
            Breadcrumb {
                label: "QL-800".into(),
                href: None,
            },
        ],
        printer_name: "QL-800".into(),
        online: false,
        error: None,
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("QL-800"));
    insta::assert_snapshot!("printer_detail_offline", html);
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
                request_schema_json: None,
                response_schema_json: Some("{\"type\":\"array\"}".into()),
                sample_json: Some("[]".into()),
                is_destructive: false,
                path_params: vec![],
            },
            EndpointView {
                method: "POST".into(),
                path: "/api/printer/print/{name}".into(),
                summary: "Print a label".into(),
                request_schema_json: Some("{\"type\":\"object\"}".into()),
                response_schema_json: None,
                sample_json: None,
                is_destructive: true,
                path_params: vec!["name".into()],
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

    // Non-destructive `GET /api/printer` gets a Try it out form (execute
    // button + result <pre>); the destructive `print` endpoint stays
    // schema-only in 4a (modal gate for it is added in 4b).
    assert_eq!(html.matches(r#"class="try-it-out-form"#).count(), 1);
    assert!(html.contains(r#"hx-get="/api/printer""#));
    assert!(html.contains("Execute"));
    assert!(html.contains(r#"id="try-it-out-result-1""#));

    insta::assert_snapshot!("api_docs_two_endpoints", html);
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
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: Some("{\"type\":\"object\"}".into()),
            sample_json: Some("{}".into()),
            is_destructive: false,
            path_params: vec!["name".into()],
        }],
        error: None,
    };
    let html = tmpl.render().unwrap();

    assert!(html.contains(r#"data-path-param="name""#));
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
            request_schema_json: Some("{\"type\":\"object\"}".into()),
            response_schema_json: Some("{\"type\":\"object\"}".into()),
            sample_json: Some("{}".into()),
            is_destructive: false,
            path_params: vec!["name".into()],
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
