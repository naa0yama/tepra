//! Cycle 34 M3: Server span 5xx `set_status(Error)` + error.type attribute assertions.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening
)]

use std::sync::Arc;

use axum::{body::Body, http::Request, http::StatusCode, response::IntoResponse, routing::get};
use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use opentelemetry_semantic_conventions::attribute;
use tepra_core::otel::TelemetryGuard;
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

async fn always_500() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn always_200() -> impl IntoResponse {
    StatusCode::OK
}

fn build_test_app() -> axum::Router {
    let router = axum::Router::new()
        .route("/error", get(always_500))
        .route("/ok", get(always_200));
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(OtelHttpServerMakeSpan)
            .on_response(OtelOnResponse::new(Arc::new(
                tepra_core::otel::metrics::Meters::new(),
            ))),
    )
}

#[tokio::test]
async fn server_5xx_sets_span_status_error_and_error_type() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let app = build_test_app();

    let req = Request::builder()
        .uri("/error")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    // Find the server span (has url.scheme attribute from make_span)
    let server_span = spans
        .iter()
        .find(|s| {
            s.attributes
                .iter()
                .any(|kv| kv.key.as_str() == attribute::URL_SCHEME)
        })
        .expect("server span with url.scheme must exist");

    // span.status must be Error
    assert!(
        matches!(
            server_span.status,
            opentelemetry::trace::Status::Error { .. }
        ),
        "span status must be Error for 5xx responses, got {:?}",
        server_span.status
    );

    // error.type must be "500"
    let error_type = server_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == attribute::ERROR_TYPE)
        .expect("error.type attribute must be present for 5xx responses");
    assert_eq!(
        error_type.value.as_str().as_ref(),
        "500",
        "error.type must be the HTTP status code string"
    );
}

#[tokio::test]
async fn server_2xx_does_not_set_span_status_error() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let app = build_test_app();

    let req = Request::builder()
        .uri("/ok")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let server_span = spans
        .iter()
        .find(|s| {
            s.attributes
                .iter()
                .any(|kv| kv.key.as_str() == attribute::URL_SCHEME)
        })
        .expect("server span with url.scheme must exist");

    // span.status must NOT be Error for 2xx
    assert!(
        !matches!(
            server_span.status,
            opentelemetry::trace::Status::Error { .. }
        ),
        "span status must not be Error for 2xx responses, got {:?}",
        server_span.status
    );

    // error.type must NOT be present for 2xx
    let has_error_type = server_span
        .attributes
        .iter()
        .any(|kv| kv.key.as_str() == attribute::ERROR_TYPE);
    assert!(
        !has_error_type,
        "error.type must not be present for 2xx responses"
    );
}
