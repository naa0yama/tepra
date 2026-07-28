//! Askama render-smoke tests for `GET /ui/print?from={record_id}` prefill.
//!
//! Render-path only, per spec T6 ("TDD-applicable: No"): verify each
//! prefill case renders expected markup without panicking, not full
//! behavioural coverage of the client-side JS.
#![allow(clippy::unwrap_used, clippy::missing_const_for_fn)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tepra::{
    handlers::merge_print::MergePrintRequest,
    jobs::JobOutcome,
    merge::{MergeField, MergePrintOverrides, SerialSpec},
    router::build_ui_router,
    state::AppState,
};
use tepra_core::{
    client::mock::MockTepraClient, dto::enums::ImportFrameAttribute, dto::template::ImportFrameItem,
};
use tower::ServiceExt;

fn make_app(state: AppState) -> axum::Router {
    build_ui_router(state)
}

async fn body_html(body: Body) -> String {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.into_iter().collect()).unwrap()
}

async fn get(app: axum::Router, uri: &str) -> String {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_html(response.into_body()).await
}

#[tokio::test]
async fn print_page_prefills_rows_from_job_record() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));

    let state = AppState::new_with_template_dir(mock, dir.path().to_path_buf());
    let record_id = state.jobs.record(
        "printer1".to_owned(),
        "label.lw1".to_owned(),
        MergePrintRequest {
            template: "label.lw1".to_owned(),
            rows: vec![
                vec![MergeField {
                    title: "Name".to_owned(),
                    value: "Alice".to_owned(),
                }],
                vec![MergeField {
                    title: "Name".to_owned(),
                    value: "Bob".to_owned(),
                }],
            ],
            serial: None,
            overrides: MergePrintOverrides::default(),
        },
        JobOutcome::Accepted { jobid: 1 },
        0,
    );

    let html = get(make_app(state), &format!("/ui/print?from={record_id}")).await;

    assert!(
        html.contains("value=\"Alice\""),
        "must prefill first tape's value; got:\n{html}"
    );
    assert!(
        html.contains("value=\"Bob\""),
        "must prefill second tape's value (N-tape join); got:\n{html}"
    );
    assert!(
        html.contains("data-template=\"label.lw1\""),
        "form must carry the record's template; got:\n{html}"
    );
}

#[tokio::test]
async fn print_page_prefills_printer_from_job_record() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));

    let state = AppState::new_with_template_dir(mock, dir.path().to_path_buf());
    let record_id = state.jobs.record(
        "printer1".to_owned(),
        "label.lw1".to_owned(),
        MergePrintRequest {
            template: "label.lw1".to_owned(),
            rows: vec![vec![MergeField {
                title: "Name".to_owned(),
                value: "Alice".to_owned(),
            }]],
            serial: None,
            overrides: MergePrintOverrides::default(),
        },
        JobOutcome::Accepted { jobid: 1 },
        0,
    );

    let html = get(make_app(state), &format!("/ui/print?from={record_id}")).await;

    assert!(
        html.contains(r#"data-selected-printer="printer1""#),
        "printer-info-select must carry the record's printer for client-side reselection; got:\n{html}"
    );
}

#[tokio::test]
async fn print_page_prefills_serial_enabled_from_job_record() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));

    let state = AppState::new_with_template_dir(mock, dir.path().to_path_buf());
    let record_id = state.jobs.record(
        "printer1".to_owned(),
        "label.lw1".to_owned(),
        MergePrintRequest {
            template: "label.lw1".to_owned(),
            rows: vec![vec![MergeField {
                title: "Name".to_owned(),
                value: "Alice".to_owned(),
            }]],
            serial: Some(SerialSpec {
                title: "Name".to_owned(),
                start: 100,
                count: 5,
                step: 2,
                pad: 3,
            }),
            overrides: MergePrintOverrides::default(),
        },
        JobOutcome::Accepted { jobid: 1 },
        0,
    );

    let html = get(make_app(state), &format!("/ui/print?from={record_id}")).await;

    assert!(
        html.contains(r#"id="serial-enable" checked"#),
        "serial checkbox must be checked; got:\n{html}"
    );
    assert!(
        html.contains(r#"id="serial-start" value="100""#),
        "serial-start must be prefilled; got:\n{html}"
    );
    assert!(
        html.contains(r#"id="serial-count" value="5""#),
        "serial-count must be prefilled; got:\n{html}"
    );
}

#[tokio::test]
async fn print_page_prefills_settings_from_job_record() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));

    let state = AppState::new_with_template_dir(mock, dir.path().to_path_buf());
    let record_id = state.jobs.record(
        "printer1".to_owned(),
        "label.lw1".to_owned(),
        MergePrintRequest {
            template: "label.lw1".to_owned(),
            rows: vec![vec![MergeField {
                title: "Name".to_owned(),
                value: "Alice".to_owned(),
            }]],
            serial: None,
            overrides: MergePrintOverrides {
                copies: Some(7),
                density: Some(-2),
                tape_cut: Some(2),
                half_cut: Some(2),
                half_cut_separate: Some(1),
                print_speed: Some(3),
                margin_left_right: Some(15),
                // task point C: 2 means "checked" on submit, but must invert
                // to *unchecked* on prefill.
                display_tape_width: Some(2),
                display_print_setting: Some(2),
            },
        },
        JobOutcome::Accepted { jobid: 1 },
        0,
    );

    let html = get(make_app(state), &format!("/ui/print?from={record_id}")).await;

    assert!(
        html.contains(r#"value="7" class="input input-bordered w-full" id="print-copies""#),
        "copies must be prefilled; got:\n{html}"
    );
    assert!(
        html.contains(r#"id="setting-density" value="-2""#),
        "density must be prefilled; got:\n{html}"
    );
    assert!(
        html.contains(r#"id="setting-margin-left-right" value="15""#),
        "margin must be prefilled; got:\n{html}"
    );

    let tape_width_tag = html
        .split(r#"id="setting-display-tape-width""#)
        .nth(1)
        .and_then(|rest| rest.split('>').next())
        .unwrap_or_default();
    assert!(
        !tape_width_tag.contains("checked"),
        "display_tape_width=Some(2) must render unchecked (checkbox inversion); got tag:\n{tape_width_tag}"
    );
}

#[tokio::test]
async fn print_page_shows_mismatch_banner_when_template_missing() {
    let dir = tempfile::tempdir().unwrap();
    let mock = Arc::new(MockTepraClient::new());

    let state = AppState::new_with_template_dir(mock, dir.path().to_path_buf());
    let record_id = state.jobs.record(
        "printer1".to_owned(),
        "gone.lw1".to_owned(),
        MergePrintRequest {
            template: "gone.lw1".to_owned(),
            rows: vec![vec![MergeField {
                title: "Name".to_owned(),
                value: "Alice".to_owned(),
            }]],
            serial: None,
            overrides: MergePrintOverrides::default(),
        },
        JobOutcome::Accepted { jobid: 1 },
        0,
    );

    let html = get(make_app(state), &format!("/ui/print?from={record_id}")).await;

    assert!(
        html.contains("alert-error"),
        "missing template must render mismatch banner; got:\n{html}"
    );
    assert!(
        html.contains("data-template=\"gone.lw1\""),
        "best-effort fill must still carry the original template; got:\n{html}"
    );
}

#[tokio::test]
async fn print_page_falls_back_to_empty_form_for_unknown_record_id() {
    let dir = tempfile::tempdir().unwrap();
    let mock = Arc::new(MockTepraClient::new());

    let state = AppState::new_with_template_dir(mock, dir.path().to_path_buf());

    let html = get(make_app(state), "/ui/print?from=999").await;

    assert!(
        html.contains("Select a template to load frames."),
        "unknown record_id must fall back to the traditional empty form; got:\n{html}"
    );
}
