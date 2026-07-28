//! RED unit tests for job-related `/api/printer/*` handlers.
//!
//! Each test uses `axum::Router::oneshot` + `MockTepraClient`.
//! Handlers marked `todo!()` panic → all tests FAIL (RED).
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::significant_drop_tightening,
    clippy::missing_const_for_fn
)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tepra::{router::build_jobs_router, state::AppState};
use tepra_core::{
    client::{mock::MockTepraClient, traits::TepraClient},
    dto::job::{
        DensityParam, ErrorMessageParam, JobControlRequest, JobInfoResponse, JobProgressResponse,
        PrintFiles, PrintParameter, PrintRequest, PrintResponse,
    },
};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_app(client: Arc<dyn TepraClient>) -> axum::Router {
    build_jobs_router(AppState::new(client))
}

async fn body_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn minimal_print_request() -> PrintRequest {
    PrintRequest {
        print_file: PrintFiles {
            template_file: None,
            csv_file: None,
            image_file: None,
        },
        print_parameter: PrintParameter {
            copies: 1,
            tape_cut: 2,
            half_cut: 1,
            print_speed: 1,
            density: DensityParam { mode: 1, value: 0 },
            tape_id: 261,
            priority_cut_setting: 1,
            half_cut_separate: 1,
            margin_left_right: 0,
            display_tape_width: 1,
            error_message: ErrorMessageParam {
                mode: 1,
                file_output: 0,
                file_path: String::new(),
            },
            display_transfer_tape: 1,
            display_print_setting: 1,
            cut_title: 0,
            kana_zen: 0,
            display_print_preview: 1,
            stretch_image: 0,
        },
    }
}

// ---------------------------------------------------------------------------
// 1. POST /api/printer/print/{name}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_print_returns_200_with_jobid() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_print(Ok(PrintResponse {
        result: 1,
        jobid: 42,
    }));

    let req_body = serde_json::to_vec(&minimal_print_request()).unwrap();

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/print/PT-P710BT")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["jobid"], 42u64);
    assert_eq!(json["result"], 1u32);
}

#[tokio::test]
async fn test_print_client_error_returns_non_200() {
    use tepra_core::error::TepraError;

    let mock = Arc::new(MockTepraClient::new());
    mock.push_print(Err(TepraError::Creator { errcode: 1 }));

    let req_body = serde_json::to_vec(&minimal_print_request()).unwrap();

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/print/PT-P710BT")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// 2. GET /api/printer/tapefeed/{name}?cutflag=true
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_tapefeed_returns_200() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_tapefeed(Ok(()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/tapefeed/PT-P710BT?cutflag=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_tapefeed_no_cut_returns_200() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_tapefeed(Ok(()));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/tapefeed/PT-P710BT?cutflag=false")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// 3. GET /api/printer/job/progress/{name}?jobid=N
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_progress_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(JobProgressResponse {
        data_progress: 50,
        page_number: 1,
        total_page_count: 2,
        job_end: false,
        canceled: false,
        status_error: 0,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/job/progress/PT-P710BT?jobid=42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["dataProgress"], 50u32);
    assert_eq!(json["jobEnd"], false);
}

#[tokio::test]
async fn test_job_progress_job_end_true() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_progress(Ok(JobProgressResponse {
        data_progress: 100,
        page_number: 2,
        total_page_count: 2,
        job_end: true,
        canceled: false,
        status_error: 0,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/job/progress/PT-P710BT?jobid=42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["jobEnd"], true);
}

// ---------------------------------------------------------------------------
// 4. GET /api/printer/job/info/{name}?jobid=N
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_info_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_info(Ok(JobInfoResponse { status: 0 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/job/info/PT-P710BT?jobid=42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["status"], 0u32);
}

// ---------------------------------------------------------------------------
// 5. POST /api/printer/job/control/{name}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_job_control_cancel_returns_200() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_control(Ok(()));

    let req_body = serde_json::to_vec(&JobControlRequest {
        jobid: 42,
        control: 3,
    })
    .unwrap();

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/job/control/PT-P710BT")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_job_control_error_returns_non_200() {
    use tepra_core::error::TepraError;

    let mock = Arc::new(MockTepraClient::new());
    mock.push_job_control(Err(TepraError::Creator { errcode: 404 }));

    let req_body = serde_json::to_vec(&JobControlRequest {
        jobid: 99,
        control: 3,
    })
    .unwrap();

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/job/control/PT-P710BT")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::OK);
}
