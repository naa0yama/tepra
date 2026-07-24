# Askama Templates

HTML templates live in `crates/tepra/templates/` and are compiled at build time
by the [Askama](https://djc.github.io/askama/) template engine.

## Directory Structure

```
crates/tepra/templates/
  shells/
    dashboard.html      # L1 shell (base layout)
  pages/
    index.html          # Printer list page (GET /ui/)
    printer_detail.html # Per-printer detail page (GET /ui/printers/{name})
    api.html            # API Reference page (GET /ui/api)
  partials/
    job_card.html       # HTMX job-status polling card (GET /ui/jobs/{printer}/{id})
    try_it_out.html     # Per-endpoint "Try it out" form macro (used by api.html)
  components/
    alert.html          # Reusable alert macros
    sidebar.html        # Drawer sidebar nav (logo + section menu)
    breadcrumbs.html    # Navbar breadcrumb trail
    theme_toggle.html   # Navbar corporate/business theme swap control
```

## Template Roles

### shells/dashboard.html

Base layout used by all page templates via `{% extends %}`:

- Loads `/static/app.css` (Tailwind 4 + DaisyUI 5 bundle served by `tepra-web`)
- Loads `/static/htmx.min.js` (deferred, no CDN)
- Favicon: `<link rel="icon">` → `/favicon.svg` (shared printer-mark icon with the sidebar logo)
- DaisyUI theme: `data-theme="corporate"` default, swapped to `business` at runtime
  by the theme toggle (see `components/theme_toggle.html` below); persisted in
  `localStorage`, not server-side
  - Head inline `<script>` (before the stylesheet `<link>`) reads `localStorage`
    and sets `data-theme` before paint, to avoid a flash of the wrong theme (FOUC)
  - Body-end inline `<script>` listens for `.theme-controller` `change` and writes
    the selection back to `localStorage`
- Accessibility: skip-to-content link, `<main id="main" tabindex="-1">`
- Navbar: hamburger (mobile only), breadcrumb trail (`components/breadcrumbs.html`),
  and theme toggle (`components/theme_toggle.html`)
- Responsive drawer nav — sidebar (`components/sidebar.html`) in `drawer-side`,
  collapses to hamburger on mobile
- Toast container: `#toast-container` (DaisyUI toast, `aria-live="polite"`)
- Exposes `{% block title %}` and `{% block body %}` blocks

### pages/index.html

Extends `shells/dashboard.html`. Bound to `IndexTemplate` in `views.rs`.

- Shows a DaisyUI menu list of known printer names
- Renders `components::error_alert` when `error: Option<String>` is set
- Empty-state hero when `printers` is empty

### pages/printer_detail.html

Extends `shells/dashboard.html`. Bound to `PrinterDetailTemplate` in `views.rs`.

- Shows per-printer metadata and job history
- Each job is rendered as a `job_card.html` partial via HTMX out-of-band swap

### pages/api.html

Extends `shells/dashboard.html`. Bound to `ApiDocsTemplate` in `views.rs`.

- Swagger-UI-like reference for the built-in `/api/*` HTTP API, rendered from
  the code-derived `openapi.json` (view-model built in-process by
  `build_endpoint_views`, not fetched client-side)
- One DaisyUI accordion (`collapse`) per endpoint, showing `method` + `path` +
  `summary` and the request/response JSON schemas (`request_schema_json` /
  `response_schema_json`) plus a request `sample_json`
- Embeds the `try_it_out` macro per endpoint for live execution against the
  running server's own `/api/*` routes
- Destructive-endpoint confirm gate (inline `<script>`, IIFE-scoped):
  - Endpoints whose path contains a `DESTRUCTIVE_PATH_MARKERS` segment
    (`/print/`, `/tapefeed/`, `/job/control/`) render with a
    `data-destructive-form` marker and must pass through a `<dialog>` confirm
    modal before firing
  - A **capturing-phase** `submit` listener on `document.body` (capture=true)
    intercepts every native submit (including single-field Enter-key submit),
    `preventDefault` + `stopPropagation`, and opens the modal — this closes the
    click-only-gate bypass where Enter would skip a `type="button"` Execute
  - A `destructiveConfirmed` flag authorizes exactly one pass-through after the
    user confirms; it is force-cleared immediately after `requestSubmit()` so a
    constraint-validation failure (which skips submit-event dispatch) cannot
    leave the gate stuck open
  - Non-destructive forms pass the guard untouched and execute directly

### partials/try_it_out.html

Macro file: `{% macro try_it_out(endpoint, index) %}`. Imported by
`pages/api.html`; not a standalone page.

- Builds one execution form per endpoint from an `EndpointView`
- `path_params` (extracted from `{...}` path segments) render as required text
  inputs; endpoints with a request body get a JSON `<textarea>` prefilled with
  `sample_json`
- Non-destructive forms submit via HTMX (`hx-{method}`, or `data-json-body-form`
  for body-carrying POSTs) with a `type="submit"` Execute button
- Destructive forms carry `data-destructive-form` and use a `type="button"`
  Execute (`data-destructive-trigger`) so the confirm gate in `api.html`
  mediates every execution

### partials/job_card.html

Standalone partial, not extending any shell. Bound to `JobCardTemplate`.

- `<div id="job-{job_id}">` — HTMX target for OOB swaps
- Polls `GET /ui/jobs/{printer}/{job_id}` every 1 s while job is in-flight
- Stops polling when `job_end=true` or `canceled=true` (removes `hx-trigger`)
- States: waiting (no progress), in-progress (percent), completed, cancelled

### components/alert.html

Macro file (no `{% extends %}`):

```jinja
{% macro error_alert(message) %} … {% endmacro %}
```

Import with `{% import "components/alert.html" as components %}`.

### components/sidebar.html

Macro file: `{% macro sidebar(active) %}`.

- Renders the `drawer-side` content: a clickable logo link (`<a href="/ui/">`,
  printer-mark icon + "TEPRA Creator") followed by a separate DaisyUI `menu`
  list
- Menu items: Printers (linked), Jobs / Templates (`menu-disabled`, no `href`,
  "Coming soon" badge), API (linked, `href="/ui/api"`, between Templates and
  Settings), Settings (`menu-disabled`) — unimplemented sections never 404
- `active` (from `nav_active`) marks the current item with `menu-active` +
  `aria-current="page"`

### components/breadcrumbs.html

Macro file: `{% macro breadcrumbs(items) %}`, `items` is a `Vec<Breadcrumb>`
(`views::Breadcrumb`).

- Renders a DaisyUI `breadcrumbs` list in the navbar
- Entries with `href` render as links; the entry without `href` (current page)
  renders as plain text — e.g. `Printers > KING JIM SR-R7900-NW`

### components/theme_toggle.html

Macro file: `{% macro theme_toggle() %}`.

- DaisyUI "Theme Controller using a swap" pattern: a `swap swap-rotate`
  checkbox (class `theme-controller`, value `business`) with sun/moon SVG icons
- Toggles between the `corporate` (unchecked) and `business` (checked) themes;
  persistence is wired by the inline scripts in `shells/dashboard.html`, not by
  this component

## Rust Bindings (`crates/tepra/src/views.rs`)

| Struct                  | Template path               |
| ----------------------- | --------------------------- |
| `IndexTemplate`         | `pages/index.html`          |
| `PrinterDetailTemplate` | `pages/printer_detail.html` |
| `JobCardTemplate`       | `partials/job_card.html`    |
| `ApiDocsTemplate`       | `pages/api.html`            |

All implement `askama::Template` and are wrapped in `HtmlTemplate<T>` for
axum `IntoResponse` compatibility.

`IndexTemplate`, `PrinterDetailTemplate`, and `ApiDocsTemplate` all carry
`nav_active: String` (sidebar active section, `components/sidebar.html`) and
`breadcrumbs: Vec<Breadcrumb>` (navbar trail, `components/breadcrumbs.html`).
`nav_active` is set from named constants (`views::NAV_PRINTERS` /
`views::NAV_API`) rather than literals, so the handlers that build it cannot
drift out of sync with each other. `ApiDocsTemplate` additionally carries
`endpoints: Vec<EndpointView>` (see `try_it_out.html` above) and
`error: Option<String>`.
`Breadcrumb` is a plain data carrier (not an `askama::Template`):

```rust
pub struct Breadcrumb {
    pub label: String,
    pub href: Option<String>,
}
```

Each handler builds its own trail — `index` yields a single non-linked
`"Printers"` entry, `printer_detail` yields `Printers` (linked to `/ui/`)
followed by the printer name.

## Related

- `docs/specs/architecture/pwa-asset-pipeline.md` — how CSS/JS assets are built and served
- `docs/adr/latest/0003-server-rendered-ui-with-askama-and-htmx.md`
- `docs/adr/latest/0007-ui-testing-strategy.md`
