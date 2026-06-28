//! RED unit tests for Askama template rendering.
//!
//! These tests verify that `IndexTemplate`, `PrinterDetailTemplate`, and
//! `JobCardTemplate` render to the expected HTML. They are committed in the
//! RED phase: `cargo build` fails with "template not found" because the
//! `.html` files do not exist yet (created in T15b GREEN).
//!
//! Snapshot files are also absent until the first `cargo test` pass in T15b,
//! at which point `insta` writes them to `tests/snapshots/`.
#![allow(
    clippy::unwrap_used,
    clippy::missing_const_for_fn,
    clippy::items_after_statements,
    clippy::needless_pass_by_value
)]

use askama::Template as _;
use tepra_api::views::{IndexTemplate, JobCardTemplate, PrinterDetailTemplate};

// ---------------------------------------------------------------------------
// IndexTemplate
// ---------------------------------------------------------------------------

#[test]
fn test_index_render_empty_printers() {
    let tmpl = IndexTemplate { printers: vec![] };
    let html = tmpl.render().unwrap();
    assert!(html.contains("<!DOCTYPE html") || html.contains("<html"));
    insta::assert_snapshot!("index_empty", html);
}

#[test]
fn test_index_render_multiple_printers() {
    let tmpl = IndexTemplate {
        printers: vec!["PT-P710BT".into(), "QL-800".into()],
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
        printer_name: "PT-P710BT".into(),
        online: true,
    };
    let html = tmpl.render().unwrap();
    assert!(html.contains("PT-P710BT"));
    insta::assert_snapshot!("printer_detail_online", html);
}

#[test]
fn test_printer_detail_offline() {
    let tmpl = PrinterDetailTemplate {
        printer_name: "QL-800".into(),
        online: false,
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
