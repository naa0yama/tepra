//! Application metric instruments.
//!
//! Create a [`Meters`] instance once via [`Meters::default`] after the global
//! `MeterProvider` has been initialized, then pass it by shared reference to
//! every function that records measurements.

// ---------------------------------------------------------------------------
// OTel implementation (feature = "otel")
// ---------------------------------------------------------------------------

#[cfg(feature = "process-metrics")]
mod process;

#[cfg(feature = "otel")]
use opentelemetry::metrics::Histogram;
#[cfg(feature = "otel")]
use opentelemetry_semantic_conventions::{attribute, metric as semconv};

/// Collected `OTel` metric instruments for this application.
///
/// All instruments are created once and reused — do not construct per-request.
/// The `_process` field keeps observable process metric callbacks registered
/// for the lifetime of this struct (requires the `process-metrics` feature).
#[cfg(feature = "otel")]
pub struct Meters {
    // --- Sync instruments ---
    http_request_duration: Histogram<f64>,
    // --- Observable process metrics (feature = "process-metrics") ---
    // Disabled under Miri: sysinfo calls sysconf(_SC_CLK_TCK) which Miri does not stub.
    #[cfg(all(feature = "process-metrics", not(miri)))]
    _process: process::ProcessMetricHandles,
}

#[cfg(feature = "otel")]
impl std::fmt::Debug for Meters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Meters").finish_non_exhaustive()
    }
}

#[cfg(feature = "otel")]
impl Default for Meters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "otel")]
impl Meters {
    /// Create all application instruments from the global `MeterProvider`.
    ///
    /// Call exactly once after `opentelemetry::global::set_meter_provider` has
    /// been called. When the `process-metrics` feature is enabled (default),
    /// process metric observable callbacks are also registered here.
    #[must_use]
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter(env!("CARGO_PKG_NAME"));

        Self {
            http_request_duration: meter
                .f64_histogram(semconv::HTTP_CLIENT_REQUEST_DURATION)
                .with_unit("s")
                .with_description(
                    "HTTP client request duration including response body download \
                     (`OTel` HTTP semconv)",
                )
                .build(),
            #[cfg(all(feature = "process-metrics", not(miri)))]
            _process: process::ProcessMetricHandles::register(&meter),
        }
    }

    /// Record an HTTP client request duration with `OTel` HTTP semantic convention attributes.
    ///
    /// `status` is `None` for transport errors (connection refused, timeout, etc.)
    /// where no HTTP response was received; the `http.response.status_code` attribute
    /// is omitted in that case per `OTel` HTTP semconv.
    pub fn record_http_request(
        &self,
        duration_s: f64,
        method: &str,
        status: Option<u16>,
        host: &str,
        scheme: &str,
    ) {
        use opentelemetry::KeyValue;
        let mut attrs = vec![
            KeyValue::new(attribute::HTTP_REQUEST_METHOD, method.to_owned()),
            KeyValue::new(attribute::SERVER_ADDRESS, host.to_owned()),
            KeyValue::new(attribute::URL_SCHEME, scheme.to_owned()),
        ];
        if let Some(s) = status {
            attrs.push(KeyValue::new(
                attribute::HTTP_RESPONSE_STATUS_CODE,
                i64::from(s),
            ));
        }
        self.http_request_duration.record(duration_s, &attrs);
    }
}

// ---------------------------------------------------------------------------
// No-op stub (feature != "otel")
// ---------------------------------------------------------------------------

/// No-op metric instruments used when the `otel` feature is disabled.
#[cfg(not(feature = "otel"))]
#[derive(Debug, Default)]
pub struct Meters;

#[cfg(not(feature = "otel"))]
impl Meters {
    /// Record an HTTP client request (no-op).
    pub fn record_http_request(
        &self,
        _duration_s: f64,
        _method: &str,
        _status: Option<u16>,
        _host: &str,
        _scheme: &str,
    ) {
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// opentelemetry_sdk::metrics::PeriodicReader calls readlink (process env detection)
// which miri isolation blocks; OTel metric tests are not the target of UB detection.
#[cfg(all(test, not(miri), feature = "otel"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry_sdk::metrics::{
        InMemoryMetricExporter, SdkMeterProvider,
        data::{AggregatedMetrics, MetricData},
    };

    fn test_provider() -> (SdkMeterProvider, InMemoryMetricExporter) {
        let exporter = InMemoryMetricExporter::default();
        let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        (provider, exporter)
    }

    /// Find a metric by name in the exported `ResourceMetrics` snapshot.
    fn find_metric<'a>(
        metrics: &'a [opentelemetry_sdk::metrics::data::ResourceMetrics],
        name: &str,
    ) -> Option<&'a opentelemetry_sdk::metrics::data::Metric> {
        metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == name)
    }

    #[test]
    fn http_request_duration_records_semconv_attributes() {
        use opentelemetry::KeyValue;
        use opentelemetry_semantic_conventions::{attribute, metric as semconv};

        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");

        let histogram = meter
            .f64_histogram(semconv::HTTP_CLIENT_REQUEST_DURATION)
            .with_unit("s")
            .with_description("HTTP client request duration including response body download")
            .build();

        let attrs = [
            KeyValue::new(attribute::HTTP_REQUEST_METHOD, "GET"),
            KeyValue::new(attribute::HTTP_RESPONSE_STATUS_CODE, 200_i64),
            KeyValue::new(attribute::SERVER_ADDRESS, "example.com"),
            KeyValue::new(attribute::URL_SCHEME, "https"),
        ];
        histogram.record(0.123, &attrs);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = find_metric(&metrics, semconv::HTTP_CLIENT_REQUEST_DURATION)
            .expect("http.client.request.duration not found");

        let (count, sum) = match metric.data() {
            AggregatedMetrics::F64(MetricData::Histogram(hist)) => {
                let dp = hist.data_points().next().expect("no data points");
                (dp.count(), dp.sum())
            }
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert_eq!(count, 1);
        assert!((sum - 0.123).abs() < 1e-9);

        provider.shutdown().unwrap();
    }

    #[test]
    fn meters_record_http_request_via_struct() {
        use opentelemetry_semantic_conventions::metric as semconv;

        let (provider, exporter) = test_provider();
        opentelemetry::global::set_meter_provider(provider.clone());

        let meters = super::Meters::new();
        meters.record_http_request(0.042, "POST", Some(201), "example.com", "https");

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let metric = find_metric(&metrics, semconv::HTTP_CLIENT_REQUEST_DURATION)
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
    fn meters_debug_does_not_panic() {
        let (provider, _) = test_provider();
        opentelemetry::global::set_meter_provider(provider.clone());
        let meters = super::Meters::default();
        let _ = format!("{meters:?}");
        provider.shutdown().unwrap();
    }
}
