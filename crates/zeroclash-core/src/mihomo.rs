//! Mihomo (Clash Meta) core IPC client.
//!
//! Communicates with a running mihomo instance via its REST API
//! over Unix domain sockets (macOS/Linux) or named pipes (Windows).

use anyhow::{Context, Result};
use serde_json::Value;

/// Client for communicating with the mihomo core's REST API.
///
/// Connects via the mihomo external controller address
/// (default: `127.0.0.1:9090`).
pub struct MihomoClient {
    /// Base URL for the mihomo REST API (e.g., "http://127.0.0.1:9090")
    pub base_url: String,
    client: reqwest::Client,
}

impl MihomoClient {
    /// Create a new MihomoClient connected to the given address.
    pub fn new(addr: impl Into<String>) -> Self {
        let base_url = addr.into();
        let client = reqwest::Client::new();
        Self { base_url, client }
    }

    /// Create a client using the default external controller address.
    pub fn default_addr() -> Self {
        Self::new("http://127.0.0.1:9090")
    }

    /// GET a mihomo API endpoint and return parsed JSON.
    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        let body = resp
            .text()
            .await
            .with_context(|| format!("read body from {url}"))?;
        serde_json::from_str(&body).with_context(|| format!("parse JSON from {url}"))
    }

    /// Get the mihomo core version.
    pub async fn version(&self) -> Result<String> {
        let v = self.get("/version").await?;
        let version = v
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(version.to_string())
    }

    /// Get the current clash configuration.
    pub async fn configs(&self) -> Result<Value> {
        self.get("/configs").await
    }

    /// Get all proxy information (groups and nodes).
    pub async fn proxies(&self) -> Result<Value> {
        self.get("/proxies").await
    }

    /// Get all rules.
    pub async fn rules(&self) -> Result<Value> {
        self.get("/rules").await
    }

    /// Get active connections.
    pub async fn connections(&self) -> Result<Value> {
        self.get("/connections").await
    }

    /// Get traffic statistics.
    pub async fn traffic(&self) -> Result<Value> {
        self.get("/traffic").await
    }
}
