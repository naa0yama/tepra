#![allow(deprecated)]
//! Integration test: process metrics arrive via the global `MeterProvider` path.
//!
//! Verifies that `Meters::new()` (which calls `global::meter()`) registers
//! `ProcessMetricHandles` so that process.* metrics appear in exports.
//! This mirrors the production path in tepra-web where `Meters` is created
//! after `init_telemetry` sets the global provider.
//!
//! Run: `cargo test -p tepra-core --features process-metrics --test otel_process_metrics`
#![cfg(all(not(miri), feature = "otel", feature = "process-metrics"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};
use opentelemetry_semantic_conventions::metric as semconv;
use tepra_core::otel::metrics::Meters;

fn build_in_memory_provider() -> (SdkMeterProvider, InMemoryMetricExporter) {
    let exporter = InMemoryMetricExporter::default();
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build();
    let provider = SdkMeterProvider::builder().with_reader(reader).build();
    (provider, exporter)
}

/// Assert that `Meters::new()` — which uses the global `MeterProvider` — registers
/// process metric observable callbacks so the metrics are present on export.
///
/// This is the same code path used by the tepra-web binary:
///   1. `init_telemetry()` calls `global::set_meter_provider()`
///   2. `ReqwestTepraClient::new()` → `Meters::new()` → `global::meter()`
#[test]
#[cfg_attr(miri, ignore)]
fn process_metrics_arrive_via_global_provider() {
    let (provider, exporter) = build_in_memory_provider();
    opentelemetry::global::set_meter_provider(provider.clone());

    // Mirrors the production path: Meters::new() uses global::meter()
    let _meters = Meters::new();

    provider.force_flush().expect("flush failed");

    let metrics = exporter
        .get_finished_metrics()
        .expect("no metrics exported");

    let names: Vec<String> = metrics
        .iter()
        .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
        .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
        .map(|m| m.name().to_owned())
        .collect();

    for expected in [
        semconv::PROCESS_CPU_UTILIZATION,
        semconv::PROCESS_MEMORY_USAGE,
        semconv::PROCESS_UPTIME,
    ] {
        assert!(
            names.contains(&expected.to_owned()),
            "metric '{expected}' not found in exported metrics.\n\
             This usually means process-metrics feature is not enabled or \
             ProcessMetricHandles was dropped before flush.\n\
             Found: {names:?}",
        );
    }

    provider.shutdown().unwrap();
}
