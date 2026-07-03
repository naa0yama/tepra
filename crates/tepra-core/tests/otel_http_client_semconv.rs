//! TDD cycle tests for `OTel` HTTP client semantic conventions (Cycle 19–26).
// wiremock spawns a TCP listener; not suitable for miri isolation.
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use opentelemetry::trace::SpanKind;
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use tepra_core::{client::ReqwestTepraClient, client::TepraClient, otel::TelemetryGuard};
use tracing_test::traced_test;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ── Cycle 19: SpanKind::Client ────────────────────────────────────────────────

#[tokio::test]
async fn http_client_spans_have_span_kind_client() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.version().await.expect("version() must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| matches!(s.name.as_ref(), "GET" | "POST"))
        .expect("expected an HTTP client span");

    assert_eq!(
        http_span.span_kind,
        SpanKind::Client,
        "HTTP client span must have SpanKind::Client"
    );
}

// ── Cycle 20: span name = method only ────────────────────────────────────────

#[tokio::test]
async fn http_client_span_name_is_method_only() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.version().await.expect("version() must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a span named exactly 'GET'");

    assert_eq!(http_span.name.as_ref(), "GET", "span name must be 'GET'");
}

// ── Cycle 21: url.full attribute ──────────────────────────────────────────────

#[tokio::test]
async fn http_client_span_records_url_full() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    let base_url = server.uri();
    let expected_url = format!("{base_url}/api/printer/version");

    let client = ReqwestTepraClient::new(&base_url);
    client.version().await.expect("version() must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a span named 'GET'");

    let url_full = http_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "url.full")
        .expect("url.full attribute must be present");

    assert_eq!(
        url_full.value.as_str().as_ref(),
        expected_url.as_str(),
        "url.full must be the full request URL"
    );
}

// ── Cycle 22: http.request.body.size span attribute ───────────────────────────

#[tokio::test]
async fn post_json_records_request_body_size() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let res_body = include_str!("fixtures/dto/import_frame_res.json");
    Mock::given(method("POST"))
        .and(path("/api/printer/template/importframe"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(res_body, "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let req = tepra_core::dto::template::ImportFrameRequest {
        template_file: tepra_core::dto::job::FilePayload {
            file_name: "frame.lbx".into(),
            base64_str: "dGVzdA==".into(),
        },
    };
    client
        .import_frame(req)
        .await
        .expect("import_frame() must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let post_span = spans
        .iter()
        .find(|s| s.name == "POST")
        .expect("expected a POST span");

    let body_size = post_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.request.body.size")
        .expect("http.request.body.size attribute must be present");

    assert!(
        matches!(body_size.value, opentelemetry::Value::I64(n) if n > 0),
        "http.request.body.size must be a positive integer"
    );
}

// ── Cycle 22: debug log contains http.request.body field ──────────────────────

#[tokio::test]
#[traced_test]
async fn post_json_logs_request_body() {
    let server = MockServer::start().await;
    let res_body = include_str!("fixtures/dto/import_frame_res.json");
    Mock::given(method("POST"))
        .and(path("/api/printer/template/importframe"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(res_body, "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let req = tepra_core::dto::template::ImportFrameRequest {
        template_file: tepra_core::dto::job::FilePayload {
            file_name: "frame.lbx".into(),
            base64_str: "dGVzdA==".into(),
        },
    };
    client
        .import_frame(req)
        .await
        .expect("import_frame() must succeed");

    assert!(
        logs_contain("http.request.body"),
        "debug log must contain http.request.body field"
    );
}

// ── Cycle 23: http.response.body.size span attribute (GET 200) ────────────────

#[tokio::test]
async fn get_json_records_response_body_size() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.version().await.expect("version() must succeed");

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let get_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a GET span");

    let body_size = get_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.response.body.size")
        .expect("http.response.body.size attribute must be present");

    assert!(
        matches!(body_size.value, opentelemetry::Value::I64(n) if n > 0),
        "http.response.body.size must be a positive integer"
    );
}

// ── Cycle 23: debug log contains http.response.body field (GET 200) ───────────

#[tokio::test]
#[traced_test]
async fn get_json_logs_response_body() {
    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    client.version().await.expect("version() must succeed");

    assert!(
        logs_contain("http.response.body"),
        "debug log must contain http.response.body field"
    );
}
