//! DTOs for print-job Creator `WebAPI` endpoints.
//!
//! Endpoints covered:
//!   POST /api/printer/print/{name}            req: [`PrintRequest`]   res: [`PrintResponse`]
//!   GET  /api/printer/tapefeed/{name}         (no req/res body on success)
//!   GET  /api/printer/job/progress/{name}     res: [`JobProgressResponse`]
//!   GET  /api/printer/job/info/{name}         res: [`JobInfoResponse`]
//!   POST /api/printer/job/control/{name}      req: [`JobControlRequest`]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared file object (used in print and template requests)
// ---------------------------------------------------------------------------

/// Base64-encoded file payload used in request bodies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePayload {
    pub file_name: String,
    pub base64_str: String,
}

// ---------------------------------------------------------------------------
// POST /api/printer/print/{name}
// ---------------------------------------------------------------------------

/// Files to be printed (template, CSV, or image — all optional).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintFiles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_file: Option<FilePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_file: Option<FilePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_file: Option<FilePayload>,
}

/// Density mode sub-object within a print parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DensityParam {
    /// 1 = specified density; other values reserved.
    pub mode: u32,
    pub value: i32,
}

/// Error message output control within a print parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessageParam {
    /// 1 = suppress, 2 = show.
    pub mode: u32,
    /// 0 = no file output.
    pub file_output: u32,
    pub file_path: String,
}

/// Wire-format print parameter as sent in `POST /api/printer/print/{name}`.
///
/// Integer fields use **REST wire values** (not JS SDK logical constants).
/// See `tepraprint_getWebApiPrintParameter` in `tepraprint.js` for the mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintParameter {
    pub copies: u32,
    /// 1=not cut, 2=each label, 3=after job.
    pub tape_cut: u32,
    /// 1=no half-cut, 2=half-cut.
    pub half_cut: u32,
    /// 1=high, 2=low, 3=middle.
    pub print_speed: u32,
    pub density: DensityParam,
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
    /// 1=tape cartridge setting, 2=print setting priority.
    pub priority_cut_setting: u32,
    /// 1=continuous (joined labels), 2=continuous (separated labels).
    pub half_cut_separate: u32,
    pub margin_left_right: u32,
    /// 1=hide, 2=show.
    pub display_tape_width: u32,
    pub error_message: ErrorMessageParam,
    /// 1=hide, 2=show.
    pub display_transfer_tape: u32,
    /// 1=hide, 2=show.
    pub display_print_setting: u32,
    /// 0=include header row, 1=skip header row.
    pub cut_title: u32,
    /// 0=no conversion, 1=convert half-width kana to full-width.
    pub kana_zen: u32,
    /// 1=no preview, 2=show preview.
    pub display_print_preview: u32,
    /// 0=no stretch, 1=stretch.
    pub stretch_image: u32,
}

/// Request body for `POST /api/printer/print/{name}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintRequest {
    pub print_file: PrintFiles,
    pub print_parameter: PrintParameter,
}

/// Response body for `POST /api/printer/print/{name}` on success.
///
/// `result = 1` means the job was enqueued; `jobid` is the assigned ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintResponse {
    pub result: u32,
    pub jobid: u64,
}

// ---------------------------------------------------------------------------
// GET /api/printer/job/progress/{name}?jobid=
// ---------------------------------------------------------------------------

/// Response body for `GET /api/printer/job/progress/{name}?jobid=N`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressResponse {
    pub data_progress: u32,
    pub page_number: u32,
    pub total_page_count: u32,
    pub job_end: bool,
    pub canceled: bool,
    /// Device status error code (matches [`super::enums::StatusError`] values).
    pub status_error: u32,
}

// ---------------------------------------------------------------------------
// GET /api/printer/job/info/{name}?jobid=
// ---------------------------------------------------------------------------

/// Response body for `GET /api/printer/job/info/{name}?jobid=N`.
///
/// `status` is a Win32 job status bitmask. Bit 0 (`0x01`) = paused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobInfoResponse {
    pub status: u32,
}

// ---------------------------------------------------------------------------
// POST /api/printer/job/control/{name}
// ---------------------------------------------------------------------------

/// Request body for `POST /api/printer/job/control/{name}`.
///
/// `control`: 1 = pause, 2 = resume, 3 = cancel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobControlRequest {
    pub jobid: u64,
    pub control: u32,
}
