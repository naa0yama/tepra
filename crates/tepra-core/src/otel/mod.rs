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

#[cfg(feature = "otel")]
use {
    opentelemetry_sdk::{
        logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
    },
    std::sync::atomic::{AtomicBool, Ordering},
};

/// Runtime telemetry guard. Keeps providers alive for the process lifetime.
///
/// Drop issues a warning when explicit `shutdown()` was not called first.
#[derive(Debug)]
pub enum TelemetryGuard {
    /// Active OTLP exporters. Hold providers alive until `shutdown()` is called.
    #[cfg(feature = "otel")]
    Otlp {
        /// Tracer provider with OTLP span exporter.
        tracer_provider: SdkTracerProvider,
        /// Meter provider with OTLP metric exporter.
        meter_provider: SdkMeterProvider,
        /// Logger provider with OTLP log exporter.
        logger_provider: SdkLoggerProvider,
        /// Set to true by `shutdown()` to prevent duplicate calls and warn on drop.
        shutdown_called: AtomicBool,
    },
    /// OTLP exporters are disabled; only the stderr fmt subscriber is active.
    Disabled,
}

impl TelemetryGuard {
    /// Shut down all telemetry providers in order: tracer → meter (flush+shutdown) → logger.
    ///
    /// The blocking shutdown calls run on a dedicated `std::thread` to avoid blocking
    /// the tokio executor. A 5-second timeout is applied; any provider that does not
    /// finish within the window is abandoned.
    ///
    /// Safe to call multiple times; subsequent calls are no-ops.
    pub async fn shutdown(&self) {
        #[cfg(feature = "otel")]
        {
            let Self::Otlp {
                tracer_provider,
                meter_provider,
                logger_provider,
                shutdown_called,
            } = self
            else {
                return;
            };
            if shutdown_called.swap(true, Ordering::SeqCst) {
                return;
            }

            let tracer = tracer_provider.clone();
            let meter = meter_provider.clone();
            let logger = logger_provider.clone();
            let (tx, rx) = tokio::sync::oneshot::channel::<()>();
            std::thread::spawn(move || {
                #[allow(clippy::print_stderr)]
                {
                    if let Err(e) = tracer.shutdown() {
                        eprintln!("tracer shutdown: {e}");
                    }
                    if let Err(e) = meter.force_flush() {
                        eprintln!("meter flush: {e}");
                    }
                    if let Err(e) = meter.shutdown() {
                        eprintln!("meter shutdown: {e}");
                    }
                    if let Err(e) = logger.shutdown() {
                        eprintln!("logger shutdown: {e}");
                    }
                }
                tx.send(()).ok();
            });
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), rx).await;
        }
    }
}

#[cfg(feature = "otel")]
impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Self::Otlp {
            shutdown_called, ..
        } = self
            && !shutdown_called.load(Ordering::SeqCst)
        {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("warning: TelemetryGuard dropped without explicit shutdown() call");
            }
        }
    }
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

/// Cycle 7: [`TelemetryGuard`] shutdown ordering tests.
///
/// Uses custom [`SpanProcessor`], [`MetricReader`], and [`LogProcessor`] implementations
/// that record which operations were invoked and in what order.
#[cfg(all(test, not(miri), feature = "otel"))]
mod shutdown_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use std::{
        sync::{Arc, Mutex, Weak},
        time::Duration,
    };

    use opentelemetry::InstrumentationScope;
    use opentelemetry_sdk::{
        Resource,
        error::OTelSdkResult,
        logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider},
        metrics::{
            InstrumentKind, Pipeline, SdkMeterProvider, Temporality, data::ResourceMetrics,
            reader::MetricReader,
        },
        trace::{SdkTracerProvider, Span, SpanData, SpanProcessor},
    };

    use super::*;

    type CallLog = Arc<Mutex<Vec<&'static str>>>;

    // ---- recording SpanProcessor ----

    #[derive(Debug)]
    struct RecordingSpanProcessor(CallLog);

    impl SpanProcessor for RecordingSpanProcessor {
        fn on_start(&self, _span: &mut Span, _cx: &opentelemetry::Context) {}
        fn on_end(&self, _span: SpanData) {}
        fn force_flush(&self) -> OTelSdkResult {
            Ok(())
        }
        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            self.0.lock().unwrap().push("tracer.shutdown");
            Ok(())
        }
    }

    // ---- recording MetricReader ----

    #[derive(Debug)]
    struct RecordingMetricReader(CallLog);

    impl MetricReader for RecordingMetricReader {
        fn register_pipeline(&self, _pipeline: Weak<Pipeline>) {}
        fn collect(&self, _rm: &mut ResourceMetrics) -> OTelSdkResult {
            Ok(())
        }
        fn force_flush(&self) -> OTelSdkResult {
            self.0.lock().unwrap().push("meter.flush");
            Ok(())
        }
        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            self.0.lock().unwrap().push("meter.shutdown");
            Ok(())
        }
        fn temporality(&self, _kind: InstrumentKind) -> Temporality {
            Temporality::Cumulative
        }
    }

    // ---- recording LogProcessor ----

    #[derive(Debug)]
    struct RecordingLogProcessor(CallLog);

    impl LogProcessor for RecordingLogProcessor {
        fn emit(&self, _data: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {}
        fn force_flush(&self) -> OTelSdkResult {
            Ok(())
        }
        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            self.0.lock().unwrap().push("logger.shutdown");
            Ok(())
        }
    }

    fn make_guard_with_log(log: &CallLog) -> TelemetryGuard {
        let resource = Resource::builder_empty().build();

        let tracer_provider = SdkTracerProvider::builder()
            .with_span_processor(RecordingSpanProcessor(Arc::clone(log)))
            .with_resource(resource.clone())
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_reader(RecordingMetricReader(Arc::clone(log)))
            .with_resource(resource.clone())
            .build();

        let logger_provider = SdkLoggerProvider::builder()
            .with_log_processor(RecordingLogProcessor(Arc::clone(log)))
            .with_resource(resource)
            .build();

        TelemetryGuard::Otlp {
            tracer_provider,
            meter_provider,
            logger_provider,
            shutdown_called: AtomicBool::new(false),
        }
    }

    #[tokio::test]
    async fn shutdown_order_is_tracer_then_meter_then_logger() {
        let log: CallLog = Arc::new(Mutex::new(Vec::new()));
        let guard = make_guard_with_log(&log);

        guard.shutdown().await;

        assert_eq!(
            *log.lock().unwrap(),
            [
                "tracer.shutdown",
                "meter.flush",
                "meter.shutdown",
                "logger.shutdown"
            ],
            "shutdown order must be tracer → meter(flush+shutdown) → logger"
        );
    }

    #[tokio::test]
    async fn shutdown_sets_called_flag() {
        let log: CallLog = Arc::new(Mutex::new(Vec::new()));
        let guard = make_guard_with_log(&log);

        let TelemetryGuard::Otlp {
            ref shutdown_called,
            ..
        } = guard
        else {
            panic!("expected Otlp variant");
        };

        assert!(
            !shutdown_called.load(Ordering::SeqCst),
            "flag should be false before shutdown"
        );
        guard.shutdown().await;
        assert!(
            shutdown_called.load(Ordering::SeqCst),
            "flag should be true after shutdown"
        );
    }

    #[tokio::test]
    async fn shutdown_is_idempotent() {
        let log: CallLog = Arc::new(Mutex::new(Vec::new()));
        let guard = make_guard_with_log(&log);

        guard.shutdown().await;
        guard.shutdown().await; // second call must be a no-op

        assert_eq!(
            log.lock().unwrap().len(),
            4,
            "providers must be shut down exactly once"
        );
    }

    #[tokio::test]
    async fn disabled_guard_shutdown_is_noop() {
        // TelemetryGuard::Disabled::shutdown() must return without panic.
        TelemetryGuard::Disabled.shutdown().await;
    }
}
