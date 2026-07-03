//! Handlers for `/api/printer*` endpoints — one-to-one facade over Creator `WebAPI`.
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::literal_string_with_formatting_args
)]

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use opentelemetry_semantic_conventions::attribute as semconv;
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
use tracing::{Span, instrument};

use super::err_502;

/// `GET /api/printer` — list all connected printers.
#[axum::debug_handler]
#[instrument(
    name = "handler.list_printers",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/printer",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn list_printers(
    State(client): State<Arc<dyn TepraClient>>,
) -> Result<Json<Vec<PrinterListItem>>, StatusCode> {
    let result = client.list_printers().await.map(Json).map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `GET /api/printer/version` — `WebAPI` module and driver versions.
#[axum::debug_handler]
#[instrument(
    name = "handler.version",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/printer/version",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn version(
    State(client): State<Arc<dyn TepraClient>>,
) -> Result<Json<VersionResponse>, StatusCode> {
    let result = client.version().await.map(Json).map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `GET /api/printer/autoselect` — currently auto-selected printer name.
#[axum::debug_handler]
#[instrument(
    name = "handler.autoselect",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/printer/autoselect",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn autoselect(
    State(client): State<Arc<dyn TepraClient>>,
) -> Result<Json<AutoselectResponse>, StatusCode> {
    let result = client.autoselect().await.map(Json).map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `GET /api/printer/info/{name}` — printer capabilities and tape list.
#[axum::debug_handler]
#[instrument(
    name = "handler.printer_info",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/printer/info/{name}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn printer_info(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
) -> Result<Json<PrinterInfoResponse>, StatusCode> {
    let result = client.printer_info(&name).await.map(Json).map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `GET /api/printer/onlinestatus/{name}` — printer online/offline state.
#[axum::debug_handler]
#[instrument(
    name = "handler.online_status",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/printer/onlinestatus/{name}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn online_status(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
) -> Result<Json<OnlineStatusResponse>, StatusCode> {
    let result = client.online_status(&name).await.map(Json).map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `GET /api/printer/lwstatus/{name}` — detailed tape and device status.
#[axum::debug_handler]
#[instrument(
    name = "handler.lw_status",
    skip_all,
    fields(
        http.request.method = "GET",
        http.route = "/api/printer/lwstatus/{name}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn lw_status(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
) -> Result<Json<LwStatusResponse>, StatusCode> {
    let result = client.lw_status(&name).await.map(Json).map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}

/// `POST /api/printer/getmargin/{name}` — compute print margins.
#[axum::debug_handler]
#[instrument(
    name = "handler.get_margin",
    skip_all,
    fields(
        http.request.method = "POST",
        http.route = "/api/printer/getmargin/{name}",
        http.response.status_code = tracing::field::Empty,
        url.scheme = tracing::field::Empty,
    )
)]
pub async fn get_margin(
    State(client): State<Arc<dyn TepraClient>>,
    Path(name): Path<String>,
    Json(req): Json<GetMarginRequest>,
) -> Result<Json<GetMarginResponse>, StatusCode> {
    let result = client
        .get_margin(&name, req)
        .await
        .map(Json)
        .map_err(err_502);
    Span::current().record(
        semconv::HTTP_RESPONSE_STATUS_CODE,
        if result.is_ok() { 200_i64 } else { 502_i64 },
    );
    result
}
