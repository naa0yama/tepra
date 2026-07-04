//! Custom `MakeSpan` / `OnResponse` for `OTel` HTTP server semantic conventions.
//!
//! Emits the server span at `INFO` level so it survives the default `RUST_LOG=warn`
//! filter in production. Attribute names follow `OTel` HTTP semconv 1.23+.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ConnectInfo, MatchedPath};
use axum::http::{Request, Response};
use opentelemetry::global;
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
        );
        // AlreadyStarted error is expected when no parent; ignore it.
        let _ = span.set_parent(parent_cx);
        span
    }
}

/// [`OnResponse`][tower_http::trace::OnResponse] that records the HTTP response
/// status code onto the active server span and emits `http.server.request.duration`.
///
/// Method and route are not carried through because `tower_http::trace::OnResponse`
/// only receives the `Response`, not the originating `Request`. These attributes are
/// left empty; a future refactor with shared-state `MakeSpan` can improve fidelity.
#[derive(Clone, Debug)]
pub struct OtelOnResponse {
    meters: Arc<Meters>,
}

impl OtelOnResponse {
    /// Create a new [`OtelOnResponse`] backed by the given [`Meters`].
    #[must_use]
    pub const fn new(meters: Arc<Meters>) -> Self {
        Self { meters }
    }
}

impl Default for OtelOnResponse {
    fn default() -> Self {
        Self::new(Arc::new(Meters::new()))
    }
}

impl<B> tower_http::trace::OnResponse<B> for OtelOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status = response.status().as_u16();
        span.record(attribute::HTTP_RESPONSE_STATUS_CODE, i64::from(status));

        let error_type = if status >= 500 {
            Some(status.to_string())
        } else {
            None
        };
        self.meters.record_http_server_request(
            latency.as_secs_f64(),
            "",
            status,
            "",
            error_type.as_deref(),
        );
    }
}
