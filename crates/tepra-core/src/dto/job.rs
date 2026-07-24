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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePayload {
    /// Original file name (including extension).
    pub file_name: String,
    /// File content encoded as a Base64 string.
    pub base64_str: String,
}

// ---------------------------------------------------------------------------
// POST /api/printer/print/{name}
// ---------------------------------------------------------------------------

/// Files to be printed (template, CSV, or image — all optional).
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintFiles {
    /// Template file; combined with `csv_file` for merge printing, or used alone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_file: Option<FilePayload>,
    /// CSV file supplying merge data for `template_file`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_file: Option<FilePayload>,
    /// Image file for image printing (used alone, without a template).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_file: Option<FilePayload>,
}

/// Density mode sub-object within a print parameter.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DensityParam {
    /// 1 = specified density; other values reserved.
    pub mode: u32,
    /// Print density value, meaningful when `mode` selects a specified density.
    pub value: i32,
}

/// Error message output control within a print parameter.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessageParam {
    /// 1 = suppress, 2 = show.
    pub mode: u32,
    /// 0 = no file output.
    pub file_output: u32,
    /// Output path for the error message file, used when `file_output` enables it.
    pub file_path: String,
}

/// Wire-format print parameter as sent in `POST /api/printer/print/{name}`.
///
/// Integer fields use **REST wire values** (not JS SDK logical constants).
/// See `tepraprint_getWebApiPrintParameter` in `tepraprint.js` for the mapping.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintParameter {
    /// Number of print copies (1-999).
    pub copies: u32,
    /// 1=not cut, 2=each label, 3=after job.
    pub tape_cut: u32,
    /// 1=no half-cut, 2=half-cut.
    pub half_cut: u32,
    /// 1=high, 2=low, 3=middle.
    pub print_speed: u32,
    /// Print density setting.
    pub density: DensityParam,
    /// Tape ID to use for image printing (see `TepraPrintTapeID` in the Creator `WebAPI` reference).
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
    /// 1=tape cartridge setting, 2=print setting priority.
    pub priority_cut_setting: u32,
    /// 1=continuous (joined labels), 2=continuous (separated labels).
    pub half_cut_separate: u32,
    /// Left/right margin in 0.1mm units.
    pub margin_left_right: u32,
    /// 1=hide, 2=show.
    pub display_tape_width: u32,
    /// Error message output settings.
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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintRequest {
    /// Files to be printed.
    pub print_file: PrintFiles,
    /// Print settings applied to this job.
    pub print_parameter: PrintParameter,
}

/// Response body for `POST /api/printer/print/{name}` on success.
///
/// `result = 1` means the job was enqueued; `jobid` is the assigned ID.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintResponse {
    /// 1 = the job was enqueued successfully.
    pub result: u32,
    /// ID assigned to the enqueued print job.
    pub jobid: u64,
}

// ---------------------------------------------------------------------------
// GET /api/printer/job/progress/{name}?jobid=
// ---------------------------------------------------------------------------

/// Response body for `GET /api/printer/job/progress/{name}?jobid=N`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressResponse {
    /// Data transfer progress, as a percentage.
    pub data_progress: u32,
    /// Page number currently being transferred.
    pub page_number: u32,
    /// Total page count for this print job.
    pub total_page_count: u32,
    /// Whether the print job has finished (`false` while printing).
    pub job_end: bool,
    /// Whether the print job was canceled.
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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobInfoResponse {
    /// Win32 job status bitmask; bit 0 (`0x01`) = paused.
    pub status: u32,
}

// ---------------------------------------------------------------------------
// POST /api/printer/job/control/{name}
// ---------------------------------------------------------------------------

/// Request body for `POST /api/printer/job/control/{name}`.
///
/// `control`: 1 = pause, 2 = resume, 3 = cancel.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobControlRequest {
    /// ID of the print job to control.
    pub jobid: u64,
    /// Requested control action: 1 = pause, 2 = resume, 3 = cancel.
    pub control: u32,
}
