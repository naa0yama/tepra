# 0006. HTTP observability with tower-http TraceLayer

- Status: Accepted
- Date: 2026-06-28
- Deciders: project owner

## Context

The project exports traces to an OTel collector via OTLP (`opentelemetry-otlp`,
`tracing-opentelemetry`). HTTP request/response spans are needed so that
latency, status codes, and route labels are visible in Jaeger / o2.

Candidates for generating those spans:

- `tower-http` `TraceLayer` — a Tower middleware that wraps every request in a
  `tracing` span before it reaches any handler.
- Per-handler `#[instrument]` — annotate each axum handler individually.
- Custom OTel middleware — write a `tower::Layer` that calls the OTel API
  directly.

## Decision

Use `tower_http::trace::TraceLayer::new_for_http()` applied once at the top of
the composed router in `tepra-web/src/main.rs`. The existing
`tracing-opentelemetry` subscriber bridge propagates those spans to the OTLP
exporter without any additional code.

## Consequences

Positive:

- Single `.layer(TraceLayer::new_for_http())` call covers all routes uniformly.
- No per-handler boilerplate; new routes are instrumented automatically.
- Spans flow into the existing OTel pipeline via `tracing-opentelemetry` with
  no extra configuration.
- `tower-http` was already declared in workspace dependencies; no new crate is
  introduced.

Negative:

- Span granularity is at the HTTP layer only. Business-logic spans inside
  handlers still require `#[instrument]` where needed.
- Default field set (method, URI, status) is sufficient for MVP; richer
  attributes (user-agent, request-id) need custom `MakeSpan` if required later.

## Alternatives Considered

- **Per-handler `#[instrument]`** — rejected for the HTTP observability layer.
  Every new route must remember to add the annotation; easy to miss. Acceptable
  as a complement for handler-internal logic, not as a replacement.
- **Custom OTel middleware** — rejected. Duplicates what `TraceLayer` already
  provides; adds maintenance burden with no benefit for this project's scope.

## History

- 2026-06-28: initial version
