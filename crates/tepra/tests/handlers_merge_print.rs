//! Integration tests for `POST /api/rest/merge-print/{printer}` and
//! `GET /api/rest/templates/preview` against `MockTepraClient`.
#![cfg(not(miri))]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::significant_drop_tightening,
    clippy::missing_const_for_fn,
    clippy::items_after_statements,
    clippy::needless_pass_by_value
)]

use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tepra::{
    router::{build_merge_router, build_templates_router},
    state::AppState,
};
use tepra_core::{
    client::{mock::MockTepraClient, traits::TepraClient},
    dto::{enums::ImportFrameAttribute, job::PrintResponse, template::ImportFrameItem},
    error::TepraError,
};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn merge_app(client: Arc<dyn TepraClient>, template_dir: PathBuf) -> axum::Router {
    build_merge_router(AppState::new_with_template_dir(client, template_dir))
}

fn templates_app(client: Arc<dyn TepraClient>, template_dir: PathBuf) -> axum::Router {
    build_templates_router(AppState::new_with_template_dir(client, template_dir))
}

async fn body_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn name_frame() -> ImportFrameItem {
    ImportFrameItem {
        column: "A1".into(),
        title: "Name".into(),
        attribute: ImportFrameAttribute::Text,
    }
}

/// Minimal self-contained BMP: "BM" magic + `bfSize` (LE u32) covering only
/// the header bytes themselves, followed by trailing `.lw1`-only data that
/// must NOT be included in the sliced-out preview.
fn fake_lw1_bytes() -> Vec<u8> {
    let mut bmp = b"BM".to_vec();
    bmp.extend_from_slice(&20_u32.to_le_bytes());
    bmp.extend_from_slice(&[0_u8; 14]); // pad the BMP segment out to bfSize=20
    let mut file = bmp;
    file.extend_from_slice(b"TRAILING_LW1_FRAME_DATA");
    file
}

// ---------------------------------------------------------------------------
// POST /api/rest/merge-print/{printer}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_merge_print_returns_200_with_jobid() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![name_frame()]));
    mock.push_print(Ok(PrintResponse {
        result: 1,
        jobid: 42,
    }));

    let req_body = json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "田中"}]],
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["jobid"], 42);
}

#[tokio::test]
async fn test_merge_print_missing_template_returns_404() {
    let dir = tempfile::tempdir().unwrap();
    let mock = Arc::new(MockTepraClient::new());

    let req_body = json!({
        "template": "missing.lw1",
        "rows": [[{"title": "Name", "value": "田中"}]],
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_merge_print_path_traversal_returns_404() {
    // A real file must exist outside `root` for this test to exercise the
    // `starts_with(root)` prefix check — a non-existent traversal target
    // fails earlier at `canonicalize()` and never reaches that check.
    let outer = tempfile::tempdir().unwrap();
    let root = outer.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(outer.path().join("outside.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());

    let req_body = json!({
        "template": "../outside.lw1",
        "rows": [[{"title": "Name", "value": "田中"}]],
    });

    let response = merge_app(mock, root)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_merge_print_import_frame_error_returns_502() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Err(TepraError::Creator { errcode: 1 }));

    let req_body = json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "田中"}]],
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn test_merge_print_print_error_returns_502() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![name_frame()]));
    mock.push_print(Err(TepraError::Creator { errcode: 2 }));

    let req_body = json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "田中"}]],
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn test_merge_print_unknown_title_returns_400() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![name_frame()]));

    let req_body = json!({
        "template": "label.lw1",
        "rows": [[{"title": "NotAColumn", "value": "田中"}]],
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_merge_print_zero_tapes_returns_400() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![name_frame()]));

    let req_body = json!({
        "template": "label.lw1",
        "rows": [],
        "serial": {"title": "Name", "start": 1, "count": 0, "step": 1, "pad": 3},
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_merge_print_serial_expansion_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![
        name_frame(),
        ImportFrameItem {
            column: "A2".into(),
            title: "Serial".into(),
            attribute: ImportFrameAttribute::Text,
        },
    ]));
    mock.push_print(Ok(PrintResponse {
        result: 1,
        jobid: 7,
    }));

    let req_body = json!({
        "template": "label.lw1",
        "rows": [[{"title": "Name", "value": "資産"}]],
        "serial": {"title": "Serial", "start": 1, "count": 3, "step": 1, "pad": 3},
    });

    let response = merge_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rest/merge-print/printer1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["jobid"], 7);
}

// ---------------------------------------------------------------------------
// GET /api/rest/templates/preview
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_template_preview_returns_200_with_bmp_bytes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());
    let response = templates_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .uri("/api/rest/templates/preview?path=label.lw1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-type").unwrap(), "image/bmp");
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(bytes.len(), 20);
    assert_eq!(&bytes[0..2], b"BM");
}

#[tokio::test]
async fn test_template_preview_missing_template_returns_404() {
    let dir = tempfile::tempdir().unwrap();
    let mock = Arc::new(MockTepraClient::new());

    let response = templates_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .uri("/api/rest/templates/preview?path=missing.lw1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_template_preview_non_bmp_template_returns_404() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("label.lw1"), b"NOT_A_BMP_FILE").unwrap();

    let mock = Arc::new(MockTepraClient::new());
    let response = templates_app(mock, dir.path().to_owned())
        .oneshot(
            Request::builder()
                .uri("/api/rest/templates/preview?path=label.lw1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_template_preview_path_traversal_returns_404() {
    // See test_merge_print_path_traversal_returns_404: the traversal target
    // must exist outside `root`, or canonicalize() fails before the
    // starts_with(root) prefix check ever runs.
    let outer = tempfile::tempdir().unwrap();
    let root = outer.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(outer.path().join("outside.lw1"), fake_lw1_bytes()).unwrap();

    let mock = Arc::new(MockTepraClient::new());

    let response = templates_app(mock, root)
        .oneshot(
            Request::builder()
                .uri("/api/rest/templates/preview?path=../outside.lw1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
