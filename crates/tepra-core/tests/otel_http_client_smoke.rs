//! Real-env OTLP export smoke test.
//! Run with: `cargo test --test otel_http_client_smoke -- --ignored --nocapture`
#![cfg(not(miri))]
#![cfg(feature = "otel")]
#![allow(clippy::unwrap_used, clippy::undocumented_unsafe_blocks)]

use tepra_core::client::{ReqwestTepraClient, TepraClient};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[ignore = "requires OpenObserve on localhost:5080"]
async fn real_otlp_smoke_all_methods() {
    // Point at real o2 OTLP HTTP endpoint (v1/traces default sub-path)
    unsafe {
        std::env::set_var(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            "http://localhost:5080/api/default",
        );
    }

    let telemetry =
        tepra_core::otel::init_telemetry("tepra-smoke", "cycle27").expect("init_telemetry");

    let mock_server = MockServer::start().await;

    let version_body = include_str!("fixtures/dto/version_res.json");

    // 2xx GET path
    Mock::given(method("GET"))
        .and(path("/api/printer/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_raw(version_body, "application/json"),
        )
        .mount(&mock_server)
        .await;

    // 5xx GET path (list_printers → /api/printer)
    Mock::given(method("GET"))
        .and(path("/api/printer"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let client = ReqwestTepraClient::new(mock_server.uri());

    // 2xx GET — should succeed
    let _ = client.version().await;
    // 5xx GET — should return Err (transport/status error); ignore result
    let _ = client.list_printers().await;

    // shutdown → OTLP flush
    telemetry.shutdown().await;

    // give o2 ingest time
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}
