//! OTLP metric exporter and [`SdkMeterProvider`] builder.

use anyhow::Context as _;
use opentelemetry_sdk::{
    Resource,
    metrics::{
        Aggregation, Instrument, SdkMeterProvider, Stream,
        periodic_reader_with_async_runtime::PeriodicReader,
    },
    runtime,
};
use opentelemetry_semantic_conventions::metric::{
    HTTP_CLIENT_REQUEST_DURATION, HTTP_SERVER_REQUEST_DURATION,
};

/// Semconv recommended HTTP latency bucket boundaries (seconds).
/// <https://opentelemetry.io/docs/specs/semconv/http/http-metrics/>
const HTTP_LATENCY_BOUNDARIES: [f64; 14] = [
    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

fn http_latency_view(i: &Instrument) -> Option<Stream> {
    if i.name() == HTTP_SERVER_REQUEST_DURATION || i.name() == HTTP_CLIENT_REQUEST_DURATION {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: HTTP_LATENCY_BOUNDARIES.to_vec(),
                record_min_max: true,
            })
            .build()
            .ok()
    } else {
        None
    }
}

/// Build a production [`SdkMeterProvider`] with an OTLP HTTP/protobuf metric exporter.
///
/// The OTLP endpoint is resolved from `OTEL_EXPORTER_OTLP_ENDPOINT` at call time.
/// No endpoint argument is accepted; the env-var route lets the SDK append `/v1/metrics`
/// automatically.
///
/// # Errors
///
/// Returns an error if the OTLP HTTP exporter cannot be constructed.
pub fn build(resource: Resource) -> anyhow::Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .build()
        .context("failed to build OTLP HTTP metric exporter")?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .with_view(http_latency_view)
        .build())
}

/// Build a test [`SdkMeterProvider`] with a caller-supplied exporter.
///
/// The exporter is wrapped in a [`PeriodicReader`]; call `provider.force_flush()` before
/// asserting to ensure buffered data points are flushed to the exporter.
#[cfg(test)]
pub(crate) fn build_for_test(
    resource: Resource,
    exporter: opentelemetry_sdk::metrics::InMemoryMetricExporter,
) -> SdkMeterProvider {
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter).build();
    SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .with_view(http_latency_view)
        .build()
}

// PeriodicReader calls readlink (process env detection) which miri isolation blocks;
// metric builder tests are not the target of UB detection.
#[cfg(all(test, not(miri), feature = "otel"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use opentelemetry_sdk::{
        Resource,
        metrics::{
            InMemoryMetricExporter,
            data::{AggregatedMetrics, MetricData},
        },
    };
    use opentelemetry_semantic_conventions::metric as semconv;

    use crate::otel::metrics::Meters;

    #[test]
    fn build_for_test_records_http_request_duration_via_meters() {
        let exporter = InMemoryMetricExporter::default();
        let resource = Resource::builder_empty().build();
        let provider = super::build_for_test(resource, exporter.clone());

        opentelemetry::global::set_meter_provider(provider.clone());
        let meters = Meters::new();
        meters.record_http_request(0.1, "GET", Some(200), "example.com", "https");

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == semconv::HTTP_CLIENT_REQUEST_DURATION)
            .expect("http.client.request.duration not found");

        let count = match metric.data() {
            AggregatedMetrics::F64(MetricData::Histogram(hist)) => {
                hist.data_points().next().expect("no data points").count()
            }
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert_eq!(count, 1);

        provider.shutdown().unwrap();
    }

    #[test]
    fn http_latency_view_applies_semconv_bucket_boundaries() {
        let exporter = InMemoryMetricExporter::default();
        let resource = Resource::builder_empty().build();
        let provider = super::build_for_test(resource, exporter.clone());

        opentelemetry::global::set_meter_provider(provider.clone());
        let meters = Meters::new();
        meters.record_http_request(0.1, "GET", Some(200), "example.com", "https");

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == semconv::HTTP_CLIENT_REQUEST_DURATION)
            .expect("http.client.request.duration not found");

        let bounds: Vec<f64> = match metric.data() {
            AggregatedMetrics::F64(MetricData::Histogram(hist)) => hist
                .data_points()
                .next()
                .expect("no data points")
                .bounds()
                .collect(),
            other => panic!("unexpected metric type: {other:?}"),
        };

        let expected: Vec<f64> = super::HTTP_LATENCY_BOUNDARIES.to_vec();
        assert_eq!(
            bounds, expected,
            "bucket boundaries must match semconv recommendation"
        );

        provider.shutdown().unwrap();
    }
}
