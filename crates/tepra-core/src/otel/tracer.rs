//! OTLP span exporter and [`SdkTracerProvider`] builder.

use anyhow::Context as _;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};

/// Build a production [`SdkTracerProvider`] with an OTLP HTTP/protobuf span exporter.
///
/// The OTLP endpoint is resolved from `OTEL_EXPORTER_OTLP_ENDPOINT` at call time.
/// No endpoint argument is accepted; the env-var route lets the SDK append `/v1/traces`
/// automatically, avoiding the base-vs-signal-URL confusion with `.with_endpoint()`.
///
/// # Errors
///
/// Returns an error if the OTLP HTTP exporter cannot be constructed.
pub fn build(resource: Resource) -> anyhow::Result<SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .context("failed to build OTLP HTTP span exporter")?;
    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build())
}

/// Build a test [`SdkTracerProvider`] with a caller-supplied exporter.
///
/// Uses [`SimpleSpanProcessor`] so spans are exported synchronously when ended,
/// which makes assertions in unit tests straightforward without async flushing.
#[cfg(test)]
pub(crate) fn build_for_test<E: opentelemetry_sdk::trace::SpanExporter + 'static>(
    resource: Resource,
    exporter: E,
) -> SdkTracerProvider {
    SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::{Tracer, TracerProvider as _};
    use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;

    #[test]
    fn span_name_is_recorded_via_in_memory_exporter() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let resource = Resource::builder_empty().build();
        let provider = build_for_test(resource, exporter.clone());
        let tracer = provider.tracer("test-tracer");

        // Dropping the span ends it; SimpleSpanProcessor exports synchronously on end.
        drop(tracer.start("test"));

        let spans = exporter
            .get_finished_spans()
            .expect("should get finished spans");
        assert_eq!(spans.len(), 1, "expected exactly one finished span");
        let span = spans.first().expect("one span");
        assert_eq!(&*span.name, "test");
    }
}
