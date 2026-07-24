//! Cycle 32 H2: Server span extended HTTP semconv attribute assertions.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening
)]

use std::sync::Arc;

use axum::{body::Body, http::Request};
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use opentelemetry_semantic_conventions::attribute;
use tepra::router::build_router;
use tepra_core::{client::mock::MockTepraClient, otel::TelemetryGuard};
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

#[tokio::test]
async fn server_span_records_extended_http_attrs() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![]));

    let app = build_router(mock).layer(
        TraceLayer::new_for_http()
            .make_span_with(OtelHttpServerMakeSpan)
            .on_response(OtelOnResponse),
    );

    let req = Request::builder()
        .uri("/api/printer?limit=10")
        .header("host", "example.com")
        .header("user-agent", "TestAgent/1.0")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let server_span = spans
        .iter()
        // Find the TraceLayer server span via url.scheme (only make_span emits it).
        .find(|s| {
            s.attributes
                .iter()
                .any(|kv| kv.key.as_str() == attribute::URL_SCHEME)
        })
        .expect("server span with url.scheme must exist");

    let attrs: std::collections::HashMap<&str, _> = server_span
        .attributes
        .iter()
        .map(|kv| (kv.key.as_str(), &kv.value))
        .collect();

    // url.full: scheme + host + path + query
    let url_full = attrs
        .get(attribute::URL_FULL)
        .expect("url.full must be present");
    assert_eq!(
        url_full.as_str().as_ref(),
        "http://example.com/api/printer?limit=10",
        "url.full must include scheme, host, path, and query"
    );

    // url.path
    let url_path = attrs
        .get(attribute::URL_PATH)
        .expect("url.path must be present");
    assert_eq!(
        url_path.as_str().as_ref(),
        "/api/printer",
        "url.path must match request path"
    );

    // url.query
    let url_query = attrs
        .get(attribute::URL_QUERY)
        .expect("url.query must be present");
    assert_eq!(
        url_query.as_str().as_ref(),
        "limit=10",
        "url.query must match request query string"
    );

    // user_agent.original
    let user_agent = attrs
        .get(attribute::USER_AGENT_ORIGINAL)
        .expect("user_agent.original must be present");
    assert_eq!(
        user_agent.as_str().as_ref(),
        "TestAgent/1.0",
        "user_agent.original must match User-Agent header"
    );

    // server.address — from Host header
    let server_address = attrs
        .get(attribute::SERVER_ADDRESS)
        .expect("server.address must be present");
    assert_eq!(
        server_address.as_str().as_ref(),
        "example.com",
        "server.address must be the Host header value"
    );

    // network.protocol.version
    assert!(
        attrs.contains_key(attribute::NETWORK_PROTOCOL_VERSION),
        "network.protocol.version must be present"
    );
}
