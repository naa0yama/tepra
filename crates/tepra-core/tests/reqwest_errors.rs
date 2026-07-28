#![allow(deprecated)]
//! Error-path tests for `ReqwestTepraClient` — transport and parse failures.
// wiremock spawns a TCP listener which miri isolation blocks;
// HTTP integration tests are not the target of UB detection.
#![cfg(not(miri))]
#![allow(missing_docs, clippy::unwrap_used)]

use tepra_core::{
    client::{ReqwestTepraClient, TepraClient},
    dto::{
        job::{
            DensityParam, ErrorMessageParam, FilePayload, JobControlRequest, PrintFiles,
            PrintParameter, PrintRequest,
        },
        template::{GetMarginRequest, ImportFrameRequest},
    },
    error::TepraError,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

const PRINTER_NAME: &str = "TestPrinter";
const JOB_ID: u64 = 1;

// ---------------------------------------------------------------------------
// Transport errors (connection refused → TepraError::Transport)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_json_transport_error_on_connection_refused() {
    // Port 1 is never open; connect immediately refused.
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let result = client.list_printers().await;
    assert!(
        matches!(result, Err(TepraError::Transport { .. })),
        "expected Transport error, got: {result:?}"
    );
}

#[tokio::test]
async fn get_query_json_transport_error() {
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let result = client.job_progress(PRINTER_NAME, JOB_ID).await;
    assert!(matches!(result, Err(TepraError::Transport { .. })));
}

#[tokio::test]
async fn get_query_empty_transport_error() {
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let result = client.tapefeed(PRINTER_NAME, true).await;
    assert!(matches!(result, Err(TepraError::Transport { .. })));
}

#[tokio::test]
async fn post_json_transport_error() {
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let result = client
        .import_frame(ImportFrameRequest {
            template_file: FilePayload {
                file_name: "a.lbx".into(),
                base64_str: String::new(),
            },
        })
        .await;
    assert!(matches!(result, Err(TepraError::Transport { .. })));
}

#[tokio::test]
async fn post_empty_transport_error() {
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let result = client
        .job_control(
            PRINTER_NAME,
            JobControlRequest {
                jobid: 0,
                control: 1,
            },
        )
        .await;
    assert!(matches!(result, Err(TepraError::Transport { .. })));
}

// ---------------------------------------------------------------------------
// Parse errors (200 OK with invalid JSON → TepraError::Parse)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_json_parse_error_on_invalid_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(b"not valid json", "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.list_printers().await;
    assert!(
        matches!(result, Err(TepraError::Parse { .. })),
        "expected Parse error, got: {result:?}"
    );
}

#[tokio::test]
async fn get_query_json_parse_error_on_invalid_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/job/progress/{PRINTER_NAME}")))
        .and(query_param("jobid", JOB_ID.to_string().as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(b"not valid json", "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client.job_progress(PRINTER_NAME, JOB_ID).await;
    assert!(matches!(result, Err(TepraError::Parse { .. })));
}

#[tokio::test]
async fn post_json_parse_error_on_invalid_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/printer/template/importframe"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(b"not valid json", "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let result = client
        .import_frame(ImportFrameRequest {
            template_file: FilePayload {
                file_name: "a.lbx".into(),
                base64_str: String::new(),
            },
        })
        .await;
    assert!(matches!(result, Err(TepraError::Parse { .. })));
}

// ---------------------------------------------------------------------------
// Additional GET endpoints — parse errors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn version_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.version().await,
        Err(TepraError::Parse { .. })
    ));
}

#[tokio::test]
async fn autoselect_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/autoselect"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.autoselect().await,
        Err(TepraError::Parse { .. })
    ));
}

#[tokio::test]
async fn printer_info_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/info/{PRINTER_NAME}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.printer_info(PRINTER_NAME).await,
        Err(TepraError::Parse { .. })
    ));
}

#[tokio::test]
async fn online_status_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/onlinestatus/{PRINTER_NAME}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.online_status(PRINTER_NAME).await,
        Err(TepraError::Parse { .. })
    ));
}

#[tokio::test]
async fn lw_status_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/lwstatus/{PRINTER_NAME}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.lw_status(PRINTER_NAME).await,
        Err(TepraError::Parse { .. })
    ));
}

#[allow(clippy::missing_const_for_fn)]
fn minimal_print_req() -> PrintRequest {
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

#[tokio::test]
async fn print_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/printer/print/{PRINTER_NAME}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.print(PRINTER_NAME, minimal_print_req()).await,
        Err(TepraError::Parse { .. })
    ));
}

#[tokio::test]
async fn job_info_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/printer/job/info/{PRINTER_NAME}")))
        .and(query_param("jobid", JOB_ID.to_string().as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    assert!(matches!(
        client.job_info(PRINTER_NAME, JOB_ID).await,
        Err(TepraError::Parse { .. })
    ));
}

#[tokio::test]
async fn get_margin_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/printer/getmargin/{PRINTER_NAME}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"bad", "application/json"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let req = GetMarginRequest {
        tape_id: 261,
        template_file: None,
    };
    assert!(matches!(
        client.get_margin(PRINTER_NAME, req).await,
        Err(TepraError::Parse { .. })
    ));
}
