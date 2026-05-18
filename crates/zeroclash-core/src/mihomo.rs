//! Mihomo (Clash Meta) core IPC client and lifecycle manager.
//!
//! Communicates with a running mihomo instance via its REST API and manages the
//! sidecar process lifecycle (start, stop, restart).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::constants::network::DEFAULT_EXTERNAL_CONTROLLER;

// ── Data types ──────────────────────────────────────────────────────────────

/// A single proxy (node) in a proxy group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    #[serde(default)]
    pub udp: bool,
    #[serde(default)]
    pub history: Vec<DelayHistory>,
    #[serde(default)]
    pub now: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayHistory {
    pub time: String,
    pub delay: u64,
}

/// A proxy group (selector, url-test, fallback, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub history: Vec<DelayHistory>,
}

/// Active connection info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub metadata: ConnectionMetadata,
    pub upload: u64,
    pub download: u64,
    pub start: String,
    pub chains: Vec<String>,
    pub rule: String,
    #[serde(rename = "rulePayload")]
    pub rule_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub network: String,
    #[serde(rename = "type")]
    pub conn_type: String,
    #[serde(rename = "sourceIP")]
    pub source_ip: String,
    #[serde(rename = "destinationIP")]
    pub destination_ip: String,
    #[serde(rename = "sourcePort")]
    pub source_port: String,
    #[serde(rename = "destinationPort")]
    pub destination_port: String,
    pub host: String,
    #[serde(rename = "dnsMode")]
    pub dns_mode: String,
}

/// Real-time traffic stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Traffic {
    pub up: u64,
    pub down: u64,
}

/// Core running mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreMode {
    NotRunning,
    Running(String), // version
}

// ── Mihomo REST client ─────────────────────────────────────────────────────

/// Client for communicating with the mihomo core's REST API.
pub struct MihomoClient {
    pub base_url: String,
    client: reqwest::Client,
}

impl MihomoClient {
    /// Create a client using the default external controller address.
    pub fn default_addr() -> Self {
        Self::new(format!("http://{DEFAULT_EXTERNAL_CONTROLLER}"))
    }

    /// Create a new MihomoClient connected to the given address.
    pub fn new(addr: impl Into<String>) -> Self {
        let base_url = addr.into();
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    // ── Low-level HTTP helpers ────────────────────────────────────────────

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{path}", self.base_url);
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

    async fn put(&self, path: &str, body: &Value) -> Result<()> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .put(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("PUT {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("PUT {url} returned {}", resp.status());
        }
        Ok(())
    }

    // ── Core info ─────────────────────────────────────────────────────────

    /// Get the mihomo core version.
    pub async fn version(&self) -> Result<String> {
        let v = self.get("/version").await?;
        Ok(v.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string())
    }

    /// Get the current clash configuration.
    pub async fn configs(&self) -> Result<Value> {
        self.get("/configs").await
    }

    // ── Proxies & groups ──────────────────────────────────────────────────

    /// Get all proxies and groups.
    pub async fn proxies(&self) -> Result<Value> {
        self.get("/proxies").await
    }

    /// Get a specific proxy or group by name.
    pub async fn proxy(&self, name: &str) -> Result<Value> {
        self.get(&format!("/proxies/{name}")).await
    }

    /// Test delay for a specific proxy.
    pub async fn proxy_delay(&self, name: &str, timeout_ms: u32, url: &str) -> Result<u64> {
        let path = format!("/proxies/{name}/delay");
        let query = format!("timeout={timeout_ms}&url={url}");
        let full_url = format!("{}{path}?{query}", self.base_url);
        let resp = self
            .client
            .get(&full_url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .with_context(|| format!("GET {full_url}"))?;
        let body = resp.text().await?;
        let v: Value = serde_json::from_str(&body)?;
        Ok(v.get("delay")
            .and_then(|d| d.as_u64())
            .unwrap_or(0))
    }

    /// Select a proxy in a selector group.
    pub async fn select_proxy(&self, group: &str, proxy: &str) -> Result<()> {
        let body = serde_json::json!({ "name": proxy });
        self.put(&format!("/proxies/{group}"), &body).await
    }

    // ── Rules ─────────────────────────────────────────────────────────────

    /// Get all rules.
    pub async fn rules(&self) -> Result<Value> {
        self.get("/rules").await
    }

    // ── Connections ───────────────────────────────────────────────────────

    /// Get active connections.
    pub async fn connections(&self) -> Result<Value> {
        self.get("/connections").await
    }

    /// Close a specific connection by ID.
    pub async fn close_connection(&self, id: &str) -> Result<()> {
        let url = format!("{}/connections/{id}", self.base_url);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .with_context(|| format!("DELETE {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("DELETE {url} returned {}", resp.status());
        }
        Ok(())
    }

    /// Close all connections.
    pub async fn close_all_connections(&self) -> Result<()> {
        let url = format!("{}/connections", self.base_url);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .with_context(|| format!("DELETE {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("DELETE {url} returned {}", resp.status());
        }
        Ok(())
    }

    // ── Traffic ───────────────────────────────────────────────────────────

    /// Get traffic statistics.
    pub async fn traffic(&self) -> Result<Traffic> {
        let v = self.get("/traffic").await?;
        Ok(Traffic {
            up: v.get("up").and_then(|u| u.as_u64()).unwrap_or(0),
            down: v.get("down").and_then(|d| d.as_u64()).unwrap_or(0),
        })
    }

    // ── Config patching ───────────────────────────────────────────────────

    /// Patch the clash config (hot reload). Accepts a JSON object with keys to update.
    pub async fn patch_config(&self, patch: &Value) -> Result<()> {
        self.put("/configs", patch).await
    }

    /// Switch the clash mode (rule / global / direct).
    pub async fn switch_mode(&self, mode: &str) -> Result<()> {
        self.patch_config(&serde_json::json!({ "mode": mode }))
            .await
    }
}

// ── Core lifecycle manager ─────────────────────────────────────────────────

/// Manages the mihomo sidecar process lifecycle.
pub struct CoreManager {
    child: Arc<Mutex<Option<Child>>>,
    core_path: PathBuf,
    controller_addr: String,
}

impl CoreManager {
    /// Create a new CoreManager.
    pub fn new(core_path: PathBuf, controller_addr: Option<String>) -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            core_path,
            controller_addr: controller_addr
                .unwrap_or_else(|| DEFAULT_EXTERNAL_CONTROLLER.to_string()),
        }
    }

    /// Check if the core is currently running.
    pub async fn is_running(&self) -> bool {
        let guard = self.child.lock().await;
        guard.is_some()
    }

    /// Start the mihomo core as a child process.
    pub async fn start(&self) -> Result<()> {
        let mut guard = self.child.lock().await;
        if guard.is_some() {
            anyhow::bail!("Core is already running");
        }

        let child = Command::new(&self.core_path)
            .arg("-d")
            .arg(
                dirs_next::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("zeroclash"),
            )
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("Failed to start core at {:?}", self.core_path))?;

        *guard = Some(child);
        Ok(())
    }

    /// Stop the mihomo core.
    pub async fn stop(&self) -> Result<()> {
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            child
                .kill()
                .await
                .context("Failed to kill core process")?;
        }
        Ok(())
    }

    /// Create a MihomoClient configured for this core's controller address.
    pub fn client(&self) -> MihomoClient {
        MihomoClient::new(format!("http://{}", self.controller_addr))
    }

    /// Get the controller address.
    pub fn controller_addr(&self) -> &str {
        &self.controller_addr
    }
}
