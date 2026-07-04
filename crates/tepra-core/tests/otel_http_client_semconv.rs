#![allow(deprecated)]
//! TDD cycle tests for `OTel` HTTP client semantic conventions (Cycle 19–26).
// wiremock spawns a TCP listener; not suitable for miri isolation.
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use opentelemetry::trace::SpanKind;
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use opentelemetry_semantic_conventions::attribute;
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
        .find(|kv| kv.key.as_str() == attribute::URL_FULL)
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
        .find(|kv| kv.key.as_str() == attribute::HTTP_REQUEST_BODY_SIZE)
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
        .find(|kv| kv.key.as_str() == attribute::HTTP_RESPONSE_BODY_SIZE)
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

// ── Cycle 25: 4xx/5xx error path ─────────────────────────────────────────────

#[tokio::test]
async fn get_404_sets_error_type_span_attr() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(ResponseTemplate::new(404).set_body_string("{\"error\":\"not found\"}"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let _ = client.version().await;

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let get_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a GET span");

    let error_type = get_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::ERROR_TYPE)
        .expect("error.type attribute must be present for 404");

    assert_eq!(
        error_type.value.as_str().as_ref(),
        "404",
        "error.type must be the HTTP status code string"
    );
}

#[tokio::test]
async fn get_500_sets_span_status_error() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(ResponseTemplate::new(500).set_body_string("{\"error\":\"internal\"}"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let _ = client.version().await;

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let get_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a GET span");

    assert!(
        matches!(get_span.status, opentelemetry::trace::Status::Error { .. }),
        "span status must be Error for 5xx response"
    );
}

#[tokio::test]
#[traced_test]
async fn get_error_response_body_logged_as_warn() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(ResponseTemplate::new(404).set_body_string("{\"error\":\"not found\"}"))
        .mount(&server)
        .await;

    let client = ReqwestTepraClient::new(server.uri());
    let _ = client.version().await;

    assert!(
        logs_contain("http.response.body"),
        "warn log must contain http.response.body field for error responses"
    );
}

// ── Cycle 24: header allowlist + BLOCK regression ─────────────────────────────

#[tokio::test]
async fn response_header_content_type_is_recorded_in_span() {
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

    let content_type = get_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.response.header.content-type")
        .expect("http.response.header.content-type must be present in span attributes");

    assert_eq!(
        content_type.value.as_str().as_ref(),
        "application/json",
        "http.response.header.content-type must equal 'application/json'"
    );
}

#[tokio::test]
async fn authorization_header_is_not_recorded_in_span() {
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

    let auth_attr = get_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.request.header.authorization");

    assert!(
        auth_attr.is_none(),
        "http.request.header.authorization must NOT be recorded in span (BLOCK list)"
    );
}

// ── Cycle 26: transport error path ───────────────────────────────────────────

#[tokio::test]
async fn transport_connect_error_sets_error_type_connection() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    // Port 1 is reserved and not listening → guaranteed connection refused.
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let _ = client.version().await;

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let get_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a GET span after transport error");

    let error_type = get_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::ERROR_TYPE)
        .expect("error.type attribute must be present for transport error");

    assert_eq!(
        error_type.value.as_str().as_ref(),
        "connection",
        "error.type must be 'connection' for connection-refused transport errors"
    );
}

#[tokio::test]
async fn transport_error_sets_span_status_error() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    // Port 1 is reserved and not listening → guaranteed connection refused.
    let client = ReqwestTepraClient::new("http://127.0.0.1:1");
    let _ = client.version().await;

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let get_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a GET span after transport error");

    assert!(
        matches!(get_span.status, opentelemetry::trace::Status::Error { .. }),
        "span status must be Error for transport failure"
    );
}

// ── Cycle 37: error.type kind 分岐 ────────────────────────────────────────────

#[tokio::test]
async fn transport_timeout_error_sets_error_type_timeout() {
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    // Delayed response longer than the client timeout → reqwest timeout error.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&server)
        .await;

    // 500 ms: enough for localhost TCP connect, shorter than 10 s mock delay.
    let client =
        ReqwestTepraClient::new_with_timeout_for_test(server.uri(), Duration::from_millis(500));
    let _ = client.version().await;

    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let get_span = spans
        .iter()
        .find(|s| s.name == "GET")
        .expect("expected a GET span after timeout error");

    let error_type = get_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::ERROR_TYPE)
        .expect("error.type attribute must be present for timeout error");

    assert_eq!(
        error_type.value.as_str().as_ref(),
        "timeout",
        "error.type must be 'timeout' for reqwest timeout errors"
    );
}
