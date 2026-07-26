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
    print.html          # Print job page (GET /ui/print) with printer panel + polling
    api.html            # API Reference page (GET /ui/api)
  partials/
    job_card.html              # HTMX job-status polling card (GET /ui/jobs/{printer}/{id})
    printer_status_card.html   # HTMX lazy-loaded printer status card (GET /ui/printers/{name}/status-card)
    endpoint_entry.html        # Per-endpoint collapse accordion macro (used by api.html)
    try_it_out.html            # Per-endpoint "Try it out" form macro (used by api.html)
    property_table.html        # Request/response/param property table macro (used by api.html)
    merge_printer_panel.html   # Printer status display (online/offline, tape, margin) for print page (GET /ui/print/{printer}/panel)
  components/
    alert.html          # Reusable alert macros
    sidebar.html        # Drawer sidebar nav (logo + section menu)
    breadcrumbs.html    # Navbar breadcrumb trail
    theme_toggle.html   # Navbar corporate/business theme swap control
    printer_refresh_toggle.html # Navbar auto-refresh toggle for printer status polling
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

- Shows a 2-column responsive grid of printer cards (lazy-loaded via HTMX)
- Each card performs `hx-get="/ui/printers/{printer}/status-card"` on load and swaps into the card body
- Renders `components::error_alert` when `error: Option<String>` is set
- Empty-state hero when `printers` is empty

### pages/print.html

Extends `shells/dashboard.html`. Bound to `PrintTemplate` in `views.rs`.

- Print job submission and status tracking for label printers
- Printer selection (`<select id="printer-info-select">`) with manual refresh button (`#printer-refresh-btn`)
  - Select is in a DaisyUI `join` layout with a refresh button (SVG icon, square button class)
  - Button calls `loadPrinterPanel(currentPrinter)` on click to immediately fetch updated status
- Printer info panel (`#printer-info-panel`, target for `GET /ui/print/{printer}/panel`):
  - Shows online/offline status, tape width/kind, and margin (via `merge_printer_panel.html`)
  - Auto-refresh behavior: when the navbar printer-refresh-toggle (`#printer-refresh-toggle`) is ON,
    the print page polls this endpoint every 5s via JavaScript `setInterval`, updating status in real-time
  - Polling stops when toggle is OFF or printer is unselected (`clearInterval`)
  - Global toggle state is persisted to `localStorage` via the dashboard shell's toggle-persistence script
- Print job progress — fixed slot (`#print-result`), placed between the printer info
  panel and the copies/advanced settings in the right card. Always present from
  first page load, showing a muted placeholder ("印刷ジョブなし") when no job is
  active; job submission swaps only this slot's contents (`result.innerHTML = html`),
  so the page never grows vertically to reveal progress. Replaces the earlier
  approach of an inline `#job-{job_id}` card appended inside the template/frames pane
  - After the swap, `submitPrint()`/`cancelPrint()` call `htmx.process(result)` so the
    freshly-inserted `job_card.html`'s `hx-trigger="every 1s"` polling attribute is
    activated by htmx (raw `innerHTML` assignment does not auto-wire htmx attributes)
  - A `MutationObserver` watches `#print-result` (not `#print-frames-pane`) for
    `childList`/`subtree` mutations and calls `reflectJobState()` to morph the
    Print/Cancel button. Retargeting away from `#print-frames-pane` is deliberate:
    `#print-submit-btn` lives in that pane, so `setPrintMode`/`setCancelMode`'s
    `textContent` writes used to re-trigger the observer there (loop risk); `#print-result`
    is a static slot present since page load and never overlaps the button's subtree
- Print/Cancel button morph (`#print-submit-btn`): on submit success the button
  switches from "Print" (`btn-primary`) to "Cancel" (`btn-error`, `data-mode="cancel"`);
  clicking it while in cancel mode posts to `POST /ui/jobs/{printer}/{job_id}/cancel`
  and swaps the response into `#print-result` the same way as submit. `reflectJobState()`
  reads the terminal job card's `data-terminal`/`data-printer` attributes (see
  `partials/job_card.html` below) to decide whether to morph to cancel mode (job running)
  or back to print mode (job reached a terminal state)
- Tape input: template selection loads `partials/merge_frames.html` via
  `GET /ui/print/frames`, rendering one nested `data-tape-card` per tape (label) inside
  `#tapes-container`. Each card holds one `data-field-title` input per import frame,
  in API column order. `+ Add tape` clones the first card (`data-tape-card`) and clears
  its inputs; `collectRows()` — the DOM contract the submit body depends on — walks
  `#tapes-container [data-tape-card]`, and within each card `[data-field-title]`, building
  the `rows: Vec<Vec<MergeField>>` submit shape (outer = tape, inner = `{title, value}`
  from `dataset.fieldTitle` / `.value`)
- Submit form with template selection, frame settings, and submit button (`#print-submit-btn`)
  - Errors display in a toast (`#toast-container`)
- REST curl sample panel: collapsible section showing a live `curl` command that reflects
  current template and print settings; copy button syncs command to clipboard; updates
  automatically as user changes template or configuration values
- Template frame loading: spinner element displayed while `GET /ui/print/frames` completes;
  remains visible during frame DOM processing
- Advanced settings toggles: two new checkboxes below standard print settings
  - Hide tape-width confirmation (controlled by `display_tape_width` override; show when value = 2)
  - Hide print-setting confirmation (controlled by `display_print_setting` override; show when value = 2)

### pages/api.html

Extends `shells/dashboard.html`. Bound to `ApiDocsTemplate` in `views.rs`.

- Swagger-UI-like reference for the built-in `/api/*` HTTP API, rendered from
  the code-derived `openapi.json` (view-model built in-process by
  `build_endpoint_views`, not fetched client-side)
- Organizes endpoints into two sections:
  - **公式 Creator WebAPI** — official facade endpoints (`/api/printer/*`),
    where `is_custom == false`
  - **プログラム独自 REST** — program-specific helper endpoints (`/api/rest/*`),
    where `is_custom == true`
  - Section grouping is driven by the `is_custom` field in `EndpointView`,
    set by checking if the path starts with `/api/rest/` (no additional
    metadata required in `openapi.json`)
- Each section renders a DaisyUI accordion (`join join-vertical`) with the
  endpoint entries rendered via the `endpoint_entry` macro
- Printer-name dropdown population (inline `<script>`, IIFE-scoped): on load,
  a single client-side `fetch("/api/printer")` fills every
  `[data-printer-select]` `<option>` with the connected printer names, so the
  `{name}` path param is a pick-list, not free text. On fetch failure or an
  empty list it degrades each `<select>` to a plain text input (keeping
  `data-path-param` so the existing path-substitution logic is unchanged).
  The `api_docs` handler stays stateless — the printer list is fetched
  client-side from the already-instrumented `/api/printer` route rather than
  injected server-side (see `try_it_out.html` and ADR 0010 rationale)
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

### partials/endpoint_entry.html

Macro file: `{% macro endpoint_entry(endpoint, index) %}`. Imported by
`pages/api.html`; not a standalone page.

- Renders a single endpoint as a DaisyUI collapse accordion (`join-item`)
  with collapse-arrow
- Collapse title: `method` badge (GET/POST colour-coded, fixed width) +
  `path` code + `summary` + `destructive` badge if applicable
- Collapse content: property tables (Parameters, Request body, Response body) +
  raw JSON schema disclosures (`<details>`, expanded by default) + `try_it_out`
  macro for live execution

### partials/try_it_out.html

Macro file: `{% macro try_it_out(endpoint, index) %}`. Imported by
`endpoint_entry.html` (which is in turn imported by `pages/api.html`); not
a standalone page.

- Builds one execution form per endpoint from an `EndpointView`
- `path_params` (extracted from `{...}` path segments) render as required
  inputs; the `name` param renders as a `<select data-printer-select>` dropdown
  (populated client-side, see `api.html` above), all other params as text
  inputs. Endpoints with a request body get a JSON `<textarea>` prefilled with
  `sample_json`
- `query_params` (the operation's `in == "query"` parameters, e.g. `jobid`,
  `cutflag`) each render as a text input carrying `name` but **not**
  `data-path-param` — on the htmx GET path they are serialized into the query
  string, and the `configRequest` handler in `api.html` strips only
  `data-path-param` inputs, so query inputs survive as `?jobid=N`
- Non-destructive forms submit via HTMX (`hx-{method}`, or `data-json-body-form`
  for body-carrying POSTs) with a `type="submit"` Execute button
- Destructive forms carry `data-destructive-form` and use a `type="button"`
  Execute (`data-destructive-trigger`) so the confirm gate in `api.html`
  mediates every execution

### partials/property_table.html

Macro file: `{% macro property_table(title, name_header, rows) %}`. Imported by
`pages/api.html`; not a standalone page.

- Renders a DaisyUI `table table-sm` of `rows` (each a `ParamView` or
  `PropertyView`), one row per field: name (`<code>`) / type / required
  (Yes/No) / description (em-dash when absent)
- Skips rendering entirely when `rows` is empty, so body-less endpoints and
  param-less operations produce no empty table
- Called once per shape per endpoint (path params, request properties,
  response properties)

### partials/printer_status_card.html

Standalone partial, not extending any shell. Bound to `PrinterStatusCardTemplate`.

- `hx-get` target for `GET /ui/printers/{name}/status-card`
- Shows printer name, online/offline status badge, and current tape width / kind
- Renders error message when the status fetch fails (offline/unreachable printer)
- Replaced the old per-printer detail page; lazy-loads into cards on the
  printer list (`pages/index.html`)

### partials/job_card.html

Standalone partial, not extending any shell. Bound to `JobCardTemplate`.

- `<div id="job-{job_id}">` — HTMX target for OOB swaps
- `data-terminal="true"|"false"` (job_end‖canceled) and `data-printer="{printer_name}"` —
  read by `pages/print.html`'s `reflectJobState()` to morph the Print/Cancel button
  without depending on htmx's self-poll swap event (see `pages/print.html` above)
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
- Menu items (in render order): Printers (linked, `href="/ui/"`),
  Print (linked, `href="/ui/print"`), Jobs (`menu-disabled`, "Coming soon"
  badge), API (linked, `href="/ui/api"`) — Templates and Settings items removed
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

### components/printer_refresh_toggle.html

Macro file: `{% macro printer_refresh_toggle() %}`.

- Navbar auto-refresh control for printer status polling on the print page
- Renders a labeled toggle with refresh SVG icon, "Auto-refresh" text (sm+ screens),
  and `toggle-primary` DaisyUI styling
- Checkbox `id="printer-refresh-toggle"`, `class="toggle"` for JS targeting
- Tooltip (`title="Auto-refresh printer status every 5s"`) explains the 5s interval
- Persistence is wired by inline scripts in `shells/dashboard.html` (localStorage key:
  `PRINTER_AUTOREFRESH_KEY`), similar to theme toggle pattern
- Only has functional effect on `pages/print.html` (controls `setInterval` for
  `GET /ui/print/{printer}/panel` polling); other pages just preserve the setting

## Rust Bindings (`crates/tepra/src/views.rs`)

| Struct                      | Template path                       |
| --------------------------- | ----------------------------------- |
| `IndexTemplate`             | `pages/index.html`                  |
| `PrintPageTemplate`         | `pages/print.html`                  |
| `PrinterStatusCardTemplate` | `partials/printer_status_card.html` |
| `JobCardTemplate`           | `partials/job_card.html`            |
| `ApiDocsTemplate`           | `pages/api.html`                    |

All implement `askama::Template` and are wrapped in `HtmlTemplate<T>` for
axum `IntoResponse` compatibility.

Compile-time constants injected into templates:

- `APP_VERSION` — application version string from `Cargo.toml` version field
- `GIT_HASH` — 7-character git short hash from `build.rs`; displays alongside version
  in sidebar footer as `v{APP_VERSION} ({GIT_HASH})`

`IndexTemplate` and `ApiDocsTemplate` both carry
`nav_active: String` (sidebar active section, `components/sidebar.html`) and
`breadcrumbs: Vec<Breadcrumb>` (navbar trail, `components/breadcrumbs.html`).
`nav_active` is set from named constants (`views::NAV_PRINTERS` /
`views::NAV_API`) rather than literals, so the handlers that build it cannot
drift out of sync with each other. `ApiDocsTemplate` additionally carries
`endpoints: Vec<EndpointView>` (see `try_it_out.html` above) and
`error: Option<String>`.

`EndpointView` carries both the raw schema JSON (`request_schema_json` /
`response_schema_json` / `sample_json`, kept for the `<details>` disclosure)
and structured, pre-extracted view-models for the property tables:
`params: Vec<ParamView>` (path/query parameters, for the property table),
`query_params: Vec<ParamView>` (query-only subset, for the Try-it-out form's
query-string inputs), `request_properties` and
`response_properties: Vec<PropertyView>`. `ParamView` and `PropertyView` are
plain data carriers (`name`, `type_name`, `required: bool`,
`description: Option<String>`). They are built by pure functions
(`extract_params` / `extract_query_params` / `extract_properties`, resolving
`$ref` via `resolve_ref`)
inside `build_endpoint_views`, which keeps the seam unit-testable against a
fixture `openapi.json` (`views.rs` tests).
`EndpointView` also carries `is_custom: bool` — `true` for program-specific
REST helpers (`/api/rest/*`), `false` for the official Creator `WebAPI` facade
(`/api/printer/*`). This field drives the two-section grouping in
`pages/api.html`, determined by checking if the endpoint's `path` starts with
the `CUSTOM_PATH_PREFIX` constant (`"/api/rest/"`).
`Breadcrumb` is a plain data carrier (not an `askama::Template`):

```rust
pub struct Breadcrumb {
    pub label: String,
    pub href: Option<String>,
}
```

Each handler builds its own trail — `index` yields a single non-linked
`"Printers"` entry.

## Related

- `docs/specs/architecture/pwa-asset-pipeline.md` — how CSS/JS assets are built and served
- `docs/adr/latest/0003-server-rendered-ui-with-askama-and-htmx.md`
- `docs/adr/latest/0007-ui-testing-strategy.md`
