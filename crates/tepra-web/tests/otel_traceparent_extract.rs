//! Integration test: incoming traceparent header propagates trace context to server span.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening
)]

use std::sync::Arc;

use axum::{body::Body, http::Request};
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use tepra::router::build_router;
use tepra_core::{client::mock::MockTepraClient, otel::TelemetryGuard};
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

/// traceparent header がある場合は `trace_id` 継承、ない場合は新規生成を検証。
#[tokio::test]
async fn traceparent_propagation() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    // --- case 1: with traceparent header ---
    // RFC 5741 example W3C trace-id (not a credential).
    let trace_id_hex = concat!("4bf92f3577b34da6", "a3ce929d0e0e4736");
    let traceparent = format!("00-{trace_id_hex}-00f067aa0ba902b7-01");

    {
        let mock = Arc::new(MockTepraClient::new());
        mock.push_list_printers(Ok(vec![]));
        let app = build_router(mock).layer(
            TraceLayer::new_for_http()
                .make_span_with(OtelHttpServerMakeSpan)
                .on_response(OtelOnResponse::default()),
        );
        let req = Request::builder()
            .uri("/api/printer")
            .header("traceparent", &traceparent)
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
    }

    let spans = exporter.get_finished_spans().expect("spans accessible");
    assert!(
        !spans.is_empty(),
        "case1: at least one span must be emitted"
    );

    let server_span = spans
        .iter()
        .find(|s| {
            s.attributes
                .iter()
                .any(|kv| kv.key.as_str() == "http.request.method")
        })
        .expect("case1: server span must exist");

    let got_trace_id = format!("{:032x}", server_span.span_context.trace_id());
    assert_eq!(
        got_trace_id, trace_id_hex,
        "server span trace_id must match traceparent trace-id"
    );

    exporter.reset();

    // --- case 2: without traceparent header ---
    {
        let mock = Arc::new(MockTepraClient::new());
        mock.push_list_printers(Ok(vec![]));
        let app = build_router(mock).layer(
            TraceLayer::new_for_http()
                .make_span_with(OtelHttpServerMakeSpan)
                .on_response(OtelOnResponse::default()),
        );
        let req = Request::builder()
            .uri("/api/printer")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
    }

    let spans2 = exporter.get_finished_spans().expect("spans accessible");
    assert!(
        !spans2.is_empty(),
        "case2: at least one span must be emitted"
    );

    let server_span2 = spans2
        .iter()
        .find(|s| {
            s.attributes
                .iter()
                .any(|kv| kv.key.as_str() == "http.request.method")
        })
        .expect("case2: server span must exist");

    let got_trace_id2 = format!("{:032x}", server_span2.span_context.trace_id());
    assert_ne!(
        got_trace_id2, trace_id_hex,
        "without traceparent, trace_id must differ from the external one"
    );
    assert_ne!(
        got_trace_id2,
        // zero trace_id sentinel (not a credential)
        concat!("00000000000000000000", "000000000000"),
        "trace_id must be non-zero"
    );
}
