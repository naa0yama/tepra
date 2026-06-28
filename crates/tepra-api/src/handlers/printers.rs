//! Handlers for `/api/printer*` endpoints — one-to-one facade over Creator `WebAPI`.
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tepra_core::{
    client::traits::TepraClient,
    dto::{
        printer::{
            AutoselectResponse, LwStatusResponse, OnlineStatusResponse, PrinterInfoResponse,
            PrinterListItem, VersionResponse,
        },
        template::{GetMarginRequest, GetMarginResponse},
    },
};

use super::err_502;

/// `GET /api/printer` — list all connected printers.
#[axum::debug_handler]
pub async fn list_printers(
    State(client): State<Arc<dyn TepraClient>>,
) -> Result<Json<Vec<PrinterListItem>>, StatusCode> {
    client.list_printers().await.map(Json).map_err(err_502)
}

/// `GET /api/printer/version` — `WebAPI` module and driver versions.
#[axum::debug_handler]
pub async fn version(
    State(client): State<Arc<dyn TepraClient>>,
) -> Result<Json<VersionResponse>, StatusCode> {
    client.version().await.map(Json).map_err(err_502)
}

/// `GET /api/printer/autoselect` — currently auto-selected printer name.
#[axum::debug_handler]
pub async fn autoselect(
    State(client): State<Arc<dyn TepraClient>>,
) -> Result<Json<AutoselectResponse>, StatusCode> {
    client.autoselect().await.map(Json).map_err(err_502)
}

/// `GET /api/printer/info/{name}` — printer capabilities and tape list.
#[axum::debug_handler]
pub async fn printer_info(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
) -> Result<Json<PrinterInfoResponse>, StatusCode> {
    client.printer_info(&name).await.map(Json).map_err(err_502)
}

/// `GET /api/printer/onlinestatus/{name}` — printer online/offline state.
#[axum::debug_handler]
pub async fn online_status(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
) -> Result<Json<OnlineStatusResponse>, StatusCode> {
    client.online_status(&name).await.map(Json).map_err(err_502)
}

/// `GET /api/printer/lwstatus/{name}` — detailed tape and device status.
#[axum::debug_handler]
pub async fn lw_status(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
) -> Result<Json<LwStatusResponse>, StatusCode> {
    client.lw_status(&name).await.map(Json).map_err(err_502)
}

/// `POST /api/printer/getmargin/{name}` — compute print margins.
#[axum::debug_handler]
pub async fn get_margin(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
    Json(req): Json<GetMarginRequest>,
) -> Result<Json<GetMarginResponse>, StatusCode> {
    client
        .get_margin(&name, req)
        .await
        .map(Json)
        .map_err(err_502)
}
