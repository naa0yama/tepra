//! Integration test: http.server.request.duration Histogram is emitted on every request.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening,
    clippy::panic
)]

use std::sync::Arc;

use axum::{body::Body, http::Request};
use opentelemetry_sdk::metrics::{
    InMemoryMetricExporter, SdkMeterProvider,
    data::{AggregatedMetrics, MetricData},
};
use opentelemetry_semantic_conventions::metric as semconv;
use tepra::router::build_router;
use tepra_core::client::mock::MockTepraClient;
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse};
use tower::ServiceExt as _;
use tower_http::trace::TraceLayer;

fn metric_provider() -> (SdkMeterProvider, InMemoryMetricExporter) {
    let exporter = InMemoryMetricExporter::default();
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build();
    let provider = SdkMeterProvider::builder().with_reader(reader).build();
    (provider, exporter)
}

#[tokio::test]
async fn server_request_duration_histogram_recorded() {
    let (provider, exporter) = metric_provider();
    opentelemetry::global::set_meter_provider(provider.clone());

    let meters = Arc::new(tepra_core::otel::metrics::Meters::new());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_list_printers(Ok(vec![]));

    let app = build_router(mock).layer(
        TraceLayer::new_for_http()
            .make_span_with(OtelHttpServerMakeSpan)
            .on_response(OtelOnResponse::new(meters)),
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

    let count = match metric.data() {
        AggregatedMetrics::F64(MetricData::Histogram(hist)) => {
            hist.data_points().next().expect("no data points").count()
        }
        other => panic!("unexpected metric type: {other:?}"),
    };
    assert_eq!(count, 1, "one data point per request");

    provider.shutdown().unwrap();
}
