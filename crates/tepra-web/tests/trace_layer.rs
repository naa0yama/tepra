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
use tepra::router::build_router;
use tepra_core::{
    client::mock::MockTepraClient, dto::printer::PrinterListItem, otel::TelemetryGuard,
};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

#[tokio::test]
async fn traceable_request_emits_span_with_http_method() {
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

    let has_get = spans.iter().any(|s| {
        s.attributes.iter().any(|kv| {
            kv.key.as_str() == "method"
                && matches!(&kv.value, Value::String(v) if v.as_str() == "GET")
        })
    });
    assert!(
        has_get,
        "expected method=GET attribute on request span, got spans: {spans:#?}"
    );
}
