# 0010. OpenAPI schema derivation in tepra-core behind a feature gate

- Status: Accepted
- Date: 2026-07-24
- Deciders: naa0yama, agmgr

## Context

The built-in HTTP API (`/api/*`, 13 endpoints) is defined in `crates/tepra`
and wraps the external TEPRA Creator WebAPI. We are adding a `/ui/api`
reference page that displays request/response schemas and offers a Try-it-out
execution surface, with the OpenAPI document generated from code (utoipa) so it
cannot drift.

The workspace already declares `utoipa = { version = "5", features =
["axum_extras"] }` but does not use it. The request/response DTOs live in
`tepra-core`, which is deliberately structured as an interface-agnostic domain

- client library: main functionality is concentrated in `tepra-core` so that
  any front end (`tepra` web today, a CLI later) only has to write a thin
  interface layer. OpenAPI, however, is conceptually an HTTP/web concern.

The question is where the `utoipa::ToSchema` derivations belong, without
eroding `tepra-core`'s web-agnostic property.

## Decision

Derive `ToSchema` on the DTOs **in `tepra-core`**, but gate it behind a new
`schema` Cargo feature:

```toml
# crates/tepra-core/Cargo.toml
[features]
schema = ["dep:utoipa"]
```

```rust
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterListItem { /* ... */ }
```

`crates/tepra` enables it via `tepra-core = { features = ["schema"] }`.

The HTTP operation metadata — `#[utoipa::path]` annotations, the
`#[derive(OpenApi)]` `ApiDoc` aggregation, and the `GET /api/openapi.json`
route — stays in `crates/tepra`, because method/path/params/responses are
HTTP-specific and have no meaning in a non-web consumer.

Split summary: **`tepra-core` owns data shapes (schemas); `tepra` owns the
HTTP interface (paths, aggregation, serving).**

## Consequences

Positive:

- `tepra-core`'s default build stays utoipa-free, preserving its web-agnostic
  character — a consumer that does not want OpenAPI pays nothing.
- The schema-generation capability lives in core as an opt-in capability. A
  future CLI that wants to emit JSON schema / OpenAPI enables the same feature
  and writes only its own interface layer — exactly the intended architecture.
- OpenAPI schemas are code-derived; no hand-written spec to drift.

Negative:

- `#[cfg_attr(feature = "schema", ...)]` on ~15 DTOs is boilerplate a reader
  must understand.
- CI must build/test `tepra-core` with `--features schema` (and `tepra`, which
  enables it transitively) so the feature-on path does not rot.

## Alternatives Considered

- **Unconditional `ToSchema` derive in `tepra-core`** (no feature): simpler,
  but forces the `utoipa` dependency on every consumer including those that
  never produce docs, weakening the web-agnostic property. Rejected.
- **Define schemas in `tepra` (web) via manual `ToSchema` impls**: keeps core
  pristine, but duplicates the DTO shape in a second place and drifts from the
  serde definitions. Rejected.
- **Third-party rendered UI (`utoipa-swagger-ui` / `utoipa-scalar`)**: sidesteps
  the layering question but clashes with the DaisyUI theme and ships a heavy
  bundle. Out of scope for this ADR (presentation choice, recorded in the plan).

## History

- 2026-07-24: initial version
