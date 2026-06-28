//! Handlers for template-related endpoints.
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

use axum::{Json, extract::State, http::StatusCode};
use tepra_core::dto::template::{ImportFrameItem, ImportFrameRequest};

use super::err_502;
use crate::{state::AppState, templates::TemplateEntry};

/// `POST /api/printer/template/importframe` — extract frame list from a template file.
#[axum::debug_handler]
pub async fn import_frame(
    State(state): State<AppState>,
    Json(req): Json<ImportFrameRequest>,
) -> Result<Json<Vec<ImportFrameItem>>, StatusCode> {
    state
        .client
        .import_frame(req)
        .await
        .map(Json)
        .map_err(err_502)
}

/// `GET /api/templates` — list template files in the configured template directory.
#[axum::debug_handler]
pub async fn list_template_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<TemplateEntry>>, StatusCode> {
    crate::templates::list_templates(&state.template_dir)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
