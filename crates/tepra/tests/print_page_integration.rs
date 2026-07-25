//! Askama render-smoke tests for the merge-print UI page (`/ui/print`).
//!
//! Render-path only, per spec T6 ("TDD-applicable: No"): verify each
//! template renders with a representative context without panicking, not
//! full behavioural coverage of the client-side JS.
#![allow(clippy::unwrap_used, clippy::missing_const_for_fn)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tepra::{router::build_ui_router, state::AppState};
use tepra_core::{
    client::{
        mock::{MockCall, MockTepraClient},
        traits::TepraClient,
    },
    dto::{enums::ImportFrameAttribute, job::PrintResponse, template::ImportFrameItem},
    error::TepraError,
};
use tower::ServiceExt;

fn make_app(client: Arc<dyn TepraClient>, template_dir: std::path::PathBuf) -> axum::Router {
    build_ui_router(AppState::new_with_template_dir(client, template_dir))
}

async fn body_html(body: Body) -> String {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.into_iter().collect()).unwrap()
}

#[tokio::test]
async fn print_page_renders_with_no_templates() {
    let dir = tempfile::tempdir().unwrap();
    let mock = Arc::new(MockTepraClient::new());

    let response = make_app(mock, dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .uri("/ui/print")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(html.contains("Print"), "page must render; got:\n{html}");
    // Regression: the sidebar must expose a reachable link to /ui/print,
    // otherwise the page renders but has no navigation entry point.
    assert!(
        html.contains("href=\"/ui/print\""),
        "sidebar must link to /ui/print; got:\n{html}"
    );
}

#[tokio::test]
async fn print_frames_renders_frame_table_for_existing_template() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));

    let response = make_app(mock, dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .uri("/ui/print/frames?template=label.lw1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("Name"),
        "frame table must render column title; got:\n{html}"
    );
}

#[tokio::test]
async fn print_frames_renders_error_banner_for_missing_template() {
    let dir = tempfile::tempdir().unwrap();
    let mock = Arc::new(MockTepraClient::new());

    let response = make_app(mock, dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .uri("/ui/print/frames?template=missing.lw1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("alert-error"),
        "missing template must render an error banner; got:\n{html}"
    );
}

#[tokio::test]
async fn print_submit_renders_job_card_on_success() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));
    mock.push_print(Ok(PrintResponse {
        result: 1,
        jobid: 42,
    }));

    let body = serde_json::json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "Alice"}]],
    });

    let response = make_app(mock, dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ui/print/PR-001")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("Job #42"),
        "success submit must render job card; got:\n{html}"
    );
}

#[tokio::test]
async fn print_submit_flattens_browser_overrides_into_print_parameter() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));
    mock.push_print(Ok(PrintResponse {
        result: 1,
        jobid: 42,
    }));

    // Mirrors the browser payload shape from print.html's submitPrint():
    // override fields sit top-level (flattened), not nested under "overrides".
    let body = serde_json::json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "Alice"}]],
        "copies": 3,
    });

    let response = make_app(mock.clone(), dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ui/print/PR-001")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let copies = mock
        .calls()
        .iter()
        .find_map(|c| {
            if let MockCall::Print(_, req) = c {
                Some(req.print_parameter.copies)
            } else {
                None
            }
        })
        .expect("expected a Print call");
    assert_eq!(
        copies, 3,
        "top-level 'copies' override must reach PrintParameter, not fall back to the SDK default"
    );
}

#[tokio::test]
async fn print_submit_renders_error_banner_on_upstream_failure() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"dummy template bytes").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }]));
    mock.push_print(Err(TepraError::Creator { errcode: 500 }));

    let body = serde_json::json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "Alice"}]],
    });

    let response = make_app(mock, dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ui/print/PR-001")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("alert-error"),
        "upstream failure must render an error banner; got:\n{html}"
    );
}
