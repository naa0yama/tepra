//! Integration test: `TraceLayer` spans are captured by the `tracing_opentelemetry` subscriber.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening
)]

use std::sync::Arc;

use axum::{body::Body, http::Request};
use opentelemetry::Value;
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use opentelemetry_semantic_conventions::attribute;
use tepra::router::build_router;
use tepra_core::{
    client::mock::MockTepraClient, dto::printer::PrinterListItem, otel::TelemetryGuard,
};
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

#[tokio::test]
async fn server_span_has_otel_http_semconv_attrs() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![PrinterListItem {
        printer_name: "test-printer".into(),
    }]));

    let app = build_router(mock).layer(
        TraceLayer::new_for_http()
            .make_span_with(OtelHttpServerMakeSpan)
            .on_response(OtelOnResponse),
    );

    let req = Request::builder()
        .uri("/api/printer")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // Consume body so TraceLayer completes its span lifecycle.
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");
    assert!(
        !spans.is_empty(),
        "TraceLayer must emit at least one span per request"
    );

    // Find server span by checking for http.request.method (OTel semconv name).
    let server_span = spans
        .iter()
        .find(|s| {
            s.attributes
                .iter()
                .any(|kv| kv.key.as_str() == attribute::HTTP_REQUEST_METHOD)
        })
        .expect("server span with http.request.method attribute must exist");

    let method_attr = server_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::HTTP_REQUEST_METHOD)
        .expect("http.request.method must be present");
    assert_eq!(
        method_attr.value.as_str().as_ref(),
        "GET",
        "http.request.method must be GET"
    );

    let status_attr = server_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::HTTP_RESPONSE_STATUS_CODE);
    assert!(
        status_attr.is_some(),
        "http.response.status_code must be present on server span, got: {server_span:#?}"
    );
    assert_eq!(
        status_attr.unwrap().value,
        Value::I64(200),
        "http.response.status_code must be 200"
    );
}
