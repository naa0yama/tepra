//! `PrinterActor` cancel / in-flight state tests — RED state (T12c).
//!
//! These tests **fail** until T12d implements cancel tracking in the worker loop.
//! They compile correctly; failure is due to stub worker returning `ActorShutdown`
//! errors or `None` where real state is expected.
#![allow(missing_docs)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::significant_drop_tightening,
    clippy::missing_const_for_fn
)]

use std::sync::Arc;

use tepra_api::actor::PrinterActor;
use tepra_core::{
    client::{MockTepraClient, TepraClient, mock::MockCall},
    dto::job::{
        DensityParam, ErrorMessageParam, PrintFiles, PrintParameter, PrintRequest, PrintResponse,
    },
};

// ---------------------------------------------------------------------------
// Fixture helpers (duplicated from actor_queue.rs to keep tests independent)
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
// Test 1: cancel in-flight job A; job B continues and completes
// ---------------------------------------------------------------------------

/// A gated mock wraps `MockTepraClient` so the first `print()` call blocks
/// until the test releases the gate, simulating an in-flight job.
///
/// The gate is implemented with a `tokio::sync::Barrier(2)`: the worker
/// waits at the barrier; the test releases it after calling `cancel()`.
struct GatedMockClient {
    inner: Arc<MockTepraClient>,
    gate: Arc<tokio::sync::Barrier>,
    /// How many calls should be gated (only the first N calls wait).
    gate_count: std::sync::atomic::AtomicUsize,
}

impl GatedMockClient {
    fn new(inner: Arc<MockTepraClient>, gate: Arc<tokio::sync::Barrier>) -> Self {
        Self {
            inner,
            gate,
            gate_count: std::sync::atomic::AtomicUsize::new(1),
        }
    }
}

#[async_trait::async_trait]
impl TepraClient for GatedMockClient {
    async fn print(
        &self,
        name: &str,
        req: PrintRequest,
    ) -> Result<PrintResponse, tepra_core::error::TepraError> {
        // Block the first call at the barrier so the test can race cancel().
        let remaining = self
            .gate_count
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        if remaining > 0 {
            self.gate.wait().await;
        }
        self.inner.print(name, req).await
    }

    async fn list_printers(
        &self,
    ) -> Result<Vec<tepra_core::dto::printer::PrinterListItem>, tepra_core::error::TepraError> {
        self.inner.list_printers().await
    }

    async fn version(
        &self,
    ) -> Result<tepra_core::dto::printer::VersionResponse, tepra_core::error::TepraError> {
        self.inner.version().await
    }

    async fn autoselect(
        &self,
    ) -> Result<tepra_core::dto::printer::AutoselectResponse, tepra_core::error::TepraError> {
        self.inner.autoselect().await
    }

    async fn printer_info(
        &self,
        name: &str,
    ) -> Result<tepra_core::dto::printer::PrinterInfoResponse, tepra_core::error::TepraError> {
        self.inner.printer_info(name).await
    }

    async fn online_status(
        &self,
        name: &str,
    ) -> Result<tepra_core::dto::printer::OnlineStatusResponse, tepra_core::error::TepraError> {
        self.inner.online_status(name).await
    }

    async fn lw_status(
        &self,
        name: &str,
    ) -> Result<tepra_core::dto::printer::LwStatusResponse, tepra_core::error::TepraError> {
        self.inner.lw_status(name).await
    }

    async fn tapefeed(
        &self,
        name: &str,
        cutflag: bool,
    ) -> Result<(), tepra_core::error::TepraError> {
        self.inner.tapefeed(name, cutflag).await
    }

    async fn job_progress(
        &self,
        name: &str,
        jobid: u64,
    ) -> Result<tepra_core::dto::job::JobProgressResponse, tepra_core::error::TepraError> {
        self.inner.job_progress(name, jobid).await
    }

    async fn job_info(
        &self,
        name: &str,
        jobid: u64,
    ) -> Result<tepra_core::dto::job::JobInfoResponse, tepra_core::error::TepraError> {
        self.inner.job_info(name, jobid).await
    }

    async fn job_control(
        &self,
        name: &str,
        req: tepra_core::dto::job::JobControlRequest,
    ) -> Result<(), tepra_core::error::TepraError> {
        self.inner.job_control(name, req).await
    }

    async fn import_frame(
        &self,
        req: tepra_core::dto::template::ImportFrameRequest,
    ) -> Result<Vec<tepra_core::dto::template::ImportFrameItem>, tepra_core::error::TepraError>
    {
        self.inner.import_frame(req).await
    }

    async fn get_margin(
        &self,
        name: &str,
        req: tepra_core::dto::template::GetMarginRequest,
    ) -> Result<tepra_core::dto::template::GetMarginResponse, tepra_core::error::TepraError> {
        self.inner.get_margin(name, req).await
    }
}

#[tokio::test]
async fn test_cancel_current_job_continues_next() {
    // Two responses: job A (gated) and job B (immediate).
    let inner = client_with_responses(&[1, 2]);
    let gate = Arc::new(tokio::sync::Barrier::new(2));
    let gated = Arc::new(GatedMockClient::new(Arc::clone(&inner), Arc::clone(&gate)));

    let client: Arc<dyn TepraClient> = gated;
    let handle = PrinterActor::spawn(client, "PT-P710BT".into());

    // Submit job A (will block at gate) and job B.
    let jobid_a = handle.submit(minimal_req()).await.unwrap(); // RED: stub returns Err → panics
    let jobid_b = handle.submit(minimal_req()).await.unwrap();

    // Cancel A while it is in-flight (test side of the barrier).
    handle.cancel(jobid_a).await.unwrap();
    gate.wait().await; // release the worker

    // Give the actor time to process B.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // A should be Canceled; B should be Completed.
    let state_a = handle.status(jobid_a).await.unwrap();
    let state_b = handle.status(jobid_b).await.unwrap();

    assert_eq!(
        state_a.status,
        tepra_api::actor::job::JobStatus::Canceled,
        "job A must be Canceled"
    );
    assert_eq!(
        state_b.status,
        tepra_api::actor::job::JobStatus::Completed,
        "job B must be Completed"
    );

    // Only job B should have reached the printer API.
    assert_eq!(print_call_count(&inner), 1);

    handle.shutdown().await;
}

// ---------------------------------------------------------------------------
// Test 2: current_job() exposes the in-flight job ID
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_current_job_exposes_in_flight_jobid() {
    let inner = client_with_responses(&[10]);
    let gate = Arc::new(tokio::sync::Barrier::new(2));
    let gated = Arc::new(GatedMockClient::new(Arc::clone(&inner), Arc::clone(&gate)));

    let client: Arc<dyn TepraClient> = gated;
    let handle = PrinterActor::spawn(client, "PT-P710BT".into());

    let jobid = handle.submit(minimal_req()).await.unwrap(); // RED: stub returns Err → panics

    // Give the worker a tick to start processing.
    tokio::task::yield_now().await;

    // While blocked at the gate, current_job() must return our jobid.
    let current = handle.current_job().await; // RED: stub returns None
    assert_eq!(
        current,
        Some(jobid),
        "current_job() must return the in-flight job id"
    );

    // Release the gate so the worker can finish.
    gate.wait().await;
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    // After completion current_job() must return None.
    let after = handle.current_job().await;
    assert_eq!(
        after, None,
        "current_job() must be None after job completes"
    );

    handle.shutdown().await;
}

// ---------------------------------------------------------------------------
// Test 3: cancel with an unknown jobid is a no-op or returns an error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cancel_unknown_jobid_is_noop_or_error() {
    let mock = client_with_responses(&[]);
    let handle = PrinterActor::spawn(erase_mock(mock), "PT-P710BT".into());

    // Cancelling a jobid that was never submitted must not crash the actor.
    // The stub currently returns Err(ActorShutdown) which is acceptable as
    // a placeholder; T12d may return a dedicated CancelUnknown error variant.
    let result = handle.cancel(9999).await; // RED: stub returns Err(ActorShutdown)

    // Either Ok(()) (noop) or a non-ActorShutdown error is acceptable.
    // The actor must still be alive after the call.
    assert!(
        result.is_ok(),
        "cancel of unknown jobid must return Ok(()) (noop), got: {result:?}"
    );

    // Verify actor is still responsive.
    let current = handle.current_job().await;
    assert_eq!(current, None);

    handle.shutdown().await;
}
