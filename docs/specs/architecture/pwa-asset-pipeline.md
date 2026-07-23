# PWA Asset Pipeline

`tepra-web` uses a build-time asset pipeline to bundle CSS, fonts, and HTMX
without any runtime CDN dependency.

## Crate Layout

The pipeline lives entirely in the binary crate `crates/tepra-web/`:

```
crates/tepra-web/
  build.rs           # Cargo build script — runs pnpm + Tailwind at compile time
  package.json       # pnpm workspace root (private, devDependencies only)
  pnpm-workspace.yaml
  pnpm-lock.yaml     # committed; frozen at build time
  src/
    assets.rs        # embed + serve static assets via axum routes
    static/
      favicon.svg    # favicon (printer-mark icon), served at GET /favicon.svg
    styles/
      app.css        # Tailwind 4 + DaisyUI 5 entry point
```

The library crate `crates/tepra/` remains pure Rust (no Node.js dependency).

## Build Script (`build.rs`)

Steps executed by Cargo before compilation:

1. **pnpm check** — fails fast with a hint if `pnpm` is not on `PATH`.
2. **`pnpm install --frozen-lockfile`** — installs devDependencies from lockfile.
3. **Tailwind compile** — `pnpm exec tailwindcss -i src/styles/app.css -o $OUT_DIR/app.css --minify`.
4. **Font copy** — `@fontsource/ibm-plex-sans-jp` and `@fontsource/ibm-plex-mono` woff2
   files are copied to `$OUT_DIR/fonts/<family>/`.
5. **HTMX copy** — `node_modules/htmx.org/dist/htmx.min.js` is copied to
   `$OUT_DIR/htmx.min.js`.
6. **`rerun-if-changed`** — `walkdir` walks `../tepra/templates/` so that template
   edits trigger a Tailwind rebuild.

## Frontend Dependencies (`package.json`)

| Package                        | Role                             |
| ------------------------------ | -------------------------------- |
| `tailwindcss`                  | CSS framework (v4)               |
| `@tailwindcss/cli`             | Tailwind CLI used in build.rs    |
| `daisyui`                      | Component plugin for Tailwind v5 |
| `@fontsource/ibm-plex-sans-jp` | Japanese + Latin woff2 files     |
| `@fontsource/ibm-plex-mono`    | Monospace woff2 files            |
| `htmx.org`                     | HTMX JavaScript bundle           |

## CSS Entry Point (`src/styles/app.css`)

- `@import "tailwindcss"` — Tailwind base
- `@plugin "daisyui"` — themes: `corporate` (light, default), `business` (dark)
- `@source` directives point at `../../../tepra/templates/**/*.html` and
  `../../../tepra/src/**/*.rs` for class scanning across crates
- `@font-face` rules reference `/fonts/…` served by `assets::router()`

## Static Asset Routes (`src/assets.rs`)

`assets::router()` is merged into the main axum `Router` in `main.rs`:

| Route                     | Content             | Cache policy      |
| ------------------------- | ------------------- | ----------------- |
| `GET /static/app.css`     | Compiled CSS bundle | none (dev)        |
| `GET /static/htmx.min.js` | HTMX bundle         | none (dev)        |
| `GET /favicon.svg`        | Favicon (SVG)       | none (dev)        |
| `GET /fonts/{*path}`      | woff2 font files    | immutable, 1 year |

Assets are embedded at compile time via `include_bytes!` (`APP_CSS`, `HTMX_JS`,
`FAVICON_SVG`) and `rust_embed::RustEmbed` (`Fonts`).

## Multi-Crate Consideration

`@source` paths in `app.css` are relative to `crates/tepra-web/src/styles/`:

```css
/* tepra (library crate) templates */
@source "../../../tepra/templates/**/*.html";
@source "../../../tepra/src/**/*.rs";
/* tepra-web (binary crate) sources */
@source "../../src/**/*.rs";
```

In a single-crate layout `@source "../templates/**/*.html"` would suffice,
but the multi-crate workspace requires explicit cross-crate paths.

## Related

- `crates/tepra-web/build.rs` — build script implementation
- `crates/tepra-web/src/assets.rs` — asset embedding + axum routes
- `docs/specs/components/askama-templates.md` — template directory structure
- `docs/adr/latest/0003-server-rendered-ui-with-askama-and-htmx.md`
