//! Integration test: [`ReqwestTepraClient`] emits `OTel` HTTP client spans and metrics.
// wiremock spawns a TCP listener; not suitable for miri isolation.
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use opentelemetry::Value;
use opentelemetry_sdk::{
    Resource,
    metrics::{
        InMemoryMetricExporter, PeriodicReader, SdkMeterProvider,
        data::{AggregatedMetrics, MetricData, ResourceMetrics, ScopeMetrics},
    },
    trace::InMemorySpanExporterBuilder,
};
use opentelemetry_semantic_conventions::{attribute, metric as semconv};
use std::sync::Arc;

use tepra_core::{
    client::ReqwestTepraClient,
    client::TepraClient,
    otel::{TelemetryGuard, metrics::Meters},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn build_metric_provider() -> (InMemoryMetricExporter, SdkMeterProvider) {
    let exporter = InMemoryMetricExporter::default();
    let reader = PeriodicReader::builder(exporter.clone()).build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::builder_empty().build())
        .build();
    (exporter, provider)
}

#[tokio::test]
async fn get_request_emits_http_client_span_and_metric() {
    // 1. Meter provider must be set BEFORE ReqwestTepraClient::new() so Meters::new() picks it up.
    let (metric_exporter, meter_provider) = build_metric_provider();
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    // 2. Span subscriber (tracing → OTel bridge).
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    // 3. WireMock server for GET /api/printer/version.
    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    // 4. Client created after global provider is set.
    let client = ReqwestTepraClient::new(server.uri());
    client.version().await.expect("version() must succeed");

    // ── Span assertions ──────────────────────────────────────────────────────
    let spans = span_exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    let http_span = spans
        .iter()
        .find(|s| s.name.starts_with("GET") || s.name.starts_with("POST"))
        .expect("expected an HTTP client span");

    let attrs: std::collections::HashMap<&str, &Value> = http_span
        .attributes
        .iter()
        .map(|kv| (kv.key.as_str(), &kv.value))
        .collect();

    assert_eq!(
        attrs.get(attribute::HTTP_REQUEST_METHOD),
        Some(&&Value::String("GET".into())),
        "http.request.method must be GET"
    );
    assert_eq!(
        attrs.get(attribute::HTTP_RESPONSE_STATUS_CODE),
        Some(&&Value::I64(200)),
        "http.response.status_code must be 200"
    );
    assert!(
        attrs.contains_key(attribute::SERVER_ADDRESS),
        "server.address attribute missing"
    );
    assert_eq!(
        attrs.get(attribute::URL_SCHEME),
        Some(&&Value::String("http".into())),
        "url.scheme must be http"
    );

    // ── Metric assertions ────────────────────────────────────────────────────
    meter_provider.force_flush().expect("flush failed");
    let metrics = metric_exporter
        .get_finished_metrics()
        .expect("no metric data");

    let duration_metric = metrics
        .iter()
        .flat_map(ResourceMetrics::scope_metrics)
        .flat_map(ScopeMetrics::metrics)
        .find(|m| m.name() == semconv::HTTP_CLIENT_REQUEST_DURATION)
        .expect("http.client.request.duration metric not recorded");

    let count = match duration_metric.data() {
        AggregatedMetrics::F64(MetricData::Histogram(h)) => {
            h.data_points().next().expect("no data points").count()
        }
        other => panic!("unexpected metric type: {other:?}"),
    };
    assert_eq!(
        count, 1,
        "expected exactly 1 data point in http.client.request.duration"
    );

    meter_provider.shutdown().expect("meter shutdown failed");
}

// ── Cycle 36: Meters DI via with_meters ───────────────────────────────────────

#[tokio::test]
async fn with_meters_uses_injected_meters_instance() {
    // 1. Build an isolated meter provider with InMemory exporter.
    let (metric_exporter, meter_provider) = build_metric_provider();
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    // 2. Create Meters from the provider's meter — replicates what main.rs does.
    let meters = Arc::new(Meters::new());

    // 3. Span subscriber.
    let span_exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(span_exporter.clone());

    // 4. WireMock server.
    let server = MockServer::start().await;
    let body = include_str!("fixtures/dto/version_res.json");
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(body, "application/json"),
        )
        .mount(&server)
        .await;

    // 5. Client created via with_meters — injects the shared Meters instance.
    let client = ReqwestTepraClient::with_meters(server.uri(), Arc::clone(&meters));
    client.version().await.expect("version() must succeed");

    // 6. Flush and assert the metric was recorded via the injected Meters.
    meter_provider.force_flush().expect("flush failed");
    let metrics = metric_exporter
        .get_finished_metrics()
        .expect("no metric data");

    let duration_metric = metrics
        .iter()
        .flat_map(ResourceMetrics::scope_metrics)
        .flat_map(ScopeMetrics::metrics)
        .find(|m| m.name() == semconv::HTTP_CLIENT_REQUEST_DURATION)
        .expect("http.client.request.duration not recorded via injected Meters");

    let count = match duration_metric.data() {
        AggregatedMetrics::F64(MetricData::Histogram(h)) => {
            h.data_points().next().expect("no data points").count()
        }
        other => panic!("unexpected metric type: {other:?}"),
    };
    assert_eq!(count, 1, "injected Meters must record exactly 1 data point");

    meter_provider.shutdown().expect("meter shutdown failed");
}
