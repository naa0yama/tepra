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
    pub driver_name: String,
    pub version: String,
}

/// Response body for `GET /api/printer/version`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    pub web_api_module: String,
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
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
}

/// Response body for `GET /api/printer/info/{name}`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterInfoResponse {
    pub driver_name: String,
    pub dpi: u32,
    pub tape_list: Vec<TapeEntry>,
}

// ---------------------------------------------------------------------------
// GET /api/printer/onlinestatus/{name}
// ---------------------------------------------------------------------------

/// Response body for `GET /api/printer/onlinestatus/{name}`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineStatusResponse {
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
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
    pub tape_kind: i32,
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
