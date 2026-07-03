//! Integration test: handler `#[instrument]` creates child spans with `OTel` HTTP semconv attributes.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening,
    clippy::literal_string_with_formatting_args
)]

use std::sync::Arc;

use axum::{body::Body, http::Request};
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use tepra::router::build_router;
use tepra_core::{
    client::mock::MockTepraClient, dto::printer::PrinterListItem, otel::TelemetryGuard,
};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

#[tokio::test]
async fn handler_emits_child_span_with_http_semconv_attrs() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![PrinterListItem {
        printer_name: "test-printer".into(),
    }]));

    let app = build_router(mock).layer(TraceLayer::new_for_http());

    let req = Request::builder()
        .uri("/api/printer")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    // Must have at least 2 spans: TraceLayer root + handler child.
    assert!(
        spans.len() >= 2,
        "expected at least 2 spans (TraceLayer + handler), got {}: {spans:#?}",
        spans.len()
    );

    // Find the handler span by name.
    let handler_span = spans
        .iter()
        .find(|s| s.name == "handler.list_printers")
        .expect("handler.list_printers span must exist");

    // Assert http.request.method = "GET".
    let method_attr = handler_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.request.method");
    assert!(
        method_attr.is_some(),
        "handler span must have http.request.method attribute, got: {handler_span:#?}"
    );
    assert_eq!(method_attr.unwrap().value.as_str().as_ref(), "GET");

    // Assert http.route attribute is present.
    let route_attr = handler_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.route");
    assert!(
        route_attr.is_some(),
        "handler span must have http.route attribute, got: {handler_span:#?}"
    );
}
