//! Integration test: `init_telemetry()` wires OTLP providers when endpoint is set.

#[cfg(all(not(miri), feature = "otel"))]
mod otlp_init {
    use tepra_core::otel::TelemetryGuard;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn init_telemetry_otlp_returns_otlp_guard() {
        // Safety: nextest isolates each test in its own process; no concurrent env mutation.
        unsafe { std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:1") };
        let guard = tepra_core::otel::init_telemetry("tepra-web", "")
            .expect("init_telemetry must succeed even with unreachable endpoint");
        assert!(
            matches!(guard, TelemetryGuard::Otlp { .. }),
            "expected TelemetryGuard::Otlp when endpoint is set"
        );
        guard.shutdown().await;
    }
}
