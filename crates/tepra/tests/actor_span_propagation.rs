//! Integration test: `tokio::spawn` in `PrinterActor::spawn` propagates span context.
//!
//! RED state: fails until `run_worker` future is wrapped with `.in_current_span()`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening,
    clippy::missing_const_for_fn,
    clippy::as_conversions
)]

use std::sync::Arc;

use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
use tepra::actor::PrinterActor;
use tepra_core::{
    client::MockTepraClient,
    dto::job::{
        DensityParam, ErrorMessageParam, PrintFiles, PrintParameter, PrintRequest, PrintResponse,
    },
    otel::TelemetryGuard,
};
fn print_parameter() -> PrintParameter {
    PrintParameter {
        copies: 1,
        tape_cut: 2,
        half_cut: 1,
        print_speed: 1,
        density: DensityParam { mode: 1, value: 0 },
        tape_id: 261,
        priority_cut_setting: 1,
        half_cut_separate: 1,
        margin_left_right: 0,
        display_tape_width: 1,
        error_message: ErrorMessageParam {
            mode: 1,
            file_output: 0,
            file_path: String::new(),
        },
        display_transfer_tape: 1,
        display_print_setting: 1,
        cut_title: 0,
        kana_zen: 0,
        display_print_preview: 1,
        stretch_image: 0,
    }
}

fn minimal_req() -> PrintRequest {
    PrintRequest {
        print_file: PrintFiles {
            template_file: None,
            csv_file: None,
            image_file: None,
        },
        print_parameter: print_parameter(),
    }
}

fn ok_response(jobid: u64) -> PrintResponse {
    PrintResponse { result: 1, jobid }
}

/// Verifies that spans created inside the `run_worker` task are children of the
/// parent span that was active when `PrinterActor::spawn` was called.
#[tokio::test]
async fn actor_spawn_propagates_parent_span() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let _guard = TelemetryGuard::build_for_test(exporter.clone());

    let mock = Arc::new(MockTepraClient::new());
    mock.push_print(Ok(ok_response(1)));

    // Spawn the actor from within the parent span so `.in_current_span()` captures it.
    // Drop parent_span immediately after spawning so it is exported before we query.
    let handle = {
        let parent_span = tracing::info_span!("actor.test.parent");
        let _enter = parent_span.enter();
        let h = PrinterActor::spawn(Arc::clone(&mock) as Arc<_>, "test-printer".into());
        // _enter drops here → span exited; parent_span drops → span finished & exported.
        h
    };

    // Let the actor execute the job and then shut it down.
    let _resp = handle.print(minimal_req()).await;
    handle.shutdown().await;

    let spans = exporter
        .get_finished_spans()
        .expect("spans must be accessible");

    // Find the worker span (from run_worker).
    let worker_span = spans.iter().find(|s| s.name.as_ref() == "actor.worker.run");
    assert!(
        worker_span.is_some(),
        "expected a span named 'actor.worker.run', got: {:#?}",
        spans.iter().map(|s| s.name.as_ref()).collect::<Vec<_>>()
    );

    // Find the parent span.
    let parent = spans
        .iter()
        .find(|s| s.name.as_ref() == "actor.test.parent")
        .expect("parent span must be exported");

    let worker = worker_span.unwrap();
    assert_eq!(
        worker.parent_span_id,
        parent.span_context.span_id(),
        "run_worker span must be a child of the parent span"
    );
}
