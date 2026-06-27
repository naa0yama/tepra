# 0003. Server-rendered UI with Askama and HTMX

- Status: Accepted
- Date: 2026-06-27
- Deciders: project owner

## Context

The project needs a browser-facing UI for printer dashboard, template
library, and print flow. Reasonable candidates:

- TypeScript SPA (React / Vue / Svelte) consuming the REST API as JSON.
- Server-rendered HTML from Rust (Askama / Tera / Maud) with HTMX for
  partial updates.
- Hybrid (server-rendered shell + lightweight client islands).

Constraints:

- Single-developer project initially; minimizing toolchain count is
  valuable.
- The team already maintains skills for `rust-pwa-stack` (Askama + HTMX
  - DaisyUI), so the learning curve is amortized.
- Deployment target is LAN-attached PCs; SEO is irrelevant, but fast
  initial paint and zero npm runtime are nice properties.
- PWA / Service Worker is explicitly out of scope for MVP2.

## Decision

Render the UI server-side with Askama compile-time templates, drive
dynamic updates with HTMX (`hx-get`, `hx-post`, `hx-trigger="every Ns"`
polling), style with Tailwind v4 + DaisyUI compiled by the standalone
Tailwind CLI to a checked-in `static/app.css`. No client-side JavaScript
beyond HTMX and its required browser glue.

The Tailwind output is committed so production builds never need Node.
Page routes (`/`, `/templates`, `/print`) live in `tepra-web`; HTMX
fragments are returned by the same `tepra-web` handlers (not by the
`/api/v1` REST router).

## Consequences

Positive:

- Single-language toolchain (Rust + cargo + mise) for build and CI.
- Compile-time template safety; missing variables fail `cargo check`.
- Fast initial render, minimal JS shipped.
- Reuses existing `rust-pwa-stack` patterns and skills.

Negative:

- Complex interactive widgets (e.g. drag-and-drop template editor) are
  awkward in HTMX; would require ad-hoc JS or a future refactor to a
  framework.
- Mobile UX is acceptable but inferior to a native-feeling SPA / PWA.
- Form-state-heavy flows need careful HTMX swap targeting.

## Alternatives Considered

- **TypeScript SPA (React / Vite)** — rejected. Doubles the toolchain
  (npm + cargo), splits CI, and gains little for an internal LAN tool
  with no SEO and a small team.
- **Tera or Maud instead of Askama** — rejected. Tera is runtime-checked
  (loses compile-time safety); Maud is more verbose for moderately
  complex layouts. Askama best matches existing skills.
- **Pure server-rendered + full-page navigation, no HTMX** — rejected.
  Job progress and printer status need live polling; full reloads are
  visibly jarring.

## History

- 2026-06-27: initial version
