//! `PrinterActor` queue integration tests — RED state (T12a).
//!
//! These tests **fail** until T12b implements the actor worker loop.
//! They compile and fixture setup is correct; failure is due to `todo!()`
//! stubs panicking before any `mpsc` messages are sent.
#![allow(missing_docs)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::significant_drop_tightening,
    clippy::missing_const_for_fn
)]

use std::sync::Arc;

use tepra::actor::PrinterActor;
use tepra_core::{
    client::{MockTepraClient, TepraClient, mock::MockCall},
    dto::job::{
        DensityParam, ErrorMessageParam, PrintFiles, PrintParameter, PrintRequest, PrintResponse,
    },
};

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

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

fn client_with_responses(jobids: &[u64]) -> Arc<MockTepraClient> {
    let mock = Arc::new(MockTepraClient::new());
    for &id in jobids {
        mock.push_print(Ok(ok_response(id)));
    }
    mock
}

/// Coerce `Arc<MockTepraClient>` → `Arc<dyn TepraClient>` without an `as` cast.
fn erase_mock(mock: Arc<MockTepraClient>) -> Arc<dyn TepraClient> {
    mock
}

fn print_call_count(mock: &MockTepraClient) -> usize {
    mock.calls()
        .iter()
        .filter(|c| matches!(c, MockCall::Print(_, _)))
        .count()
}

fn first_print_name(mock: &MockTepraClient) -> String {
    mock.calls()
        .iter()
        .find_map(|c| {
            if let MockCall::Print(n, _) = c {
                Some(n.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Test 1: single job completes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn actor_completes_single_print_job() {
    let mock = client_with_responses(&[42]);
    let handle = PrinterActor::spawn(erase_mock(Arc::clone(&mock)), "PT-P710BT".into());

    let resp = handle.print(minimal_req()).await.unwrap();
    assert_eq!(resp.jobid, 42);

    assert_eq!(print_call_count(&mock), 1);
    assert_eq!(first_print_name(&mock), "PT-P710BT");

    handle.shutdown().await;
}

// ---------------------------------------------------------------------------
// Test 2: three sequential jobs preserve FIFO order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn queue_preserves_fifo_order_for_sequential_jobs() {
    let mock = client_with_responses(&[10, 20, 30]);
    let handle = PrinterActor::spawn(erase_mock(Arc::clone(&mock)), "PT-P710BT".into());

    let r1 = handle.print(minimal_req()).await.unwrap();
    let r2 = handle.print(minimal_req()).await.unwrap();
    let r3 = handle.print(minimal_req()).await.unwrap();

    // Responses arrive in submission order (FIFO).
    assert_eq!(r1.jobid, 10);
    assert_eq!(r2.jobid, 20);
    assert_eq!(r3.jobid, 30);

    assert_eq!(print_call_count(&mock), 3);

    handle.shutdown().await;
}

// ---------------------------------------------------------------------------
// Test 3: concurrent submissions are serialized in submission order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn concurrent_submissions_are_serialized_in_fifo_order() {
    // Two jobs submitted concurrently via join!.
    // The actor must serialise them: job 1 runs first → gets jobid=1,
    // job 2 runs second → gets jobid=2.
    // If the worker ran them in parallel the mock queue ordering would break.
    let mock = client_with_responses(&[1, 2]);
    let handle = PrinterActor::spawn(erase_mock(Arc::clone(&mock)), "PT-P710BT".into());

    let (r1, r2) = tokio::join!(handle.print(minimal_req()), handle.print(minimal_req()));

    // First submitted job gets the first queued response.
    assert_eq!(r1.unwrap().jobid, 1);
    // Second submitted job gets the second queued response.
    assert_eq!(r2.unwrap().jobid, 2);

    assert_eq!(print_call_count(&mock), 2);

    handle.shutdown().await;
}
