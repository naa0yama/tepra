//! Custom `MakeSpan` / `OnResponse` for `OTel` HTTP server semantic conventions.
//!
//! Emits the server span at `INFO` level so it survives the default `RUST_LOG=warn`
//! filter in production. Attribute names follow `OTel` HTTP semconv 1.23+.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, MatchedPath, State};
use axum::http::{Request, Response};
use axum::middleware::Next;
use opentelemetry::global;
use opentelemetry::trace::Status;
use opentelemetry_http::HeaderExtractor;
use opentelemetry_semantic_conventions::attribute;
use tepra_core::otel::metrics::Meters;
use tracing::{Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

/// [`MakeSpan`][tower_http::trace::MakeSpan] that creates HTTP server spans at INFO
/// level with `OTel` HTTP semantic-convention attribute names.
#[derive(Clone, Debug, Default)]
pub struct OtelHttpServerMakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for OtelHttpServerMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let method = request.method().as_str();
        let scheme = request.uri().scheme_str().unwrap_or("http");
        let path = request.uri().path();
        let query = request.uri().query().unwrap_or("");

        let route = request
            .extensions()
            .get::<MatchedPath>()
            .map_or("", MatchedPath::as_str);

        // Host header: strip optional port for server.address
        let host = request
            .headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let server_address = host.split(':').next().unwrap_or(host);

        let url_full = if query.is_empty() {
            format!("{scheme}://{host}{path}")
        } else {
            format!("{scheme}://{host}{path}?{query}")
        };

        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let protocol_version = format!("{:?}", request.version());

        let client_address = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_default();

        let span_name = if route.is_empty() {
            method.to_owned()
        } else {
            format!("{method} {route}")
        };

        let parent_cx =
            global::get_text_map_propagator(|p| p.extract(&HeaderExtractor(request.headers())));

        let span = tracing::span!(
            Level::INFO,
            "http.server.request",
            otel.name = span_name,
            otel.kind = "SERVER",
            http.request.method = method,
            url.scheme = scheme,
            url.path = path,
            url.query = query,
            url.full = url_full,
            http.route = route,
            user_agent.original = user_agent,
            network.protocol.version = protocol_version,
            server.address = server_address,
            client.address = client_address,
            http.response.status_code = tracing::field::Empty,
            error.type = tracing::field::Empty,
        );
        // AlreadyStarted error is expected when no parent; ignore it.
        let _ = span.set_parent(parent_cx);
        span
    }
}

/// [`OnResponse`][tower_http::trace::OnResponse] that records `http.response.status_code`,
/// span status (`Error` for 5xx), and `error.type` on the active server span.
///
/// Metric recording (`http.server.request.duration`) is handled separately by
/// [`server_metrics_mw`], which has access to the originating `Request` and can
/// populate `http.request.method` and `http.route` correctly.
#[derive(Clone, Debug, Default)]
pub struct OtelOnResponse;

impl<B> tower_http::trace::OnResponse<B> for OtelOnResponse {
    fn on_response(self, response: &Response<B>, _latency: Duration, span: &Span) {
        let status = response.status().as_u16();
        span.record(attribute::HTTP_RESPONSE_STATUS_CODE, i64::from(status));

        if status >= 500 {
            let code_str = status.to_string();
            span.set_status(Status::Error {
                description: std::borrow::Cow::Owned(code_str.clone()),
            });
            span.record(attribute::ERROR_TYPE, code_str.as_str());
        }
    }
}

/// Axum middleware that records `http.server.request.duration` with `http.request.method`
/// and `http.route` populated from the live request.
///
/// Mount this via `router.layer(axum::middleware::from_fn_with_state(meters, server_metrics_mw))`
/// *before* `TraceLayer` so it wraps the fully-routed request.
pub async fn server_metrics_mw(
    State(meters): State<Arc<Meters>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> axum::response::Response {
    let method = request.method().as_str().to_owned();
    let route = request.extensions().get::<MatchedPath>().map_or_else(
        || request.uri().path().to_owned(),
        |p| p.as_str().to_owned(),
    );

    let start = Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    let error_type = if status >= 500 {
        Some(status.to_string())
    } else {
        None
    };

    meters.record_http_server_request(elapsed, &method, status, &route, error_type.as_deref());
    response
}
