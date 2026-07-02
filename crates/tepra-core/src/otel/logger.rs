//! OTLP log exporter and [`SdkLoggerProvider`] builder.

use anyhow::Context as _;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};

/// Build a production [`SdkLoggerProvider`] with an OTLP HTTP/protobuf log exporter.
///
/// The OTLP endpoint is resolved from `OTEL_EXPORTER_OTLP_ENDPOINT` at call time.
/// No endpoint argument is accepted; the env-var route lets the SDK append `/v1/logs`
/// automatically, avoiding the base-vs-signal-URL confusion with `.with_endpoint()`.
///
/// # Errors
///
/// Returns an error if the OTLP HTTP exporter cannot be constructed.
pub fn build(resource: Resource) -> anyhow::Result<SdkLoggerProvider> {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .build()
        .context("failed to build OTLP HTTP log exporter")?;
    Ok(SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build())
}

/// Build a test [`SdkLoggerProvider`] with a caller-supplied exporter.
///
/// Uses [`SimpleLogProcessor`] so log records are exported synchronously when emitted,
/// which makes assertions in unit tests straightforward without async flushing.
#[cfg(test)]
pub(crate) fn build_for_test(
    resource: Resource,
    exporter: opentelemetry_sdk::logs::InMemoryLogExporter,
) -> SdkLoggerProvider {
    SdkLoggerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build()
}

#[cfg(all(test, not(miri), feature = "otel"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use opentelemetry::logs::Severity;
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use opentelemetry_sdk::{Resource, logs::InMemoryLogExporter};
    use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

    #[test]
    fn build_for_test_records_error_log_via_tracing_bridge() {
        let exporter = InMemoryLogExporter::default();
        let resource = Resource::builder_empty().build();
        let provider = super::build_for_test(resource, exporter.clone());

        let bridge = OpenTelemetryTracingBridge::new(&provider);
        // Ignore "already set" — nextest isolates per-process; safe to ignore.
        let _ = tracing_subscriber::registry().with(bridge).try_init();

        tracing::error!(code = 42, "boom");

        provider.force_flush().expect("force_flush failed");

        let logs = exporter.get_emitted_logs().expect("no emitted logs");
        assert!(!logs.is_empty(), "expected at least one log record");

        let record = &logs.first().expect("one log record").record;

        assert_eq!(
            record.severity_number(),
            Some(Severity::Error),
            "expected ERROR severity"
        );

        let body = record.body().expect("expected log body");
        let body_str = format!("{body:?}");
        assert!(
            body_str.contains("boom"),
            "expected body to contain 'boom', got: {body_str}"
        );

        let has_code_attr = record
            .attributes_iter()
            .any(|(k, v)| k.as_str() == "code" && format!("{v:?}").contains("42"));
        assert!(has_code_attr, "expected attribute code=42");
    }
}
