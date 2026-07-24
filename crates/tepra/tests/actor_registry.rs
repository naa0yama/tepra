//! `PrinterRegistry` lifecycle tests — RED state (T13a).
//!
//! These tests **fail** until T13b implements `PrinterRegistry`.
//! They compile correctly; failure is due to `todo!()` stubs panicking.
#![allow(missing_docs)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::significant_drop_tightening,
    clippy::missing_const_for_fn
)]

use std::sync::Arc;

use tepra::actor::PrinterRegistry;
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

fn erase_mock(mock: Arc<MockTepraClient>) -> Arc<dyn TepraClient> {
    mock
}

fn print_call_count(mock: &MockTepraClient) -> usize {
    mock.calls()
        .iter()
        .filter(|c| matches!(c, MockCall::Print(_, _)))
        .count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// First `get_or_spawn` call spawns the actor; a print job should reach the mock.
#[tokio::test]
async fn registry_spawns_actor_on_first_access() {
    let mock = client_with_responses(&[1]);
    let client = erase_mock(Arc::clone(&mock));
    let registry = PrinterRegistry::new(client);

    let handle = registry.get_or_spawn("LW-PX550");
    handle.print(minimal_req()).await.unwrap();

    assert_eq!(print_call_count(&mock), 1);
}

/// Two calls with the same name return a handle to the same actor —
/// a job submitted via the first handle is visible via the second.
#[tokio::test]
async fn registry_reuses_actor_for_same_name() {
    let mock = client_with_responses(&[1]);
    let client = erase_mock(Arc::clone(&mock));
    let registry = PrinterRegistry::new(client);

    let handle1 = registry.get_or_spawn("LW-PX550");
    let job_id = handle1.submit(minimal_req()).await.unwrap();

    let handle2 = registry.get_or_spawn("LW-PX550");
    let status = handle2.status(job_id).await;
    assert!(
        status.is_some(),
        "same actor must expose jobs submitted via handle1"
    );
}

/// Different names must produce independent actors with separate job spaces.
#[tokio::test]
async fn registry_spawns_separate_actors_for_different_names() {
    let mock = client_with_responses(&[1]);
    let client = erase_mock(Arc::clone(&mock));
    let registry = PrinterRegistry::new(client);

    let handle_a = registry.get_or_spawn("LW-PX550");
    let job_id = handle_a.submit(minimal_req()).await.unwrap();

    let handle_b = registry.get_or_spawn("LW-PX600");
    let status = handle_b.status(job_id).await;
    assert!(
        status.is_none(),
        "different actors must not share job state"
    );
}

/// `shutdown_all` must drain and join every registered actor within 1 second.
#[tokio::test]
async fn registry_shutdown_all_joins_actors() {
    let mock = client_with_responses(&[]);
    let client = erase_mock(Arc::clone(&mock));
    let registry = PrinterRegistry::new(client);

    // Spawn a couple of actors.
    let _h1 = registry.get_or_spawn("LW-PX550");
    let _h2 = registry.get_or_spawn("LW-PX600");

    tokio::time::timeout(std::time::Duration::from_secs(1), registry.shutdown_all())
        .await
        .expect("shutdown_all must complete within 1 second");
}
