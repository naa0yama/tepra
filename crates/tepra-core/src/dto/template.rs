//! DTOs for template Creator `WebAPI` endpoints.
//!
//! Endpoints covered:
//!   POST /api/printer/template/importframe  req: [`ImportFrameRequest`]  res: [`Vec<ImportFrameItem>`]
//!   POST /api/printer/getmargin/{name}      req: [`GetMarginRequest`]    res: [`GetMarginResponse`]

use serde::{Deserialize, Serialize};

use super::{enums::ImportFrameAttribute, job::FilePayload};

// ---------------------------------------------------------------------------
// POST /api/printer/template/importframe
// ---------------------------------------------------------------------------

/// Request body for `POST /api/printer/template/importframe`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFrameRequest {
    /// Template file to inspect for import-frame attributes.
    pub template_file: FilePayload,
}

/// One import frame entry returned by `POST /api/printer/template/importframe`.
///
/// The response body is an array of these items.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFrameItem {
    /// Import frame index within the template.
    pub id: u32,
    /// Import frame content type (see `TepraPrintImportFrameAttribute`).
    pub attribute: ImportFrameAttribute,
    /// Import frame width.
    pub width: u32,
    /// Import frame height.
    pub height: u32,
}

// ---------------------------------------------------------------------------
// POST /api/printer/getmargin/{name}
// ---------------------------------------------------------------------------

/// Request body for `POST /api/printer/getmargin/{name}`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMarginRequest {
    /// Tape ID to compute the margin for (see `TepraPrintTapeID` in the Creator `WebAPI` reference).
    #[serde(rename = "tapeID")]
    pub tape_id: u32,
    /// Optional template file; `null` when computing margin without a template.
    pub template_file: Option<FilePayload>,
}

/// Response body for `POST /api/printer/getmargin/{name}`.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMarginResponse {
    /// Top margin in 0.1mm units.
    pub top: u32,
    /// Bottom margin in 0.1mm units.
    pub bottom: u32,
    /// Left/right margin in 0.1mm units.
    pub left_right: u32,
}
