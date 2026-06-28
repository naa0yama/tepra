# 0007. UI testing strategy

- Status: Accepted
- Date: 2026-06-28
- Deciders: project owner

## Context

The project renders HTML with Askama + HTMX. Three layers need verification:

1. **HTTP / HTML structure** — routes return correct status codes and the
   rendered templates contain expected elements.
2. **HTMX behaviour** — polling, swap targets, and partial updates work as
   intended in a real browser.
3. **Visual regression** — DaisyUI styling and layout do not break across
   changes.

Constraints:

- Primary runtime is Rust / cargo; adding Node.js permanently to the local
  developer loop increases toolchain complexity.
- `fantoccini` / Selenium-style tools require test code that mirrors
  implementation code — two parallel codebases to maintain for the same UI.
- Claude Code (claude-code CLI) should be able to verify UI behaviour during
  implementation without the developer writing additional test files.

## Decision

Three-tier testing strategy:

| Layer            | Tool                               | When                        |
| ---------------- | ---------------------------------- | --------------------------- |
| HTTP / HTML      | `axum-test`                        | `cargo test` — always       |
| Dev-time UI      | Playwright MCP (`@playwright/mcp`) | Claude-driven, no test code |
| CI visual / HTMX | Playwright (Node.js via mise)      | PR gate — CI only           |

**axum-test** covers route correctness and template structure regression. It
runs entirely in Rust with no external process.

**Playwright MCP** is configured as a Claude Code MCP server. During
implementation, Claude navigates to `http://localhost:PORT` (started via
`mise run dev:up`), takes screenshots, and verifies HTMX interactions without
any test file being written. Node.js is required for the MCP server process
but is managed by `mise use node@lts` and does not affect the Rust build.

**Playwright CI** runs the same browser scenarios as gated checks on PRs that
touch UI files. The Tailwind-compiled `static/app.css` is committed, so CI
needs only `npx playwright test` — no build step.

## Consequences

Positive:

- Rust unit/integration tests remain the primary quality gate; no Node.js in
  the normal `cargo test` path.
- Claude can verify UI during implementation without the developer maintaining
  separate test files (Playwright MCP replaces fantoccini / Selenium scripts).
- CI catches HTMX and visual regressions that axum-test cannot detect.
- Node.js is isolated to `mise` and CI — never a system-level dependency.

Negative:

- Playwright MCP requires a running dev server (`mise run dev:up`) before
  Claude can verify UI.
- Two separate test runners (`cargo test` + `npx playwright`) must stay in
  sync when routes change.
- Initial Playwright CI setup (fixtures, selectors) needs a one-time
  investment when UI implementation begins.

## Alternatives Considered

- **fantoccini / thirtyfour (Rust WebDriver)** — rejected. Requires writing
  CSS-selector-based test code that mirrors the implementation, creating a
  dual maintenance burden. Does not enable Claude to verify UI autonomously.
- **Playwright for everything (no axum-test)** — rejected. Browser startup
  overhead makes it unsuitable as a `cargo test` replacement; HTTP-level
  regressions are cheapest to catch in pure Rust.
- **No visual / HTMX testing** — deferred, not rejected. Playwright CI will be
  added when the first non-trivial HTMX interaction is implemented.

## History

- 2026-06-28: initial version
