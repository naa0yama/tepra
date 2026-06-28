//! RED unit tests for HTMX progress polling handler (`GET /ui/jobs/{printer}/{job_id}`).
//!
//! These tests verify that `views::job_card` queries the real job progress from
//! `TepraClient` and reflects it in the returned partial HTML. They are committed
//! in the RED phase: the handler currently returns hardcoded `job_end: false` and
//! does not call the client, so all assertions on polling-stop behaviour fail.
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
    client::{mock::MockTepraClient, traits::TepraClient},
    dto::job::JobProgressResponse,
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

fn in_progress_response() -> JobProgressResponse {
    JobProgressResponse {
        data_progress: 50,
        page_number: 1,
        total_page_count: 2,
        job_end: false,
        canceled: false,
        status_error: 0,
    }
}

fn completed_response() -> JobProgressResponse {
    JobProgressResponse {
        data_progress: 100,
        page_number: 2,
        total_page_count: 2,
        job_end: true,
        canceled: false,
        status_error: 0,
    }
}

fn canceled_response() -> JobProgressResponse {
    JobProgressResponse {
        data_progress: 30,
        page_number: 1,
        total_page_count: 2,
        job_end: false,
        canceled: true,
        status_error: 0,
    }
}

// ---------------------------------------------------------------------------
// 1. In-progress job: polling attributes present
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_card_in_progress_has_hx_trigger() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(in_progress_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("hx-trigger=\"every 1s\""),
        "in-progress card must have HTMX polling trigger; got:\n{html}"
    );
}

#[tokio::test]
async fn test_job_card_in_progress_shows_progress_percent() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(in_progress_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let html = body_html(response.into_body()).await;
    assert!(
        html.contains("50"),
        "in-progress card must show progress percentage; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 2. Completed job: polling must stop
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_card_completed_omits_hx_trigger() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(completed_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        !html.contains("hx-trigger"),
        "completed card must NOT have HTMX polling trigger; got:\n{html}"
    );
}

#[tokio::test]
async fn test_job_card_completed_shows_done_text() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(completed_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let html = body_html(response.into_body()).await;
    // Template shows "完了" when job_end=true
    assert!(
        html.contains("完了"),
        "completed card must show completion text; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 3. Canceled job: polling must stop
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_card_canceled_omits_hx_trigger() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(canceled_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/99")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_html(response.into_body()).await;
    assert!(
        !html.contains("hx-trigger"),
        "canceled card must NOT have HTMX polling trigger; got:\n{html}"
    );
}

#[tokio::test]
async fn test_job_card_canceled_shows_canceled_text() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(canceled_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/99")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let html = body_html(response.into_body()).await;
    // Template shows "キャンセル済み" when canceled=true
    assert!(
        html.contains("キャンセル済み"),
        "canceled card must show cancel text; got:\n{html}"
    );
}

// ---------------------------------------------------------------------------
// 4. Client error: handler returns non-2xx
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_card_client_error_returns_non_200() {
    use tepra_core::error::TepraError;

    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Err(TepraError::Creator { errcode: 500 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/PT-P710BT/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(
        response.status(),
        StatusCode::OK,
        "client error must not yield 200 OK"
    );
}

// ---------------------------------------------------------------------------
// 5. Printer name and job ID are reflected in the card
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_card_reflects_printer_and_job_id() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(in_progress_response()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/ui/jobs/QL-800/7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let html = body_html(response.into_body()).await;
    assert!(html.contains("QL-800"), "card must include printer name");
    assert!(html.contains('7'), "card must include job id");
}
