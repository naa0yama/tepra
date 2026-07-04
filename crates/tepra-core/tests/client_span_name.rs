#![allow(deprecated)]
//! TDD Cycle 46-1a/2a/3a Red: span name and url.template assertion for GET and POST callers.
// wiremock spawns a TCP listener; not suitable for miri isolation.
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use opentelemetry_semantic_conventions::attribute;
use tepra_core::{
    client::ReqwestTepraClient,
    client::TepraClient,
    dto::{
        job::{JobControlRequest, PrintRequest},
        template::{GetMarginRequest, ImportFrameRequest},
    },
    otel::TelemetryGuard,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ── Cycle 46-1a: static GET — list_printers ───────────────────────────────────

#[tokio::test]
async fn list_printers_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/printer_list_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .list_printers()
        .await
        .expect("list_printers must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer",
        "span name must be 'GET /api/printer'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer",
        "url.template must be '/api/printer'"
    );
}

// ── Cycle 46-1a: static GET — version ────────────────────────────────────────

#[tokio::test]
async fn version_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/version_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.version().await.expect("version must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/version",
        "span name must be 'GET /api/printer/version'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/version",
        "url.template must be '/api/printer/version'"
    );
}

// ── Cycle 46-1a: static GET — autoselect ─────────────────────────────────────

#[tokio::test]
async fn autoselect_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/autoselect"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/autoselect_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.autoselect().await.expect("autoselect must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/autoselect",
        "span name must be 'GET /api/printer/autoselect'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/autoselect",
        "url.template must be '/api/printer/autoselect'"
    );
}

// ── Cycle 46-2a: dynamic GET — printer_info ───────────────────────────────────

#[tokio::test]
async fn printer_info_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/info/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/printer_info_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .printer_info("dummy")
        .await
        .expect("printer_info must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/info/{name}",
        "span name must be 'GET /api/printer/info/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/info/{name}",
        "url.template must be '/api/printer/info/{{name}}'"
    );
}

// ── Cycle 46-2a: dynamic GET — online_status ─────────────────────────────────

#[tokio::test]
async fn online_status_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/onlinestatus/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/online_status_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .online_status("dummy")
        .await
        .expect("online_status must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/onlinestatus/{name}",
        "span name must be 'GET /api/printer/onlinestatus/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/onlinestatus/{name}",
        "url.template must be '/api/printer/onlinestatus/{{name}}'"
    );
}

// ── Cycle 46-2a: dynamic GET — lw_status ─────────────────────────────────────

#[tokio::test]
async fn lw_status_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/lwstatus/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/lw_status_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .lw_status("dummy")
        .await
        .expect("lw_status must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/lwstatus/{name}",
        "span name must be 'GET /api/printer/lwstatus/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/lwstatus/{name}",
        "url.template must be '/api/printer/lwstatus/{{name}}'"
    );
}

// ── Cycle 46-2a: dynamic GET — tapefeed ──────────────────────────────────────

#[tokio::test]
async fn tapefeed_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/tapefeed/dummy"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .tapefeed("dummy", false)
        .await
        .expect("tapefeed must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/tapefeed/{name}",
        "span name must be 'GET /api/printer/tapefeed/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/tapefeed/{name}",
        "url.template must be '/api/printer/tapefeed/{{name}}'"
    );
}

// ── Cycle 46-2a: dynamic GET — job_progress ──────────────────────────────────

#[tokio::test]
async fn job_progress_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/job/progress/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/job_progress_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .job_progress("dummy", 1)
        .await
        .expect("job_progress must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/job/progress/{name}",
        "span name must be 'GET /api/printer/job/progress/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/job/progress/{name}",
        "url.template must be '/api/printer/job/progress/{{name}}'"
    );
}

// ── Cycle 46-2a: dynamic GET — job_info ──────────────────────────────────────

#[tokio::test]
async fn job_info_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/job/info/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/job_info_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .job_info("dummy", 1)
        .await
        .expect("job_info must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET /"))
        .expect("expected a GET span");

    assert_eq!(
        http_span.name.as_ref(),
        "GET /api/printer/job/info/{name}",
        "span name must be 'GET /api/printer/job/info/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/job/info/{name}",
        "url.template must be '/api/printer/job/info/{{name}}'"
    );
}

// ── Cycle 46-3a: POST — print ─────────────────────────────────────────────────

#[tokio::test]
async fn print_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let req: PrintRequest =
        serde_json::from_str(include_str!("fixtures/dto/print_req.json")).unwrap();
    Mock::given(method("POST"))
        .and(path("/api/printer/print/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/print_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .print("dummy", req)
        .await
        .expect("print must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("POST /"))
        .expect("expected a POST span");

    assert_eq!(
        http_span.name.as_ref(),
        "POST /api/printer/print/{name}",
        "span name must be 'POST /api/printer/print/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/print/{name}",
        "url.template must be '/api/printer/print/{{name}}'"
    );
}

// ── Cycle 46-3a: POST — job_control ──────────────────────────────────────────

#[tokio::test]
async fn job_control_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let req: JobControlRequest =
        serde_json::from_str(include_str!("fixtures/dto/job_control_req.json")).unwrap();
    Mock::given(method("POST"))
        .and(path("/api/printer/job/control/dummy"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .job_control("dummy", req)
        .await
        .expect("job_control must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("POST /"))
        .expect("expected a POST span");

    assert_eq!(
        http_span.name.as_ref(),
        "POST /api/printer/job/control/{name}",
        "span name must be 'POST /api/printer/job/control/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/job/control/{name}",
        "url.template must be '/api/printer/job/control/{{name}}'"
    );
}

// ── Cycle 46-3a: POST — import_frame ─────────────────────────────────────────

#[tokio::test]
async fn import_frame_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let req: ImportFrameRequest =
        serde_json::from_str(include_str!("fixtures/dto/import_frame_req.json")).unwrap();
    Mock::given(method("POST"))
        .and(path("/api/printer/template/importframe"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/import_frame_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .import_frame(req)
        .await
        .expect("import_frame must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("POST /"))
        .expect("expected a POST span");

    assert_eq!(
        http_span.name.as_ref(),
        "POST /api/printer/template/importframe",
        "span name must be 'POST /api/printer/template/importframe'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/template/importframe",
        "url.template must be '/api/printer/template/importframe'"
    );
}

// ── Cycle 46-3a: POST — get_margin ───────────────────────────────────────────

#[tokio::test]
async fn get_margin_span_name_is_method_and_template() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let req: GetMarginRequest =
        serde_json::from_str(include_str!("fixtures/dto/get_margin_req.json")).unwrap();
    Mock::given(method("POST"))
        .and(path("/api/printer/getmargin/dummy"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(
                    include_str!("fixtures/dto/get_margin_res.json"),
                    "application/json",
                ),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client
        .get_margin("dummy", req)
        .await
        .expect("get_margin must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("POST /"))
        .expect("expected a POST span");

    assert_eq!(
        http_span.name.as_ref(),
        "POST /api/printer/getmargin/{name}",
        "span name must be 'POST /api/printer/getmargin/{{name}}'"
    );

    let url_template = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::URL_TEMPLATE);
    assert!(
        url_template.is_some(),
        "url.template attribute must be present"
    );
    assert_eq!(
        url_template.unwrap().value.as_str().as_ref(),
        "/api/printer/getmargin/{name}",
        "url.template must be '/api/printer/getmargin/{{name}}'"
    );
}
