//! RED unit tests for template-related handler endpoints.
//!
//! `import_frame` relies on `MockTepraClient` (always passes when mock is primed).
//! `list_template_files` calls `list_templates()` which is `todo!()` → panic → RED.
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
use serde_json::Value;
use tepra_api::{router::build_templates_router, state::AppState};
use tepra_core::{
    client::{mock::MockTepraClient, traits::TepraClient},
    dto::{
        enums::ImportFrameAttribute,
        job::FilePayload,
        template::{ImportFrameItem, ImportFrameRequest},
    },
};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_app(client: Arc<dyn TepraClient>, template_dir: PathBuf) -> axum::Router {
    build_templates_router(AppState::new_with_template_dir(client, template_dir))
}

async fn body_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

// ---------------------------------------------------------------------------
// 1. POST /api/printer/template/importframe
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_import_frame_returns_200_with_items() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Ok(vec![ImportFrameItem {
        id: 1,
        attribute: ImportFrameAttribute::Text,
        width: 100,
        height: 50,
    }]));

    let req_body = serde_json::to_vec(&ImportFrameRequest {
        template_file: FilePayload {
            file_name: "label.lbx".into(),
            base64_str: "dGVzdA==".into(),
        },
    })
    .unwrap();

    let response = make_app(mock, PathBuf::new())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/template/importframe")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    let items = json.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], 1u32);
    assert_eq!(items[0]["width"], 100u32);
}

#[tokio::test]
async fn test_import_frame_client_error_returns_502() {
    use tepra_core::error::TepraError;

    let mock = Arc::new(MockTepraClient::new());
    mock.push_import_frame(Err(TepraError::Creator { errcode: 1 }));

    let req_body = serde_json::to_vec(&ImportFrameRequest {
        template_file: FilePayload {
            file_name: "bad.lbx".into(),
            base64_str: String::new(),
        },
    })
    .unwrap();

    let response = make_app(mock, PathBuf::new())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/template/importframe")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}
