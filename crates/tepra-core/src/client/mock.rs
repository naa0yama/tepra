//! In-process mock implementation of [`TepraClient`] for unit testing.
// expect() and panic!() are intentional contract-enforcement in a test double.
#![allow(clippy::expect_used, clippy::panic)]
//!
//! # Usage
//!
//! ```rust
//! # use tepra_core::client::mock::{MockCall, MockTepraClient};
//! # use tepra_core::dto::printer::PrinterListItem;
//! # use tepra_core::client::traits::TepraClient;
//! # #[tokio::main]
//! # async fn main() {
//! let mock = MockTepraClient::new();
//! mock.push_list_printers(Ok(vec![PrinterListItem { printer_name: "PT-P710BT".into() }]));
//!
//! let result = mock.list_printers().await.unwrap();
//! assert_eq!(result[0].printer_name, "PT-P710BT");
//!
//! let calls = mock.calls();
//! assert!(matches!(calls[0], MockCall::ListPrinters));
//! # }
//! ```

use std::{
    collections::VecDeque,
    sync::{Mutex, MutexGuard},
};

use async_trait::async_trait;

use crate::{
    dto::{
        job::{
            JobControlRequest, JobInfoResponse, JobProgressResponse, PrintRequest, PrintResponse,
        },
        printer::{
            AutoselectResponse, LwStatusResponse, OnlineStatusResponse, PrinterInfoResponse,
            PrinterListItem, VersionResponse,
        },
        template::{GetMarginRequest, GetMarginResponse, ImportFrameItem, ImportFrameRequest},
    },
    error::TepraError,
};

use super::traits::TepraClient;

// ---------------------------------------------------------------------------
// Call record
// ---------------------------------------------------------------------------

/// Records a single call made to [`MockTepraClient`].
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs, clippy::module_name_repetitions)]
pub enum MockCall {
    ListPrinters,
    Version,
    Autoselect,
    PrinterInfo(String),
    OnlineStatus(String),
    LwStatus(String),
    Print(String, PrintRequest),
    Tapefeed(String, bool),
    JobProgress(String, u64),
    JobInfo(String, u64),
    JobControl(String, JobControlRequest),
    ImportFrame(ImportFrameRequest),
    GetMargin(String, GetMarginRequest),
}

// ---------------------------------------------------------------------------
// Response queues
// ---------------------------------------------------------------------------

type Res<T> = Result<T, TepraError>;

#[derive(Debug, Default)]
struct Queues {
    list_printers: VecDeque<Res<Vec<PrinterListItem>>>,
    version: VecDeque<Res<VersionResponse>>,
    autoselect: VecDeque<Res<AutoselectResponse>>,
    printer_info: VecDeque<Res<PrinterInfoResponse>>,
    online_status: VecDeque<Res<OnlineStatusResponse>>,
    lw_status: VecDeque<Res<LwStatusResponse>>,
    print: VecDeque<Res<PrintResponse>>,
    tapefeed: VecDeque<Res<()>>,
    job_progress: VecDeque<Res<JobProgressResponse>>,
    job_info: VecDeque<Res<JobInfoResponse>>,
    job_control: VecDeque<Res<()>>,
    import_frame: VecDeque<Res<Vec<ImportFrameItem>>>,
    get_margin: VecDeque<Res<GetMarginResponse>>,
}

// ---------------------------------------------------------------------------
// MockTepraClient
// ---------------------------------------------------------------------------

/// Test double for [`TepraClient`].
///
/// Responses are consumed FIFO. Panics when a method is called but the
/// corresponding queue is empty.
#[derive(Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct MockTepraClient {
    calls: Mutex<Vec<MockCall>>,
    queues: Mutex<Queues>,
}

impl MockTepraClient {
    /// Creates a new empty mock with no pre-configured responses.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot of every call recorded so far.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned.
    pub fn calls(&self) -> MutexGuard<'_, Vec<MockCall>> {
        self.calls.lock().expect("calls mutex poisoned")
    }

    fn q(&self) -> MutexGuard<'_, Queues> {
        self.queues.lock().expect("queues mutex poisoned")
    }

    fn record(&self, call: MockCall) {
        self.calls.lock().expect("calls mutex poisoned").push(call);
    }
}

// ---------------------------------------------------------------------------
// Push helpers (one per method)
// ---------------------------------------------------------------------------

macro_rules! push_fn {
    ($fn_name:ident, $field:ident, $t:ty) => {
        /// Enqueue the next response returned by the corresponding method.
        pub fn $fn_name(&self, response: Res<$t>) {
            self.q().$field.push_back(response);
        }
    };
}

impl MockTepraClient {
    push_fn!(push_list_printers, list_printers, Vec<PrinterListItem>);
    push_fn!(push_version, version, VersionResponse);
    push_fn!(push_autoselect, autoselect, AutoselectResponse);
    push_fn!(push_printer_info, printer_info, PrinterInfoResponse);
    push_fn!(push_online_status, online_status, OnlineStatusResponse);
    push_fn!(push_lw_status, lw_status, LwStatusResponse);
    push_fn!(push_print, print, PrintResponse);
    push_fn!(push_tapefeed, tapefeed, ());
    push_fn!(push_job_progress, job_progress, JobProgressResponse);
    push_fn!(push_job_info, job_info, JobInfoResponse);
    push_fn!(push_job_control, job_control, ());
    push_fn!(push_import_frame, import_frame, Vec<ImportFrameItem>);
    push_fn!(push_get_margin, get_margin, GetMarginResponse);
}

// ---------------------------------------------------------------------------
// TepraClient impl
// ---------------------------------------------------------------------------

#[async_trait]
impl TepraClient for MockTepraClient {
    async fn list_printers(&self) -> Res<Vec<PrinterListItem>> {
        self.record(MockCall::ListPrinters);
        self.q()
            .list_printers
            .pop_front()
            .expect("MockTepraClient::list_printers called but response queue is empty")
    }

    async fn version(&self) -> Res<VersionResponse> {
        self.record(MockCall::Version);
        self.q()
            .version
            .pop_front()
            .expect("MockTepraClient::version called but response queue is empty")
    }

    async fn autoselect(&self) -> Res<AutoselectResponse> {
        self.record(MockCall::Autoselect);
        self.q()
            .autoselect
            .pop_front()
            .expect("MockTepraClient::autoselect called but response queue is empty")
    }

    async fn printer_info(&self, name: &str) -> Res<PrinterInfoResponse> {
        self.record(MockCall::PrinterInfo(name.to_owned()));
        self.q()
            .printer_info
            .pop_front()
            .expect("MockTepraClient::printer_info called but response queue is empty")
    }

    async fn online_status(&self, name: &str) -> Res<OnlineStatusResponse> {
        self.record(MockCall::OnlineStatus(name.to_owned()));
        self.q()
            .online_status
            .pop_front()
            .expect("MockTepraClient::online_status called but response queue is empty")
    }

    async fn lw_status(&self, name: &str) -> Res<LwStatusResponse> {
        self.record(MockCall::LwStatus(name.to_owned()));
        self.q()
            .lw_status
            .pop_front()
            .expect("MockTepraClient::lw_status called but response queue is empty")
    }

    async fn print(&self, name: &str, req: PrintRequest) -> Res<PrintResponse> {
        self.record(MockCall::Print(name.to_owned(), req));
        self.q()
            .print
            .pop_front()
            .expect("MockTepraClient::print called but response queue is empty")
    }

    async fn tapefeed(&self, name: &str, cutflag: bool) -> Res<()> {
        self.record(MockCall::Tapefeed(name.to_owned(), cutflag));
        self.q()
            .tapefeed
            .pop_front()
            .expect("MockTepraClient::tapefeed called but response queue is empty")
    }

    async fn job_progress(&self, name: &str, jobid: u64) -> Res<JobProgressResponse> {
        self.record(MockCall::JobProgress(name.to_owned(), jobid));
        self.q()
            .job_progress
            .pop_front()
            .expect("MockTepraClient::job_progress called but response queue is empty")
    }

    async fn job_info(&self, name: &str, jobid: u64) -> Res<JobInfoResponse> {
        self.record(MockCall::JobInfo(name.to_owned(), jobid));
        self.q()
            .job_info
            .pop_front()
            .expect("MockTepraClient::job_info called but response queue is empty")
    }

    async fn job_control(&self, name: &str, req: JobControlRequest) -> Res<()> {
        self.record(MockCall::JobControl(name.to_owned(), req));
        self.q()
            .job_control
            .pop_front()
            .expect("MockTepraClient::job_control called but response queue is empty")
    }

    async fn import_frame(&self, req: ImportFrameRequest) -> Res<Vec<ImportFrameItem>> {
        self.record(MockCall::ImportFrame(req));
        self.q()
            .import_frame
            .pop_front()
            .expect("MockTepraClient::import_frame called but response queue is empty")
    }

    async fn get_margin(&self, name: &str, req: GetMarginRequest) -> Res<GetMarginResponse> {
        self.record(MockCall::GetMargin(name.to_owned(), req));
        self.q()
            .get_margin
            .pop_front()
            .expect("MockTepraClient::get_margin called but response queue is empty")
    }
}

// ---------------------------------------------------------------------------
// Contract tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::significant_drop_tightening
    )]
    use super::*;
    use crate::dto::{
        enums::ImportFrameAttribute,
        job::FilePayload,
        printer::{DriverVersion, TapeEntry},
        template::ImportFrameItem,
    };

    fn printer(name: &str) -> PrinterListItem {
        PrinterListItem {
            printer_name: name.to_owned(),
        }
    }

    // --- list_printers -------------------------------------------------------

    #[tokio::test]
    async fn list_printers_records_call_and_returns_response() {
        let mock = MockTepraClient::new();
        mock.push_list_printers(Ok(vec![printer("PT-P710BT")]));

        let result = mock.list_printers().await.unwrap();
        assert_eq!(result[0].printer_name, "PT-P710BT");

        let calls = mock.calls();
        assert_eq!(calls.len(), 1);
        assert!(matches!(calls[0], MockCall::ListPrinters));
    }

    #[tokio::test]
    async fn list_printers_returns_error_when_queued() {
        let mock = MockTepraClient::new();
        mock.push_list_printers(Err(TepraError::Creator { errcode: 1 }));

        let err = mock.list_printers().await.unwrap_err();
        assert!(matches!(err, TepraError::Creator { errcode: 1 }));
    }

    // --- version -------------------------------------------------------------

    #[tokio::test]
    async fn version_records_call_and_returns_response() {
        let mock = MockTepraClient::new();
        mock.push_version(Ok(VersionResponse {
            web_api_module: "1.0.0".into(),
            printer_drivers: vec![DriverVersion {
                driver_name: "TEPRA".into(),
                version: "2.0".into(),
            }],
        }));

        let v = mock.version().await.unwrap();
        assert_eq!(v.web_api_module, "1.0.0");

        assert!(matches!(mock.calls()[0], MockCall::Version));
    }

    // --- autoselect ----------------------------------------------------------

    #[tokio::test]
    async fn autoselect_records_call() {
        let mock = MockTepraClient::new();
        mock.push_autoselect(Ok(AutoselectResponse {
            printer_name: "PT-P710BT".into(),
        }));

        let res = mock.autoselect().await.unwrap();
        assert_eq!(res.printer_name, "PT-P710BT");
        assert!(matches!(mock.calls()[0], MockCall::Autoselect));
    }

    // --- printer_info --------------------------------------------------------

    #[tokio::test]
    async fn printer_info_records_name() {
        let mock = MockTepraClient::new();
        mock.push_printer_info(Ok(PrinterInfoResponse {
            driver_name: "TEPRA".into(),
            dpi: 360,
            tape_list: vec![TapeEntry { tape_id: 261 }],
        }));

        mock.printer_info("PT-P710BT").await.unwrap();

        let calls = mock.calls();
        if let MockCall::PrinterInfo(name) = &calls[0] {
            assert_eq!(name, "PT-P710BT");
        } else {
            panic!("unexpected call variant");
        }
    }

    // --- online_status -------------------------------------------------------

    #[tokio::test]
    async fn online_status_records_name() {
        let mock = MockTepraClient::new();
        mock.push_online_status(Ok(OnlineStatusResponse { online: true }));

        let res = mock.online_status("PT-P710BT").await.unwrap();
        assert!(res.online);
        assert!(matches!(&mock.calls()[0], MockCall::OnlineStatus(n) if n == "PT-P710BT"));
    }

    // --- lw_status -----------------------------------------------------------

    #[tokio::test]
    async fn lw_status_records_name() {
        let mock = MockTepraClient::new();
        mock.push_lw_status(Ok(LwStatusResponse {
            tape_id: 261,
            tape_kind: 0,
            error: 0,
            br_tape_kind: 0,
            status: 0,
            status_type: 4,
            tape_sw: None,
            t8_option: None,
        }));

        mock.lw_status("PT-P710BT").await.unwrap();
        assert!(matches!(&mock.calls()[0], MockCall::LwStatus(n) if n == "PT-P710BT"));
    }

    // --- tapefeed ------------------------------------------------------------

    #[tokio::test]
    async fn tapefeed_records_name() {
        let mock = MockTepraClient::new();
        mock.push_tapefeed(Ok(()));

        mock.tapefeed("PT-P710BT", false).await.unwrap();
        assert!(matches!(&mock.calls()[0], MockCall::Tapefeed(n, false) if n == "PT-P710BT"));
    }

    // --- job_progress --------------------------------------------------------

    #[tokio::test]
    async fn job_progress_records_name_and_jobid() {
        let mock = MockTepraClient::new();
        mock.push_job_progress(Ok(JobProgressResponse {
            data_progress: 50,
            page_number: 1,
            total_page_count: 2,
            job_end: false,
            canceled: false,
            status_error: 0,
        }));

        mock.job_progress("PT-P710BT", 42).await.unwrap();
        assert!(matches!(&mock.calls()[0], MockCall::JobProgress(n, 42) if n == "PT-P710BT"));
    }

    // --- job_info ------------------------------------------------------------

    #[tokio::test]
    async fn job_info_records_name_and_jobid() {
        let mock = MockTepraClient::new();
        mock.push_job_info(Ok(JobInfoResponse { status: 0 }));

        mock.job_info("PT-P710BT", 7).await.unwrap();
        assert!(matches!(&mock.calls()[0], MockCall::JobInfo(n, 7) if n == "PT-P710BT"));
    }

    // --- job_control ---------------------------------------------------------

    #[tokio::test]
    async fn job_control_records_call() {
        let mock = MockTepraClient::new();
        mock.push_job_control(Ok(()));

        mock.job_control(
            "PT-P710BT",
            JobControlRequest {
                jobid: 1,
                control: 3,
            },
        )
        .await
        .unwrap();

        if let MockCall::JobControl(name, req) = &mock.calls()[0] {
            assert_eq!(name, "PT-P710BT");
            assert_eq!(req.control, 3);
        } else {
            panic!("unexpected call variant");
        }
    }

    // --- import_frame --------------------------------------------------------

    #[tokio::test]
    async fn import_frame_records_call() {
        let mock = MockTepraClient::new();
        mock.push_import_frame(Ok(vec![ImportFrameItem {
            id: 1,
            attribute: ImportFrameAttribute::Text,
            width: 100,
            height: 50,
        }]));

        let req = ImportFrameRequest {
            template_file: FilePayload {
                file_name: "label.lbx".into(),
                base64_str: "AAAA".into(),
            },
        };
        let result = mock.import_frame(req).await.unwrap();
        assert_eq!(result[0].id, 1);
        assert!(matches!(mock.calls()[0], MockCall::ImportFrame(_)));
    }

    // --- get_margin ----------------------------------------------------------

    #[tokio::test]
    async fn get_margin_records_name() {
        let mock = MockTepraClient::new();
        mock.push_get_margin(Ok(GetMarginResponse {
            top: 2,
            bottom: 2,
            left_right: 4,
        }));

        mock.get_margin(
            "PT-P710BT",
            GetMarginRequest {
                tape_id: 261,
                template_file: None,
            },
        )
        .await
        .unwrap();

        assert!(matches!(&mock.calls()[0], MockCall::GetMargin(n, _) if n == "PT-P710BT"));
    }

    // --- multiple calls ordering ---------------------------------------------

    #[tokio::test]
    async fn multiple_calls_recorded_in_order() {
        let mock = MockTepraClient::new();
        mock.push_list_printers(Ok(vec![]));
        mock.push_autoselect(Ok(AutoselectResponse {
            printer_name: "PT-P710BT".into(),
        }));

        mock.list_printers().await.unwrap();
        mock.autoselect().await.unwrap();

        let calls = mock.calls();
        assert_eq!(calls.len(), 2);
        assert!(matches!(calls[0], MockCall::ListPrinters));
        assert!(matches!(calls[1], MockCall::Autoselect));
    }

    // --- dyn dispatch --------------------------------------------------------

    #[tokio::test]
    async fn usable_as_dyn_tepra_client() {
        let mock = MockTepraClient::new();
        mock.push_list_printers(Ok(vec![printer("LABEL-1")]));

        let client: &dyn TepraClient = &mock;
        let result = client.list_printers().await.unwrap();
        assert_eq!(result[0].printer_name, "LABEL-1");
    }
}
