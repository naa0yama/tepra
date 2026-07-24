# 0001. Split tepra-api into three crates (core/api/web)

- Status: Accepted
- Date: 2026-06-27
- Deciders: project owner

## Context

The project replaces the `brust` boilerplate with a Rust workspace that
serves a web UI and REST API in front of the KING JIM TEPRA Creator
WebAPI. MVP2 scope (template + CSV merge printing) is modest enough that
a single-crate layout would compile and ship. Future work, however,
includes:

- A separate CLI tool that drives the same backend client.
- An eventual second frontend (PWA / mobile) reusing the REST layer.
- Per-layer test isolation — backend stubbing for the domain layer
  without booting Axum, snapshot tests for templates without spinning up
  the queue worker.

A single crate would entangle these concerns and force every test to
build the full transitive dependency tree (reqwest, axum, askama,
tower-http, opentelemetry).

## Decision

Split the workspace into three crates with a one-way dependency chain:

```
tepra-web -> tepra -> tepra-core
```

- `tepra-core` (library): domain types, KING JIM WebAPI client trait and
  HTTP implementation, capability matrix, error type. No HTTP server,
  no templating.
- `tepra` (library): Axum router mounted at `/api/v1`, job queue
  manager, template manager, request validation. Consumes `tepra-core`.
- `tepra-web` (library + binary `tepra`): Askama templates, HTMX
  page handlers, static asset mount, binary entrypoint that wires
  config + OTel + queue + nested REST router.

## Consequences

Positive:

- Each layer is independently testable; `tepra-core` is unit-tested with
  `wiremock` and never links Axum.
- A future CLI crate can depend on `tepra-core` alone, skipping the web
  stack.
- Compile incrementality improves when iterating on UI templates.

Negative:

- Three `Cargo.toml` files and three `lib.rs` skeletons up front.
- Cross-crate refactoring (renaming a domain type) touches multiple
  manifests.
- Workspace lint / feature configuration is duplicated per crate.

## Alternatives Considered

- **Single crate `tepra`** — rejected. Smaller boilerplate but
  couples HTTP server, templating, and domain logic; tests would always
  build the full graph.
- **Four crates (core + backend-client + api + web)** — rejected.
  Extracting the backend client into its own crate adds a fourth manifest
  with no current consumer beyond `tepra-core`. Can be split later if a
  CLI or alternative backend lands.

## History

- 2026-06-27: initial version
