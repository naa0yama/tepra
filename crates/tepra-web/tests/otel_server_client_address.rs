//! Cycle 39: client.address is populated when `ConnectInfo<SocketAddr>` is present.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening
)]

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::ConnectInfo;
use axum::{body::Body, http::Request};
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use tepra::router::build_router;
use tepra_core::{client::mock::MockTepraClient, otel::TelemetryGuard};
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

const CLIENT_ADDR: &str = "client.address";

#[tokio::test]
async fn client_address_is_set_when_connect_info_present() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![]));

    let app = build_router(mock).layer(
        TraceLayer::new_for_http()
            .make_span_with(OtelHttpServerMakeSpan)
            .on_response(OtelOnResponse),
    );

    let peer: SocketAddr = "192.0.2.1:54321".parse().unwrap();
    let mut req = Request::builder()
        .uri("/api/printer")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(ConnectInfo(peer));

    let resp = app.oneshot(req).await.unwrap();
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let server_span = spans
        .iter()
        .find(|s| s.attributes.iter().any(|kv| kv.key.as_str() == CLIENT_ADDR))
        .expect("server span with client.address must exist");

    let client_address = server_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == CLIENT_ADDR)
        .expect("client.address must be present");

    let addr_str = client_address.value.as_str();
    assert!(
        !addr_str.is_empty(),
        "client.address must not be empty when ConnectInfo is present"
    );
    assert!(
        addr_str.parse::<IpAddr>().is_ok(),
        "client.address must be a valid IP address, got: {addr_str}"
    );
    assert_eq!(
        addr_str.as_ref(),
        "192.0.2.1",
        "client.address must match peer IP"
    );
}

#[tokio::test]
async fn client_address_is_empty_when_connect_info_absent() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![]));

    let app = build_router(mock).layer(
        TraceLayer::new_for_http()
            .make_span_with(OtelHttpServerMakeSpan)
            .on_response(OtelOnResponse),
    );

    // No ConnectInfo extension — simulates missing into_make_service_with_connect_info.
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

    // Span may omit client.address entirely, or set it to empty string.
    let empty_or_absent = spans.iter().any(|s| {
        let val = s
            .attributes
            .iter()
            .find(|kv| kv.key.as_str() == CLIENT_ADDR)
            .map(|kv| kv.value.as_str().is_empty());
        // absent (None) or empty (Some(true))
        val.unwrap_or(true)
    });
    assert!(
        empty_or_absent,
        "client.address must be empty when ConnectInfo is absent"
    );
}
