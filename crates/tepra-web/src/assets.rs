//! Static asset embedding and serving routes.

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};

/// Compiled and minified CSS bundle (built by build.rs via Tailwind 4 + `DaisyUI` 5).
pub static APP_CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.css"));

/// HTMX JavaScript bundle (copied from `node_modules` by build.rs).
pub static HTMX_JS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/htmx.min.js"));

/// Favicon SVG (printer-mark icon, shared with the sidebar logo).
pub static FAVICON_SVG: &[u8] = include_bytes!("static/favicon.svg");

/// Embedded font files (woff2, copied from @fontsource packages by build.rs).
#[derive(rust_embed::RustEmbed)]
#[folder = "$OUT_DIR/fonts"]
pub struct Fonts;

impl std::fmt::Debug for Fonts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fonts").finish()
    }
}

/// Build the static-asset router.
pub fn router() -> Router {
    Router::new()
        .route("/static/app.css", get(serve_css))
        .route("/static/htmx.min.js", get(serve_htmx))
        .route("/favicon.svg", get(serve_favicon))
        .route("/fonts/{*path}", get(serve_font))
}

async fn serve_css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], APP_CSS).into_response()
}

async fn serve_htmx() -> Response {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        HTMX_JS,
    )
        .into_response()
}

async fn serve_favicon() -> Response {
    ([(header::CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG).into_response()
}

async fn serve_font(Path(path): Path<String>) -> Response {
    let Some(file) = Fonts::get(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    (
        [
            (header::CONTENT_TYPE, "font/woff2"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        file.data,
    )
        .into_response()
}
