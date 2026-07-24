//! Tests for `GET /api/openapi.json`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tepra::router::build_router;
use tepra_core::client::{mock::MockTepraClient, traits::TepraClient};
use tower::ServiceExt;

fn make_app(client: Arc<dyn TepraClient>) -> axum::Router {
    build_router(client)
}

async fn body_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_openapi_json_returns_200_valid_document() {
    let mock = Arc::new(MockTepraClient::new());

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;

    assert_eq!(json["openapi"], "3.1.0");

    let paths = json["paths"].as_object().unwrap();
    for path in [
        "/api/printer",
        "/api/printer/version",
        "/api/printer/autoselect",
        "/api/printer/info/{name}",
        "/api/printer/onlinestatus/{name}",
        "/api/printer/lwstatus/{name}",
        "/api/printer/getmargin/{name}",
        "/api/printer/print/{name}",
        "/api/printer/tapefeed/{name}",
        "/api/printer/job/progress/{name}",
        "/api/printer/job/info/{name}",
        "/api/printer/job/control/{name}",
        "/api/printer/template/importframe",
        "/api/templates",
    ] {
        assert!(paths.contains_key(path), "missing path: {path}");
    }

    let schemas = json["components"]["schemas"].as_object().unwrap();
    assert!(schemas.contains_key("PrinterListItem"));
    assert!(schemas.contains_key("PrintRequest"));
    assert!(schemas.contains_key("TemplateEntry"));
}
