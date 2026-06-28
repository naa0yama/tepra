//! RED unit tests for `/api/printer*` handlers.
//!
//! Each test uses `axum::Router::oneshot` + `MockTepraClient`.
//! Handlers are stubs (`todo!()`), so every test panics → RED.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::significant_drop_tightening
)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tepra_api::router::build_router;
use tepra_core::{
    client::{mock::MockTepraClient, traits::TepraClient},
    dto::{
        printer::{
            AutoselectResponse, DriverVersion, LwStatusResponse, OnlineStatusResponse,
            PrinterInfoResponse, PrinterListItem, TapeEntry, VersionResponse,
        },
        template::{GetMarginRequest, GetMarginResponse},
    },
};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_app(client: Arc<dyn TepraClient>) -> axum::Router {
    build_router(client)
}

async fn body_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

// ---------------------------------------------------------------------------
// 1. GET /api/printer
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_printers_returns_200_json_array() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![PrinterListItem {
        printer_name: "PT-P710BT".into(),
    }]));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json[0]["printerName"], "PT-P710BT");
}

// ---------------------------------------------------------------------------
// 2. GET /api/printer/version
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_version_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_version(Ok(VersionResponse {
        web_api_module: "1.0.0".into(),
        printer_drivers: vec![DriverVersion {
            driver_name: "TEPRA".into(),
            version: "2.0".into(),
        }],
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["webApiModule"], "1.0.0");
}

// ---------------------------------------------------------------------------
// 3. GET /api/printer/autoselect
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_autoselect_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_autoselect(Ok(AutoselectResponse {
        printer_name: "PT-P710BT".into(),
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/autoselect")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["printerName"], "PT-P710BT");
}

// ---------------------------------------------------------------------------
// 4a. GET /api/printer/info/{name} — happy path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_printer_info_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_printer_info(Ok(PrinterInfoResponse {
        driver_name: "TEPRA".into(),
        dpi: 360,
        tape_list: vec![TapeEntry { tape_id: 261 }],
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/info/PT-P710BT")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["dpi"], 360);
}

// ---------------------------------------------------------------------------
// 4b. GET /api/printer/info/{name} — client error → non-200
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_printer_info_client_error_returns_non_200() {
    use tepra_core::error::TepraError;

    let mock = Arc::new(MockTepraClient::new());
    mock.push_printer_info(Err(TepraError::Creator { errcode: 1 }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/info/UNKNOWN")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// 5. GET /api/printer/onlinestatus/{name}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_online_status_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_online_status(Ok(OnlineStatusResponse { online: true }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/onlinestatus/PT-P710BT")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["online"], true);
}

// ---------------------------------------------------------------------------
// 6. GET /api/printer/lwstatus/{name}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_lw_status_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_lw_status(Ok(LwStatusResponse {
        tape_id: 261,
        tape_kind: 0,
        error: 0,
        br_tape_kind: 0,
        status: 0,
        status_type: 4,
        tape_sw: None,
        t8_option: None,
    }));

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .uri("/api/printer/lwstatus/PT-P710BT")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["tapeID"], 261);
}

// ---------------------------------------------------------------------------
// 7. POST /api/printer/getmargin/{name}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_margin_returns_200_json() {
    let mock = Arc::new(MockTepraClient::new());
    mock.push_get_margin(Ok(GetMarginResponse {
        top: 2,
        bottom: 2,
        left_right: 4,
    }));

    let req_body = serde_json::to_vec(&GetMarginRequest {
        tape_id: 261,
        template_file: None,
    })
    .unwrap();

    let response = make_app(mock)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/printer/getmargin/PT-P710BT")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response.into_body()).await;
    assert_eq!(json["leftRight"], 4);
}
