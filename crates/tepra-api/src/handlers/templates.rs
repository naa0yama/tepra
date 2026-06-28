//! Handlers for template-related endpoints.
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

use axum::{Json, extract::State, http::StatusCode};
use tepra_core::dto::template::{ImportFrameItem, ImportFrameRequest};

use crate::{state::AppState, templates::TemplateEntry};

/// `POST /api/printer/template/importframe` — extract frame list from a template file.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn import_frame(
    _state: State<AppState>,
    _req: Json<ImportFrameRequest>,
) -> Result<Json<Vec<ImportFrameItem>>, StatusCode> {
    todo!("T14f")
}

/// `GET /api/templates` — list template files in the configured template directory.
#[axum::debug_handler]
#[allow(clippy::todo)]
pub async fn list_template_files(
    _state: State<AppState>,
) -> Result<Json<Vec<TemplateEntry>>, StatusCode> {
    todo!("T14f")
}
