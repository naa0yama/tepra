//! HTTP implementation of [`TepraClient`] using `reqwest`.

use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use tracing::{Span, instrument};

#[cfg(feature = "otel")]
use opentelemetry_semantic_conventions::attribute;

/// Request headers recorded as span attributes (allowlist).
const REQUEST_HEADER_ALLOW: &[&str] = &[
    "content-type",
    "content-length",
    "accept",
    "x-request-id",
    "idempotency-key",
];

/// Response headers recorded as span attributes (allowlist).
const RESPONSE_HEADER_ALLOW: &[&str] = &["content-type", "content-length", "x-request-id"];

/// Returns `true` if `name` is in the given `allow` list.
fn header_allowed(name: &str, allow: &[&str]) -> bool {
    allow.contains(&name)
}

/// Record allowed request headers as span attributes on the current span.
fn record_request_headers(headers: &reqwest::header::HeaderMap) {
    #[cfg(feature = "otel")]
    {
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;
        let span = Span::current();
        for (name, value) in headers {
            let name_str = name.as_str();
            if !header_allowed(name_str, REQUEST_HEADER_ALLOW) {
                continue;
            }
            if let Ok(v) = value.to_str() {
                span.set_attribute(format!("http.request.header.{name_str}"), v.to_owned());
            }
        }
    }
    #[cfg(not(feature = "otel"))]
    let _ = headers;
}

/// Record allowed response headers as span attributes on the current span.
fn record_response_headers(headers: &reqwest::header::HeaderMap) {
    #[cfg(feature = "otel")]
    {
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;
        let span = Span::current();
        for (name, value) in headers {
            let name_str = name.as_str();
            if !header_allowed(name_str, RESPONSE_HEADER_ALLOW) {
                continue;
            }
            if let Ok(v) = value.to_str() {
                span.set_attribute(format!("http.response.header.{name_str}"), v.to_owned());
            }
        }
    }
    #[cfg(not(feature = "otel"))]
    let _ = headers;
}

use crate::{
    client::traits::TepraClient,
    dto::{
        job::{
            JobControlRequest, JobInfoResponse, JobProgressResponse, PrintRequest, PrintResponse,
        },
        printer::{
            AutoselectResponse, LwStatusResponse, OnlineStatusResponse, PrinterInfoResponse,
            PrinterListItem, VersionResponse,
        },
        template::{GetMarginRequest, GetMarginResponse, ImportFrameItem, ImportFrameRequest},
    },
    error::TepraError,
    otel::metrics::Meters,
};

/// HTTP client for the KING JIM TEPRA Creator `WebAPI`.
///
/// Constructed with [`ReqwestTepraClient::new`]; inject `base_url` to point at
/// the Creator daemon (default `http://localhost:29108`) or a `WireMock` server
/// in tests.
#[derive(Debug)]
pub struct ReqwestTepraClient {
    base_url: String,
    client: reqwest::Client,
    /// Parsed scheme (`"http"` or `"https"`) from `base_url`.
    url_scheme: String,
    /// Parsed host (without port) from `base_url`.
    server_address: String,
    /// Parsed port from `base_url`, if explicit.
    server_port: Option<u16>,
    /// `OTel` metric instruments (no-op when `otel` feature is disabled).
    meters: Arc<Meters>,
}

/// Parse scheme, host, and optional port from a URL string.
fn parse_base_url(base_url: &str) -> (String, String, Option<u16>) {
    reqwest::Url::parse(base_url).map_or_else(
        |_| ("http".to_owned(), base_url.to_owned(), None),
        |parsed| {
            let scheme = parsed.scheme().to_owned();
            let host = parsed.host_str().unwrap_or("localhost").to_owned();
            let port = parsed.port();
            (scheme, host, port)
        },
    )
}

impl ReqwestTepraClient {
    /// Create a new client targeting `base_url` with a shared [`Meters`] instance.
    ///
    /// Prefer this over [`Self::new`]: inject the same `Arc<Meters>` that is
    /// passed to the HTTP server middleware so all instruments share one provider.
    pub fn with_meters(base_url: impl Into<String>, meters: Arc<Meters>) -> Self {
        let base_url = base_url.into();
        let (url_scheme, server_address, server_port) = parse_base_url(&base_url);
        Self {
            base_url,
            client: reqwest::Client::new(),
            url_scheme,
            server_address,
            server_port,
            meters,
        }
    }

    /// Create a new client targeting `base_url` (e.g. `"http://localhost:29108"`).
    ///
    /// # Deprecation
    ///
    /// Creates an isolated [`Meters`] instance. Use [`Self::with_meters`] and
    /// share a single `Arc<Meters>` across client and server middleware instead.
    #[deprecated(
        since = "0.1.0",
        note = "use `ReqwestTepraClient::with_meters` instead"
    )]
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let (url_scheme, server_address, server_port) = parse_base_url(&base_url);
        Self {
            base_url,
            client: reqwest::Client::new(),
            url_scheme,
            server_address,
            server_port,
            meters: Arc::new(Meters::new()),
        }
    }

    #[instrument(
        name = "GET",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "GET",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            url.full = tracing::field::Empty,
            http.response.status_code = tracing::field::Empty,
            http.response.body.size = tracing::field::Empty,
        )
    )]
    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, TepraError> {
        let url = format!("{}{}", self.base_url, path);
        Span::current().record(attribute::URL_FULL, url.as_str());

        let req = self
            .client
            .get(&url)
            .build()
            .map_err(|e| TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("building GET {url}")),
            })?;
        record_request_headers(req.headers());

        let start = Instant::now();
        let resp = match self.client.execute(req).await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "GET", &e);
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "GET");
        record_response_headers(resp.headers());

        let resp_bytes = resp.bytes().await.map_err(|e| TepraError::Transport {
            source: anyhow::Error::new(e).context(format!("reading response body from GET {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_RESPONSE_BODY_SIZE,
            i64::try_from(resp_bytes.len()).unwrap_or(i64::MAX),
        );
        if !status.is_success() {
            tracing::warn!(http.response.body = %String::from_utf8_lossy(&resp_bytes));
            return Err(TepraError::Http {
                status: status.as_u16(),
            });
        }
        tracing::debug!(http.response.body = %String::from_utf8_lossy(&resp_bytes));

        serde_json::from_slice::<T>(&resp_bytes).map_err(|e| TepraError::Parse {
            source: anyhow::Error::new(e).context(format!("deserializing response from GET {url}")),
        })
    }

    #[instrument(
        name = "GET",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "GET",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            url.full = tracing::field::Empty,
            http.response.status_code = tracing::field::Empty,
            http.response.body.size = tracing::field::Empty,
        )
    )]
    async fn get_query_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &str,
    ) -> Result<T, TepraError> {
        let url = format!("{}{}?{}", self.base_url, path, query);
        Span::current().record(attribute::URL_FULL, url.as_str());

        let req = self
            .client
            .get(&url)
            .build()
            .map_err(|e| TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("building GET {url}")),
            })?;
        record_request_headers(req.headers());

        let start = Instant::now();
        let resp = match self.client.execute(req).await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "GET", &e);
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "GET");
        record_response_headers(resp.headers());

        let resp_bytes = resp.bytes().await.map_err(|e| TepraError::Transport {
            source: anyhow::Error::new(e).context(format!("reading response body from GET {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_RESPONSE_BODY_SIZE,
            i64::try_from(resp_bytes.len()).unwrap_or(i64::MAX),
        );
        if !status.is_success() {
            tracing::warn!(http.response.body = %String::from_utf8_lossy(&resp_bytes));
            return Err(TepraError::Http {
                status: status.as_u16(),
            });
        }
        tracing::debug!(http.response.body = %String::from_utf8_lossy(&resp_bytes));

        serde_json::from_slice::<T>(&resp_bytes).map_err(|e| TepraError::Parse {
            source: anyhow::Error::new(e).context(format!("deserializing response from GET {url}")),
        })
    }

    #[instrument(
        name = "GET",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "GET",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            url.full = tracing::field::Empty,
            http.response.status_code = tracing::field::Empty,
            http.response.body.size = tracing::field::Empty,
        )
    )]
    async fn get_query_empty(&self, path: &str, query: &str) -> Result<(), TepraError> {
        let url = format!("{}{}?{}", self.base_url, path, query);
        Span::current().record(attribute::URL_FULL, url.as_str());

        let req = self
            .client
            .get(&url)
            .build()
            .map_err(|e| TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("building GET {url}")),
            })?;
        record_request_headers(req.headers());

        let start = Instant::now();
        let resp = match self.client.execute(req).await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "GET", &e);
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "GET");
        record_response_headers(resp.headers());

        let resp_bytes = resp.bytes().await.map_err(|e| TepraError::Transport {
            source: anyhow::Error::new(e).context(format!("reading response body from GET {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_RESPONSE_BODY_SIZE,
            i64::try_from(resp_bytes.len()).unwrap_or(i64::MAX),
        );
        if !status.is_success() {
            tracing::warn!(http.response.body = %String::from_utf8_lossy(&resp_bytes));
            return Err(TepraError::Http {
                status: status.as_u16(),
            });
        }
        tracing::debug!(http.response.body = %String::from_utf8_lossy(&resp_bytes));

        Ok(())
    }

    #[instrument(
        name = "POST",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "POST",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            url.full = tracing::field::Empty,
            http.request.body.size = tracing::field::Empty,
            http.response.status_code = tracing::field::Empty,
            http.response.body.size = tracing::field::Empty,
        )
    )]
    async fn post_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, TepraError> {
        let url = format!("{}{}", self.base_url, path);
        Span::current().record("url.full", url.as_str());

        let body_bytes = serde_json::to_vec(body).map_err(|e| TepraError::Parse {
            source: anyhow::Error::new(e).context(format!("serializing body for POST {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_REQUEST_BODY_SIZE,
            i64::try_from(body_bytes.len()).unwrap_or(i64::MAX),
        );
        tracing::debug!(http.request.body = %String::from_utf8_lossy(&body_bytes));

        let req = self
            .client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body_bytes)
            .build()
            .map_err(|e| TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("building POST {url}")),
            })?;
        record_request_headers(req.headers());

        let start = Instant::now();
        let resp = match self.client.execute(req).await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "POST", &e);
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("POST {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "POST");
        record_response_headers(resp.headers());

        let resp_bytes = resp.bytes().await.map_err(|e| TepraError::Transport {
            source: anyhow::Error::new(e).context(format!("reading response body from POST {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_RESPONSE_BODY_SIZE,
            i64::try_from(resp_bytes.len()).unwrap_or(i64::MAX),
        );
        if !status.is_success() {
            tracing::warn!(http.response.body = %String::from_utf8_lossy(&resp_bytes));
            return Err(TepraError::Http {
                status: status.as_u16(),
            });
        }
        tracing::debug!(http.response.body = %String::from_utf8_lossy(&resp_bytes));

        serde_json::from_slice::<T>(&resp_bytes).map_err(|e| TepraError::Parse {
            source: anyhow::Error::new(e)
                .context(format!("deserializing response from POST {url}")),
        })
    }

    #[instrument(
        name = "POST",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "POST",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            url.full = tracing::field::Empty,
            http.request.body.size = tracing::field::Empty,
            http.response.status_code = tracing::field::Empty,
            http.response.body.size = tracing::field::Empty,
        )
    )]
    async fn post_empty<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), TepraError> {
        let url = format!("{}{}", self.base_url, path);
        Span::current().record("url.full", url.as_str());

        let body_bytes = serde_json::to_vec(body).map_err(|e| TepraError::Parse {
            source: anyhow::Error::new(e).context(format!("serializing body for POST {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_REQUEST_BODY_SIZE,
            i64::try_from(body_bytes.len()).unwrap_or(i64::MAX),
        );
        tracing::debug!(http.request.body = %String::from_utf8_lossy(&body_bytes));

        let req = self
            .client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body_bytes)
            .build()
            .map_err(|e| TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("building POST {url}")),
            })?;
        record_request_headers(req.headers());

        let start = Instant::now();
        let resp = match self.client.execute(req).await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "POST", &e);
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("POST {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "POST");
        record_response_headers(resp.headers());

        let resp_bytes = resp.bytes().await.map_err(|e| TepraError::Transport {
            source: anyhow::Error::new(e).context(format!("reading response body from POST {url}")),
        })?;
        Span::current().record(
            attribute::HTTP_RESPONSE_BODY_SIZE,
            i64::try_from(resp_bytes.len()).unwrap_or(i64::MAX),
        );
        if !status.is_success() {
            tracing::warn!(http.response.body = %String::from_utf8_lossy(&resp_bytes));
            return Err(TepraError::Http {
                status: status.as_u16(),
            });
        }
        tracing::debug!(http.response.body = %String::from_utf8_lossy(&resp_bytes));

        Ok(())
    }

    /// Record the response status code on the current span and the duration in metrics.
    fn record_response_span(&self, status: reqwest::StatusCode, duration_s: f64, method: &str) {
        let status_u16 = status.as_u16();
        let error_type_owned = if status.is_client_error() || status.is_server_error() {
            Some(status_u16.to_string())
        } else {
            None
        };

        #[cfg(feature = "otel")]
        {
            use opentelemetry::trace::Status;
            use tracing_opentelemetry::OpenTelemetrySpanExt as _;
            let span = Span::current();
            span.record(attribute::HTTP_RESPONSE_STATUS_CODE, i64::from(status_u16));
            if let Some(ref code_str) = error_type_owned {
                span.set_attribute(attribute::ERROR_TYPE, code_str.clone());
                span.set_status(Status::Error {
                    description: std::borrow::Cow::Owned(code_str.clone()),
                });
            }
        }
        #[cfg(not(feature = "otel"))]
        Span::current().record("http.response.status_code", i64::from(status_u16));

        self.meters.record_http_request(
            duration_s,
            method,
            Some(status_u16),
            &self.server_address,
            self.server_port,
            &self.url_scheme,
            error_type_owned.as_deref(),
        );
    }

    /// Record a transport-level error (no HTTP response received) in span + metrics.
    fn record_transport_error(&self, duration_s: f64, method: &str, err: &reqwest::Error) {
        let kind = classify_transport_error(err);

        #[cfg(feature = "otel")]
        {
            use opentelemetry::trace::Status;
            use tracing_opentelemetry::OpenTelemetrySpanExt as _;
            let span = Span::current();
            span.set_attribute(attribute::ERROR_TYPE, kind);
            span.set_status(Status::Error {
                description: std::borrow::Cow::Borrowed(kind),
            });
        }

        self.meters.record_http_request(
            duration_s,
            method,
            None,
            &self.server_address,
            self.server_port,
            &self.url_scheme,
            Some(kind),
        );
    }
}

/// Classify a `reqwest::Error` into an `error.type` semconv named value.
///
/// Priority: semconv named > `"_OTHER"` fallback.
fn classify_transport_error(err: &reqwest::Error) -> &'static str {
    if err.is_timeout() {
        "timeout"
    } else if err.is_connect() {
        "connection"
    } else if err.is_request() {
        "request_build"
    } else {
        "_OTHER"
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl ReqwestTepraClient {
    /// Test-only constructor that injects a custom `reqwest::Client` (e.g. with a short timeout).
    #[doc(hidden)]
    #[allow(clippy::expect_used)] // infallible: Client::builder() with only timeout() always succeeds
    pub fn new_with_timeout_for_test(
        base_url: impl Into<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let base_url = base_url.into();
        let (url_scheme, server_address, server_port) = parse_base_url(&base_url);
        Self {
            base_url,
            client: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .expect("reqwest::Client build must succeed"),
            url_scheme,
            server_address,
            server_port,
            meters: std::sync::Arc::new(crate::otel::metrics::Meters::new()),
        }
    }
}

#[async_trait]
impl TepraClient for ReqwestTepraClient {
    #[instrument(
        name = "GET /api/printer",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "GET",
            url.template = "/api/printer",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
        )
    )]
    async fn list_printers(&self) -> Result<Vec<PrinterListItem>, TepraError> {
        self.get_json("/api/printer").await
    }

    #[instrument(
        name = "GET /api/printer/version",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "GET",
            url.template = "/api/printer/version",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
        )
    )]
    async fn version(&self) -> Result<VersionResponse, TepraError> {
        self.get_json("/api/printer/version").await
    }

    #[instrument(
        name = "GET /api/printer/autoselect",
        skip_all,
        fields(
            otel.kind = "CLIENT",
            http.request.method = "GET",
            url.template = "/api/printer/autoselect",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
        )
    )]
    async fn autoselect(&self) -> Result<AutoselectResponse, TepraError> {
        self.get_json("/api/printer/autoselect").await
    }

    async fn printer_info(&self, name: &str) -> Result<PrinterInfoResponse, TepraError> {
        self.get_json(&format!("/api/printer/info/{name}")).await
    }

    async fn online_status(&self, name: &str) -> Result<OnlineStatusResponse, TepraError> {
        self.get_json(&format!("/api/printer/onlinestatus/{name}"))
            .await
    }

    async fn lw_status(&self, name: &str) -> Result<LwStatusResponse, TepraError> {
        self.get_json(&format!("/api/printer/lwstatus/{name}"))
            .await
    }

    async fn print(&self, name: &str, req: PrintRequest) -> Result<PrintResponse, TepraError> {
        self.post_json(&format!("/api/printer/print/{name}"), &req)
            .await
    }

    async fn tapefeed(&self, name: &str, cutflag: bool) -> Result<(), TepraError> {
        self.get_query_empty(
            &format!("/api/printer/tapefeed/{name}"),
            &format!("cutflag={cutflag}"),
        )
        .await
    }

    async fn job_progress(
        &self,
        name: &str,
        jobid: u64,
    ) -> Result<JobProgressResponse, TepraError> {
        self.get_query_json(
            &format!("/api/printer/job/progress/{name}"),
            &format!("jobid={jobid}"),
        )
        .await
    }

    async fn job_info(&self, name: &str, jobid: u64) -> Result<JobInfoResponse, TepraError> {
        self.get_query_json(
            &format!("/api/printer/job/info/{name}"),
            &format!("jobid={jobid}"),
        )
        .await
    }

    async fn job_control(&self, name: &str, req: JobControlRequest) -> Result<(), TepraError> {
        self.post_empty(&format!("/api/printer/job/control/{name}"), &req)
            .await
    }

    async fn import_frame(
        &self,
        req: ImportFrameRequest,
    ) -> Result<Vec<ImportFrameItem>, TepraError> {
        self.post_json("/api/printer/template/importframe", &req)
            .await
    }

    async fn get_margin(
        &self,
        name: &str,
        req: GetMarginRequest,
    ) -> Result<GetMarginResponse, TepraError> {
        self.post_json(&format!("/api/printer/getmargin/{name}"), &req)
            .await
    }
}
