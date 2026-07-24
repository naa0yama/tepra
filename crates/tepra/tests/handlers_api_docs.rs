//! Tests for `GET /ui/api` route wiring through `build_ui_router`.
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tepra::{router::build_ui_router, state::AppState};
use tepra_core::client::{mock::MockTepraClient, traits::TepraClient};
use tower::ServiceExt;

fn make_app(client: Arc<dyn TepraClient>) -> axum::Router {
    build_ui_router(AppState::new(client))
}

async fn body_html(body: Body) -> String {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.into_iter().collect()).unwrap()
}

#[tokio::test]
async fn ui_api_route_returns_200_when_registered_in_router() {
    let mock = Arc::new(MockTepraClient::new());

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/api")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn ui_api_route_renders_api_docs_page_when_requested() {
    let mock = Arc::new(MockTepraClient::new());

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/api")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let html = body_html(response.into_body()).await;

    assert!(html.contains("API Reference"));
    assert!(html.contains("destructive"));
    assert!(html.contains("try-it-out-form"));
}
