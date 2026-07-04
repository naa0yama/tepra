#![allow(deprecated)]
//! TDD Cycle 46-1a Red: span name and url.template attribute assertion for static-path GET callers.
// wiremock spawns a TCP listener; not suitable for miri isolation.
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use opentelemetry_semantic_conventions::attribute;
use tepra_core::{client::ReqwestTepraClient, client::TepraClient, otel::TelemetryGuard};
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
