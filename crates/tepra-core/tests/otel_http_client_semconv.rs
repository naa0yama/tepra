//! TDD cycle tests for `OTel` HTTP client semantic conventions (Cycle 19–26).
// wiremock spawns a TCP listener; not suitable for miri isolation.
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use opentelemetry::trace::SpanKind;
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use tepra_core::{client::ReqwestTepraClient, client::TepraClient, otel::TelemetryGuard};
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
