//! DTOs for printer-level Creator `WebAPI` endpoints.
//!
//! Endpoints covered:
//!   GET  /api/printer                    → [`PrinterListItem`] (array)
//!   GET  /api/printer/version            → [`VersionResponse`]
//!   GET  /api/printer/autoselect         → [`AutoselectResponse`]
//!   GET  /api/printer/info/{name}        → [`PrinterInfoResponse`]
//!   GET  /api/printer/onlinestatus/{name}→ [`OnlineStatusResponse`]
//!   GET  /api/printer/lwstatus/{name}    → [`LwStatusResponse`]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GET /api/printer  — printer list
// ---------------------------------------------------------------------------

/// One element of the printer list array.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterListItem {
    /// Printer name as registered with the driver (used as the `{name}` path param elsewhere).
    pub printer_name: String,
}

// ---------------------------------------------------------------------------
// GET /api/printer/version
// ---------------------------------------------------------------------------

/// Driver version entry within a version response.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverVersion {
    /// Printer driver name.
    pub driver_name: String,
    /// Printer driver version string.
    pub version: String,
}

/// Response body for `GET /api/printer/version`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    /// `WebAPI` communication module version string.
    pub web_api_module: String,
    /// Version info for each installed printer driver.
    pub printer_drivers: Vec<DriverVersion>,
}

// ---------------------------------------------------------------------------
// GET /api/printer/autoselect
// ---------------------------------------------------------------------------

/// Response body for `GET /api/printer/autoselect`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoselectResponse {
    /// Printer name auto-selected by the driver.
    pub printer_name: String,
}

// ---------------------------------------------------------------------------
// GET /api/printer/info/{name}
// ---------------------------------------------------------------------------

/// Tape entry within a printer info response.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TapeEntry {
    /// Loaded tape ID (see `TepraPrintTapeID` in the Creator `WebAPI` reference).
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
}

/// Response body for `GET /api/printer/info/{name}`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterInfoResponse {
    /// Printer driver name.
    pub driver_name: String,
    /// Print resolution in dots per inch.
    pub dpi: u32,
    /// Tapes currently loaded or available on the printer.
    pub tape_list: Vec<TapeEntry>,
}

// ---------------------------------------------------------------------------
// GET /api/printer/onlinestatus/{name}
// ---------------------------------------------------------------------------

/// Response body for `GET /api/printer/onlinestatus/{name}`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineStatusResponse {
    /// Whether the printer is currently online.
    pub online: bool,
}

// ---------------------------------------------------------------------------
// GET /api/printer/lwstatus/{name}
// ---------------------------------------------------------------------------

/// Raw LW status response from `GET /api/printer/lwstatus/{name}`.
///
/// Field semantics are interpreted by `tepraprint.js`
/// `tepraprint_fetchPrinterStatus_Async`. Optional fields (`tapeSw`,
/// `t8Option`) are only present for `statusType >= 5`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LwStatusResponse {
    /// Loaded tape ID (see `TepraPrintTapeID` in the Creator `WebAPI` reference).
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
    /// Tape type (see `TepraPrintTapeKind` in the Creator `WebAPI` reference).
    pub tape_kind: i32,
    /// Device error code (see `TepraPrintStatusError` in the Creator `WebAPI` reference).
    pub error: u32,
    /// Internal tape type discriminator used by SDK to override `tapeKind`.
    pub br_tape_kind: u32,
    /// Device status byte (0x12 = device using, 0x13 = firmware updating).
    pub status: u32,
    /// Status structure version; `>= 5` enables tape switch and option fields.
    pub status_type: u32,
    /// Tape switch bitmap (optional, present when `statusType >= 5`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tape_sw: Option<u32>,
    /// Tape option bitmap (optional, present when `statusType >= 5`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t8_option: Option<u32>,
}
