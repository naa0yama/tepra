//! HTTP implementation of [`TepraClient`] using `reqwest`.

use async_trait::async_trait;

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
}

impl ReqwestTepraClient {
    /// Create a new client targeting `base_url` (e.g. `"http://localhost:29108"`).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, TepraError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        match resp.json::<T>().await {
            Ok(v) => Ok(v),
            Err(e) => Err(TepraError::Parse {
                source: anyhow::Error::new(e)
                    .context(format!("deserializing response from GET {url}")),
            }),
        }
    }

    async fn get_query_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &str,
    ) -> Result<T, TepraError> {
        let url = format!("{}{}?{}", self.base_url, path, query);
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("GET {url}")),
                });
            }
        };
        match resp.json::<T>().await {
            Ok(v) => Ok(v),
            Err(e) => Err(TepraError::Parse {
                source: anyhow::Error::new(e)
                    .context(format!("deserializing response from GET {url}")),
            }),
        }
    }

    async fn get_query_empty(&self, path: &str, query: &str) -> Result<(), TepraError> {
        let url = format!("{}{}?{}", self.base_url, path, query);
        match self.client.get(&url).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("GET {url}")),
            }),
        }
    }

    async fn post_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, TepraError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = match self.client.post(&url).json(body).send().await {
            Ok(r) => r,
            Err(e) => {
                return Err(TepraError::Transport {
                    source: anyhow::Error::new(e).context(format!("POST {url}")),
                });
            }
        };
        match resp.json::<T>().await {
            Ok(v) => Ok(v),
            Err(e) => Err(TepraError::Parse {
                source: anyhow::Error::new(e)
                    .context(format!("deserializing response from POST {url}")),
            }),
        }
    }

    async fn post_empty<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), TepraError> {
        let url = format!("{}{}", self.base_url, path);
        match self.client.post(&url).json(body).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(TepraError::Transport {
                source: anyhow::Error::new(e).context(format!("POST {url}")),
            }),
        }
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
