//! HTTP implementation of [`TepraClient`] using `reqwest`.

use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use tracing::{Span, instrument};

#[cfg(feature = "otel")]
use opentelemetry_semantic_conventions::attribute;

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
    /// Parsed `host` or `host:port` from `base_url`.
    server_address: String,
    /// `OTel` metric instruments (no-op when `otel` feature is disabled).
    meters: Arc<Meters>,
}

/// Parse scheme and `host[:port]` from a URL string.
fn parse_base_url(base_url: &str) -> (String, String) {
    reqwest::Url::parse(base_url).map_or_else(
        |_| ("http".to_owned(), base_url.to_owned()),
        |parsed| {
            let scheme = parsed.scheme().to_owned();
            let host = parsed.host_str().unwrap_or("localhost").to_owned();
            let address = match parsed.port() {
                Some(p) => format!("{host}:{p}"),
                None => host,
            };
            (scheme, address)
        },
    )
}

impl ReqwestTepraClient {
    /// Create a new client targeting `base_url` (e.g. `"http://localhost:29108"`).
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let (url_scheme, server_address) = parse_base_url(&base_url);
        Self {
            base_url,
            client: reqwest::Client::new(),
            url_scheme,
            server_address,
            meters: Arc::new(Meters::new()),
        }
    }

    #[instrument(
        name = "HTTP GET",
        skip_all,
        fields(
            http.request.method = "GET",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            http.response.status_code = tracing::field::Empty,
        )
    )]
    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, TepraError> {
        let url = format!("{}{}", self.base_url, path);
        let start = Instant::now();
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "GET");
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "GET");
        match resp.json::<T>().await {
            Ok(v) => Ok(v),
            Err(e) => Err(TepraError::Parse {
                source: anyhow::Error::new(e)
                    .context(format!("deserializing response from GET {url}")),
            }),
        }
    }

    #[instrument(
        name = "HTTP GET",
        skip_all,
        fields(
            http.request.method = "GET",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            http.response.status_code = tracing::field::Empty,
        )
    )]
    async fn get_query_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &str,
    ) -> Result<T, TepraError> {
        let url = format!("{}{}?{}", self.base_url, path, query);
        let start = Instant::now();
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "GET");
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "GET");
        match resp.json::<T>().await {
            Ok(v) => Ok(v),
            Err(e) => Err(TepraError::Parse {
                source: anyhow::Error::new(e)
                    .context(format!("deserializing response from GET {url}")),
            }),
        }
    }

    #[instrument(
        name = "HTTP GET",
        skip_all,
        fields(
            http.request.method = "GET",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            http.response.status_code = tracing::field::Empty,
        )
    )]
    async fn get_query_empty(&self, path: &str, query: &str) -> Result<(), TepraError> {
        let url = format!("{}{}?{}", self.base_url, path, query);
        let start = Instant::now();
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                self.record_response_span(status, start.elapsed().as_secs_f64(), "GET");
                Ok(())
            }
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "GET");
                Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                })
            }
        }
    }

    #[instrument(
        name = "HTTP POST",
        skip_all,
        fields(
            http.request.method = "POST",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            http.response.status_code = tracing::field::Empty,
        )
    )]
    async fn post_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, TepraError> {
        let url = format!("{}{}", self.base_url, path);
        let start = Instant::now();
        let resp = match self.client.post(&url).json(body).send().await {
            Ok(r) => r,
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "POST");
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("POST {url}")),
                });
            }
        };
        let status = resp.status();
        self.record_response_span(status, start.elapsed().as_secs_f64(), "POST");
        match resp.json::<T>().await {
            Ok(v) => Ok(v),
            Err(e) => Err(TepraError::Parse {
                source: anyhow::Error::new(e)
                    .context(format!("deserializing response from POST {url}")),
            }),
        }
    }

    #[instrument(
        name = "HTTP POST",
        skip_all,
        fields(
            http.request.method = "POST",
            server.address = %self.server_address,
            url.scheme = %self.url_scheme,
            http.response.status_code = tracing::field::Empty,
        )
    )]
    async fn post_empty<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), TepraError> {
        let url = format!("{}{}", self.base_url, path);
        let start = Instant::now();
        match self.client.post(&url).json(body).send().await {
            Ok(resp) => {
                let status = resp.status();
                self.record_response_span(status, start.elapsed().as_secs_f64(), "POST");
                Ok(())
            }
            Err(e) => {
                self.record_transport_error(start.elapsed().as_secs_f64(), "POST");
                Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("POST {url}")),
                })
            }
        }
    }

    /// Record the response status code on the current span and the duration in metrics.
    fn record_response_span(&self, status: reqwest::StatusCode, duration_s: f64, method: &str) {
        #[cfg(feature = "otel")]
        Span::current().record(
            attribute::HTTP_RESPONSE_STATUS_CODE,
            i64::from(status.as_u16()),
        );
        #[cfg(not(feature = "otel"))]
        Span::current().record("http.response.status_code", i64::from(status.as_u16()));

        self.meters.record_http_request(
            duration_s,
            method,
            Some(status.as_u16()),
            &self.server_address,
            &self.url_scheme,
        );
    }

    /// Record a transport-level error (no HTTP response received) in metrics.
    fn record_transport_error(&self, duration_s: f64, method: &str) {
        self.meters.record_http_request(
            duration_s,
            method,
            None,
            &self.server_address,
            &self.url_scheme,
        );
    }
}

#[async_trait]
impl TepraClient for ReqwestTepraClient {
    async fn list_printers(&self) -> Result<Vec<PrinterListItem>, TepraError> {
        self.get_json("/api/printer").await
    }

    async fn version(&self) -> Result<VersionResponse, TepraError> {
        self.get_json("/api/printer/version").await
    }

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
