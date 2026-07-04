//! Custom `MakeSpan` / `OnResponse` for `OTel` HTTP server semantic conventions.
//!
//! Emits the server span at `INFO` level so it survives the default `RUST_LOG=warn`
//! filter in production. Attribute names follow `OTel` HTTP semconv 1.23+.

use std::time::Duration;

use axum::extract::MatchedPath;
use axum::http::{Request, Response};
use opentelemetry::global;
use opentelemetry_http::HeaderExtractor;
use opentelemetry_semantic_conventions::attribute;
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
        let route = request
            .extensions()
            .get::<MatchedPath>()
            .map_or("", MatchedPath::as_str);

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
            http.route = route,
            http.response.status_code = tracing::field::Empty,
        );
        // AlreadyStarted error is expected when no parent; ignore it.
        let _ = span.set_parent(parent_cx);
        span
    }
}

/// [`OnResponse`][tower_http::trace::OnResponse] that records the HTTP response
/// status code onto the active server span.
#[derive(Clone, Debug, Default)]
pub struct OtelOnResponse;

impl<B> tower_http::trace::OnResponse<B> for OtelOnResponse {
    fn on_response(self, response: &Response<B>, _latency: Duration, span: &Span) {
        span.record(
            attribute::HTTP_RESPONSE_STATUS_CODE,
            i64::from(response.status().as_u16()),
        );
    }
}
