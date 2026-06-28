//! RED integration tests for views Creator API wiring.
//!
//! `views::index` and `views::printer_detail` are currently hardcoded stubs
//! (`printers: vec![]`, `online: false`). These tests define the expected
//! behaviour once the handlers call the Creator API via `AppState.client`.
//! All tests fail in the current state (RED).
#![allow(
    clippy::unwrap_used,
    clippy::missing_const_for_fn,
    clippy::significant_drop_tightening
)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tepra_api::{router::build_ui_router, state::AppState};
use tepra_core::{
    client::{
        mock::{MockCall, MockTepraClient},
        traits::TepraClient,
    },
    dto::printer::{OnlineStatusResponse, PrinterListItem},
    error::TepraError,
};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_app(client: Arc<dyn TepraClient>) -> axum::Router {
    build_ui_router(AppState::new(client))
}

async fn body_html(body: Body) -> String {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.into_iter().collect()).unwrap()
}

// ---------------------------------------------------------------------------
// 1. index_calls_list_printers
//    Mock returns ["PR-001", "PR-002"]; HTML must contain both names.
//    RED: stub returns printers: vec![] → names absent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn index_calls_list_printers() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![
        PrinterListItem {
            printer_name: "PR-001".into(),
        },
        PrinterListItem {
            printer_name: "PR-002".into(),
        },
    ]));

    let response = make_app(mock)
        .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("PR-001"),
        "index must render PR-001; got:\n{html}"
    );
    assert!(
        html.contains("PR-002"),
        "index must render PR-002; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 2. index_handles_client_error
//    Mock returns TepraError; handler must return 200 OK with error banner.
//    RED: current stub returns 502 BAD_GATEWAY.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn index_handles_client_error() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Err(TepraError::Creator { errcode: 500 }));

    let response = make_app(mock)
        .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "client error must yield 200 OK with error banner (not 502)"
    );
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("TEPRA Creator WebAPI に接続できません"),
        "index error must show error banner; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 3. printer_detail_calls_online_status
//    Mock returns online=true; HTML must contain "オンライン".
//    RED: stub always returns online=false → template shows "オフライン".
// ---------------------------------------------------------------------------

#[tokio::test]
async fn printer_detail_calls_online_status() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/printers/PR-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("オンライン"),
        "online printer must show オンライン; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 4. printer_detail_offline
//    Mock returns online=false; HTML must contain "オフライン".
//    Also verifies that client.online_status was actually called.
//    RED: stub never calls client → MockCall::OnlineStatus absent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn printer_detail_offline() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: false }));

    let client: Arc<dyn TepraClient> = mock.clone();
    let response = make_app(client)
        .oneshot(
            Request::builder()
                .uri("/ui/printers/PR-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("オフライン"),
        "offline printer must show オフライン; got:\n{html}"
    );
    let calls = mock.calls();
    assert!(
        calls.iter().any(|c| matches!(c, MockCall::OnlineStatus(_))),
        "printer_detail must call client.online_status"
    );
}

// ---------------------------------------------------------------------------
// 5. printer_detail_handles_client_error
//    Mock returns TepraError; handler must return 200 OK with error banner.
//    RED: current stub returns 502 BAD_GATEWAY.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn printer_detail_handles_client_error() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Err(TepraError::Creator { errcode: 500 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/printers/PR-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "client error must yield 200 OK with error banner (not 502)"
    );
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("TEPRA Creator WebAPI に接続できません"),
        "printer_detail error must show error banner; got:\n{html}"
    );
    assert!(
        html.contains("PR-001"),
        "printer_detail error must still show printer name from URL; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 6. error_banner_contains_full_product_name
//    Verify the exact vendor product name appears in the error banner.
//    RED: current stub returns 502, no HTML body with the product name.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn error_banner_contains_full_product_name() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Err(TepraError::Creator { errcode: 503 }));

    let response = make_app(mock)
        .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("TEPRA Creator WebAPI"),
        "error banner must contain full product name 'TEPRA Creator WebAPI'; got:\n{html}"
    );
}
