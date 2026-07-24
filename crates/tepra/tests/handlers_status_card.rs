//! Unit tests for the printer status-card HTMX partial
//! (`GET /ui/printers/{name}/status-card`).
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
use tepra::{router::build_ui_router, state::AppState};
use tepra_core::{
    client::{mock::MockTepraClient, traits::TepraClient},
    dto::printer::{LwStatusResponse, OnlineStatusResponse},
    error::TepraError,
};
use tower::ServiceExt;

fn make_app(client: Arc<dyn TepraClient>) -> axum::Router {
    build_ui_router(AppState::new(client))
}

async fn body_html(body: Body) -> String {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.into_iter().collect()).unwrap()
}

fn lw_status(tape_id: u32, tape_kind: i32) -> LwStatusResponse {
    LwStatusResponse {
        tape_id,
        tape_kind,
        error: 0,
        br_tape_kind: 0,
        status: 0,
        status_type: 4,
        tape_sw: None,
        t8_option: None,
    }
}

#[tokio::test]
async fn status_card_online_shows_tape_labels() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Ok(lw_status(261, 0)));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/printers/PR-001/status-card")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(html.contains("Online"), "got:\n{html}");
    assert!(html.contains("12mm"), "got:\n{html}");
    assert!(html.contains("標準ラベル"), "got:\n{html}");
}

#[tokio::test]
async fn status_card_offline_shows_offline_badge() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: false }));
    mock.push_lw_status(Ok(lw_status(263, 16)));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/printers/PR-001/status-card")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(html.contains("Offline"), "got:\n{html}");
    assert!(html.contains("24mm"), "got:\n{html}");
    assert!(html.contains("ケーブル表示ラベル"), "got:\n{html}");
}

#[tokio::test]
async fn status_card_client_error_shows_error_state() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Err(TepraError::Creator { errcode: 500 }));
    mock.push_lw_status(Err(TepraError::Creator { errcode: 500 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/printers/PR-001/status-card")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "client error must still yield 200 OK with error state in-card"
    );
    let html = body_html(response.into_body()).await;
    assert!(html.contains("Offline"), "got:\n{html}");
    assert!(
        html.contains("Cannot connect to TEPRA Creator WebAPI"),
        "got:\n{html}"
    );
}
