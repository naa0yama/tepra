//! Cycle 40: `http.server.request.duration` attributes include `http.request.method` and `http.route`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening,
    clippy::panic
)]

use std::sync::Arc;

use axum::{body::Body, http::Request, middleware};
use opentelemetry_sdk::metrics::{
    InMemoryMetricExporter, SdkMeterProvider,
    data::{AggregatedMetrics, MetricData},
};
use opentelemetry_semantic_conventions::metric as semconv;
use tepra::router::build_router;
use tepra_core::client::mock::MockTepraClient;
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse, server_metrics_mw};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

fn metric_provider() -> (SdkMeterProvider, InMemoryMetricExporter) {
    let exporter = InMemoryMetricExporter::default();
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build();
    let provider = SdkMeterProvider::builder().with_reader(reader).build();
    (provider, exporter)
}

#[tokio::test]
async fn server_metric_includes_method_and_route() {
    let (provider, exporter) = metric_provider();
    opentelemetry::global::set_meter_provider(provider.clone());

    let meters = Arc::new(tepra_core::otel::metrics::Meters::new());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![]));

    let app = build_router(mock)
        .layer(middleware::from_fn_with_state(
            Arc::clone(&meters),
            server_metrics_mw,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(OtelHttpServerMakeSpan)
                .on_response(OtelOnResponse),
        );

    let req = Request::builder()
        .uri("/api/printer")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let _body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();

    provider.force_flush().expect("flush failed");

    let metrics = exporter.get_finished_metrics().expect("no metric data");
    let metric = metrics
        .iter()
        .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
        .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
        .find(|m| m.name() == semconv::HTTP_SERVER_REQUEST_DURATION)
        .expect("http.server.request.duration histogram must be emitted");

    let data_point = match metric.data() {
        AggregatedMetrics::F64(MetricData::Histogram(hist)) => {
            hist.data_points().next().expect("no data points").clone()
        }
        other => panic!("unexpected metric type: {other:?}"),
    };

    let attrs: std::collections::HashMap<String, opentelemetry::Value> = data_point
        .attributes()
        .map(|kv| (kv.key.to_string(), kv.value.clone()))
        .collect();

    let method = attrs
        .get("http.request.method")
        .expect("http.request.method must be present");
    assert_eq!(
        method.as_str().as_ref(),
        "GET",
        "http.request.method must be GET"
    );

    let route = attrs.get("http.route").expect("http.route must be present");
    assert!(
        !route.as_str().is_empty(),
        "http.route must not be empty, got: {route}"
    );

    provider.shutdown().unwrap();
}
