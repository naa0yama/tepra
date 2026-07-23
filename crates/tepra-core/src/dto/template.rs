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
    pub template_file: FilePayload,
}

/// One import frame entry returned by `POST /api/printer/template/importframe`.
///
/// The response body is an array of these items.
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFrameItem {
    pub id: u32,
    pub attribute: ImportFrameAttribute,
    pub width: u32,
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
    pub top: u32,
    pub bottom: u32,
    pub left_right: u32,
}
