#![allow(deprecated)]
//! `WireMock` integration tests for `ReqwestTepraClient` — GET endpoints (RED state).
//!
//! These tests **fail** until T9b implements the HTTP calls in `reqwest_client.rs`.
//! They compile and the mock server / fixture setup is correct; failure is due to
//! `todo!()` stubs panicking before the HTTP request is ever issued.
// wiremock spawns a TCP listener which miri isolation blocks;
// HTTP integration tests are not the target of UB detection.
#![cfg(not(miri))]
#![allow(missing_docs)]

use tepra_core::{
    client::ReqwestTepraClient,
    client::TepraClient,
    dto::printer::{
        AutoselectResponse, LwStatusResponse, OnlineStatusResponse, PrinterInfoResponse,
        PrinterListItem, VersionResponse,
    },
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const PRINTER_NAME: &str = "TestPrinter";

// ---------------------------------------------------------------------------
// GET /api/printer → list_printers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_printers_sends_get_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/printer_list_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.list_printers().await.unwrap();

    let expected: Vec<PrinterListItem> = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/version → version
// ---------------------------------------------------------------------------

#[tokio::test]
async fn version_sends_get_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.version().await.unwrap();

    let expected: VersionResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/autoselect → autoselect
// ---------------------------------------------------------------------------

#[tokio::test]
async fn autoselect_sends_get_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/autoselect_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/autoselect"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.autoselect().await.unwrap();

    let expected: AutoselectResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/info/{name} → printer_info
// ---------------------------------------------------------------------------

#[tokio::test]
async fn printer_info_sends_get_with_name_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/printer_info_res.json");
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/info/{PRINTER_NAME}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.printer_info(PRINTER_NAME).await.unwrap();

    let expected: PrinterInfoResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/onlinestatus/{name} → online_status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn online_status_sends_get_with_name_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/online_status_res.json");
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/onlinestatus/{PRINTER_NAME}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.online_status(PRINTER_NAME).await.unwrap();

    let expected: OnlineStatusResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/lwstatus/{name} → lw_status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn lw_status_sends_get_with_name_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/lw_status_res.json");
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/lwstatus/{PRINTER_NAME}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.lw_status(PRINTER_NAME).await.unwrap();

    let expected: LwStatusResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}
