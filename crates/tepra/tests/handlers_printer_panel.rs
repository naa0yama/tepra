//! Unit tests for the merge-print printer info panel HTMX partial
//! (`GET /ui/print/{printer}/panel`).
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
    dto::{
        printer::{LwStatusResponse, OnlineStatusResponse},
        template::GetMarginResponse,
    },
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
    lw_status_with_error(tape_id, tape_kind, 0)
}

fn lw_status_with_error(tape_id: u32, tape_kind: i32, error: u32) -> LwStatusResponse {
    LwStatusResponse {
        tape_id,
        tape_kind,
        error,
        br_tape_kind: 0,
        status: 0,
        status_type: 4,
        tape_sw: None,
        t8_option: None,
    }
}

#[tokio::test]
async fn printer_panel_online_shows_tape_and_margin() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Ok(lw_status(261, 0)));
    mock.push_get_margin(Ok(GetMarginResponse {
        top: 15,
        bottom: 15,
        left_right: 5,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
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
    assert!(html.contains("1.5mm"), "got:\n{html}");
    assert!(html.contains("0.5mm"), "got:\n{html}");
}

#[tokio::test]
async fn printer_panel_offline_shows_offline_badge() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: false }));
    mock.push_lw_status(Ok(lw_status(263, 16)));
    mock.push_get_margin(Ok(GetMarginResponse {
        top: 15,
        bottom: 15,
        left_right: 5,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
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
async fn printer_panel_status_error_shows_error_state() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Err(TepraError::Creator { errcode: 500 }));
    mock.push_lw_status(Err(TepraError::Creator { errcode: 500 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "client error must still yield 200 OK with error state in-panel"
    );
    let html = body_html(response.into_body()).await;
    assert!(html.contains("Offline"), "got:\n{html}");
    assert!(
        html.contains("Cannot connect to TEPRA Creator WebAPI"),
        "got:\n{html}"
    );
}

#[tokio::test]
async fn printer_panel_lw_status_404_shows_neutral_busy_notice() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Err(TepraError::Http { status: 404 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        !html.contains("Cannot connect to TEPRA Creator WebAPI"),
        "404 busy must not show the connection-error message; got:\n{html}"
    );
    assert!(
        html.contains("印刷中はステータスを取得できません"),
        "404 busy must show the neutral busy notice; got:\n{html}"
    );
    assert!(
        html.contains("Online"),
        "printer is still reachable while busy printing; got:\n{html}"
    );
}

#[tokio::test]
async fn printer_panel_offline_lw_status_404_shows_neutral_offline() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: false }));
    mock.push_lw_status(Err(TepraError::Http { status: 404 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        !html.contains("Cannot connect to TEPRA Creator WebAPI"),
        "offline 404 must not show the connection-error message; got:\n{html}"
    );
    assert!(
        !html.contains("印刷中はステータスを取得できません"),
        "offline 404 must not show the busy-printing notice; got:\n{html}"
    );
    assert!(html.contains("Offline"), "got:\n{html}");
}

#[tokio::test]
async fn printer_panel_device_error_shows_warning() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Ok(lw_status_with_error(261, 0, 0x21)));
    mock.push_get_margin(Ok(GetMarginResponse {
        top: 15,
        bottom: 15,
        left_right: 5,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("カバーが開いています"),
        "device error must render the warning label; got:\n{html}"
    );
    assert!(
        html.contains("text-warning"),
        "device error must render in the warning color; got:\n{html}"
    );
}

#[tokio::test]
async fn printer_panel_no_device_error_shows_no_warning() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Ok(lw_status(261, 0)));
    mock.push_get_margin(Ok(GetMarginResponse {
        top: 15,
        bottom: 15,
        left_right: 5,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        !html.contains("text-warning"),
        "NoError must not render a device warning; got:\n{html}"
    );
}

#[tokio::test]
async fn printer_panel_lw_status_transport_error_shows_error_state() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Err(TepraError::Transport {
        source: anyhow::anyhow!("connection refused"),
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("Cannot connect to TEPRA Creator WebAPI"),
        "true transport failure must keep the red error message; got:\n{html}"
    );
}

#[tokio::test]
async fn printer_panel_margin_error_shows_error_state() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));
    mock.push_lw_status(Ok(lw_status(261, 0)));
    mock.push_get_margin(Err(TepraError::Creator { errcode: 500 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/print/PR-001/panel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("Online"),
        "margin-only failure must not flip the badge to Offline; got:\n{html}"
    );
    assert!(
        html.contains("標準ラベル"),
        "margin-only failure must not hide already-fetched tape info; got:\n{html}"
    );
    assert!(
        html.contains("Cannot connect to TEPRA Creator WebAPI"),
        "margin fetch failure must surface as an error message; got:\n{html}"
    );
}
