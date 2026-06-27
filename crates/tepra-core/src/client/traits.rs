//! `TepraClient` trait — abstracts all 13 Creator `WebAPI` endpoints.

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

/// Async client interface covering all 13 TEPRA Creator `WebAPI` endpoints.
///
/// All implementations must be `Send + Sync` to support `Arc<dyn TepraClient>`.
#[async_trait]
pub trait TepraClient: Send + Sync {
    /// `GET /api/printer` — list all connected printers.
    async fn list_printers(&self) -> Result<Vec<PrinterListItem>, TepraError>;

    /// `GET /api/printer/version` — `WebAPI` module and driver versions.
    async fn version(&self) -> Result<VersionResponse, TepraError>;

    /// `GET /api/printer/autoselect` — currently auto-selected printer name.
    async fn autoselect(&self) -> Result<AutoselectResponse, TepraError>;

    /// `GET /api/printer/info/{name}` — printer capabilities and tape list.
    async fn printer_info(&self, name: &str) -> Result<PrinterInfoResponse, TepraError>;

    /// `GET /api/printer/onlinestatus/{name}` — printer online/offline state.
    async fn online_status(&self, name: &str) -> Result<OnlineStatusResponse, TepraError>;

    /// `GET /api/printer/lwstatus/{name}` — detailed tape and device status.
    async fn lw_status(&self, name: &str) -> Result<LwStatusResponse, TepraError>;

    /// `POST /api/printer/print/{name}` — submit a print job.
    async fn print(&self, name: &str, req: PrintRequest) -> Result<PrintResponse, TepraError>;

    /// `GET /api/printer/tapefeed/{name}?cutflag=<bool>` — advance tape; cut if `cutflag` is true.
    async fn tapefeed(&self, name: &str, cutflag: bool) -> Result<(), TepraError>;

    /// `GET /api/printer/job/progress/{name}?jobid=N` — poll job progress.
    async fn job_progress(&self, name: &str, jobid: u64)
    -> Result<JobProgressResponse, TepraError>;

    /// `GET /api/printer/job/info/{name}?jobid=N` — Win32 job status bitmask.
    async fn job_info(&self, name: &str, jobid: u64) -> Result<JobInfoResponse, TepraError>;

    /// `POST /api/printer/job/control/{name}` — pause / resume / cancel a job.
    async fn job_control(&self, name: &str, req: JobControlRequest) -> Result<(), TepraError>;

    /// `POST /api/printer/template/importframe` — extract frame list from a template file.
    async fn import_frame(
        &self,
        req: ImportFrameRequest,
    ) -> Result<Vec<ImportFrameItem>, TepraError>;

    /// `POST /api/printer/getmargin/{name}` — compute print margins for a tape/template combination.
    async fn get_margin(
        &self,
        name: &str,
        req: GetMarginRequest,
    ) -> Result<GetMarginResponse, TepraError>;
}
