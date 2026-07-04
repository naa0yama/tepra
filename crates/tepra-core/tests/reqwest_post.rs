#![allow(deprecated)]
//! `WireMock` integration tests for `ReqwestTepraClient` — POST + remaining GET endpoints (RED state).
//!
//! These tests **fail** until T10b implements the HTTP calls in `reqwest_client.rs`.
//! They compile and the mock server / fixture setup is correct; failure is due to
//! `not_implemented` stubs returning errors before the HTTP request is ever issued.
// wiremock spawns a TCP listener which miri isolation blocks;
// HTTP integration tests are not the target of UB detection.
#![cfg(not(miri))]
#![allow(missing_docs)]

use tepra_core::{
    client::ReqwestTepraClient,
    client::TepraClient,
    dto::{
        job::{
            JobControlRequest, JobInfoResponse, JobProgressResponse, PrintRequest, PrintResponse,
        },
        template::{GetMarginRequest, GetMarginResponse, ImportFrameItem, ImportFrameRequest},
    },
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path, query_param},
};

const PRINTER_NAME: &str = "TestPrinter";
const JOB_ID: u64 = 42;

// ---------------------------------------------------------------------------
// POST /api/printer/print/{name} → print (submit_print_job)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn print_sends_post_with_body_and_deserializes() {
    let server = MockServer::start().await;

    let sent_json = include_str!("fixtures/dto/print_req.json");
    let reply_json = include_str!("fixtures/dto/print_res.json");

    let req: PrintRequest = serde_json::from_str(sent_json).unwrap();
    let expected_body = serde_json::to_value(&req).unwrap();

    Mock::given(method("POST"))
        .and(path(format!("/api/printer/print/{PRINTER_NAME}")))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(reply_json, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.print(PRINTER_NAME, req).await.unwrap();

    let expected: PrintResponse = serde_json::from_str(reply_json).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/tapefeed/{name} → tapefeed
// (SDK: ?cutflag=<bool>; trait accepts name only — cutflag wired by implementation)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tapefeed_sends_get_and_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/printer/tapefeed/{PRINTER_NAME}")))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.tapefeed(PRINTER_NAME, false).await.unwrap();

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/job/progress/{name}?jobid=N → job_progress
// ---------------------------------------------------------------------------

#[tokio::test]
async fn job_progress_sends_get_with_query_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/job_progress_res.json");

    Mock::given(method("GET"))
        .and(path(format!("/api/printer/job/progress/{PRINTER_NAME}")))
        .and(query_param("jobid", JOB_ID.to_string().as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.job_progress(PRINTER_NAME, JOB_ID).await.unwrap();

    let expected: JobProgressResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// GET /api/printer/job/info/{name}?jobid=N → job_info
// ---------------------------------------------------------------------------

#[tokio::test]
async fn job_info_sends_get_with_query_and_deserializes() {
    let server = MockServer::start().await;

    let body = include_str!("fixtures/dto/job_info_res.json");

    Mock::given(method("GET"))
        .and(path(format!("/api/printer/job/info/{PRINTER_NAME}")))
        .and(query_param("jobid", JOB_ID.to_string().as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.job_info(PRINTER_NAME, JOB_ID).await.unwrap();

    let expected: JobInfoResponse = serde_json::from_str(body).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// POST /api/printer/job/control/{name} → job_control
// ---------------------------------------------------------------------------

#[tokio::test]
async fn job_control_sends_post_with_body_and_succeeds() {
    let server = MockServer::start().await;

    let sent_json = include_str!("fixtures/dto/job_control_req.json");

    let req: JobControlRequest = serde_json::from_str(sent_json).unwrap();
    let expected_body = serde_json::to_value(&req).unwrap();

    Mock::given(method("POST"))
        .and(path(format!("/api/printer/job/control/{PRINTER_NAME}")))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.job_control(PRINTER_NAME, req).await.unwrap();

    server.verify().await;
}

// ---------------------------------------------------------------------------
// POST /api/printer/template/importframe → import_frame
// ---------------------------------------------------------------------------

#[tokio::test]
async fn import_frame_sends_post_with_body_and_deserializes() {
    let server = MockServer::start().await;

    let sent_json = include_str!("fixtures/dto/import_frame_req.json");
    let reply_json = include_str!("fixtures/dto/import_frame_res.json");

    let req: ImportFrameRequest = serde_json::from_str(sent_json).unwrap();
    let expected_body = serde_json::to_value(&req).unwrap();

    Mock::given(method("POST"))
        .and(path("/api/printer/template/importframe"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(reply_json, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.import_frame(req).await.unwrap();

    let expected: Vec<ImportFrameItem> = serde_json::from_str(reply_json).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}

// ---------------------------------------------------------------------------
// POST /api/printer/getmargin/{name} → get_margin
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_margin_sends_post_with_body_and_deserializes() {
    let server = MockServer::start().await;

    let sent_json = include_str!("fixtures/dto/get_margin_req.json");
    let reply_json = include_str!("fixtures/dto/get_margin_res.json");

    let req: GetMarginRequest = serde_json::from_str(sent_json).unwrap();
    let expected_body = serde_json::to_value(&req).unwrap();

    Mock::given(method("POST"))
        .and(path(format!("/api/printer/getmargin/{PRINTER_NAME}")))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(reply_json, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.get_margin(PRINTER_NAME, req).await.unwrap();

    let expected: GetMarginResponse = serde_json::from_str(reply_json).unwrap();
    assert_eq!(result, expected);

    server.verify().await;
}
