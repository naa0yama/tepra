//! OpenTelemetry instrumentation root module.

/// Application-specific OTel semantic convention constants mirror.
pub mod conventions;
/// OTLP log exporter and logger provider builder.
pub mod logger;
/// OTLP metric exporter and meter provider builder.
pub mod meter;
pub mod metrics;
/// Resource builder (service identity attributes).
pub mod resource;
/// OTLP span exporter and tracer provider builder.
pub mod tracer;

use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

/// Runtime telemetry guard. Keeps providers alive for the process lifetime.
///
/// Drop issues a warning when explicit `shutdown()` was not called first.
#[derive(Debug)]
pub enum TelemetryGuard {
    /// OTLP exporters are disabled; only the stderr fmt subscriber is active.
    Disabled,
}

/// Initialize telemetry for the current process.
///
/// When `OTEL_EXPORTER_OTLP_ENDPOINT` is set and non-empty, installs OpenTelemetry
/// providers with OTLP exporters (Cycles 4-7). When absent, registers only a
/// stderr fmt subscriber and returns [`TelemetryGuard::Disabled`].
///
/// The W3C Trace Context propagator is registered unconditionally so that
/// incoming `traceparent` headers are extracted even without OTLP export.
///
/// # Errors
///
/// Returns an error if the `OTEL_EXPORTER_OTLP_ENDPOINT` variable is set but
/// OTLP provider initialisation fails (implemented from Cycle 4 onwards).
pub fn init_telemetry() -> anyhow::Result<TelemetryGuard> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|s| !s.is_empty());

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_writer(std::io::stderr);

    if endpoint.is_some() {
        // Cycle 4+: build providers and install OTLP exporters
        anyhow::bail!("OTLP telemetry path not yet implemented")
    }

    // eprintln! is intentional: tracing::warn! is unreliable before subscriber
    // init completes, so startup-time warnings go directly to stderr.
    #[allow(clippy::print_stderr)]
    {
        eprintln!("OTEL_EXPORTER_OTLP_ENDPOINT not set; telemetry exporters disabled");
    }

    // Ignore "already set" — nextest isolates per-process; safe to ignore.
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init();

    Ok(TelemetryGuard::Disabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_when_endpoint_unset() {
        // Safety: nextest runs each test in an isolated process; no concurrent env mutation.
        unsafe { std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT") };
        let guard = init_telemetry().expect("init_telemetry must not fail when endpoint is absent");
        assert!(matches!(guard, TelemetryGuard::Disabled));
    }

    #[test]
    fn tracing_works_after_disabled_init() {
        // Safety: nextest runs each test in an isolated process; no concurrent env mutation.
        unsafe { std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT") };
        let _guard =
            init_telemetry().expect("init_telemetry must not fail when endpoint is absent");
        // Must not panic regardless of whether a subscriber was already registered.
        tracing::info!("smoke test: telemetry disabled path");
    }
}
