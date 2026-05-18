//! Real-time connection monitoring via mihomo WebSocket stream.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A snapshot of all active connections, updated via WebSocket.
#[derive(Debug, Clone, Default)]
pub struct ConnectionStore {
    connections: HashMap<String, ConnEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnEntry {
    pub id: String,
    pub host: String,
    pub network: String,
    pub conn_type: String,
    pub source_ip: String,
    pub destination_ip: String,
    pub source_port: String,
    pub destination_port: String,
    pub chains: Vec<String>,
    pub rule: String,
    pub rule_payload: String,
    pub upload: u64,
    pub download: u64,
    pub start: String,
    pub dns_mode: String,
    #[serde(default)]
    pub speed_up: u64,
    #[serde(default)]
    pub speed_down: u64,
}

impl ConnectionStore {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Apply a WebSocket message (JSON object or array of objects).
    pub fn apply_message(&mut self, msg: &str) -> Result<usize> {
        let v: Value = serde_json::from_str(msg).context("parse connection message")?;

        match v {
            Value::Object(map) => {
                self.apply_object(&map);
                Ok(self.connections.len())
            }
            Value::Array(arr) => {
                for obj in arr {
                    if let Value::Object(map) = obj {
                        self.apply_object(&map);
                    }
                }
                Ok(self.connections.len())
            }
            _ => Ok(self.connections.len()),
        }
    }

    fn apply_object(&mut self, map: &serde_json::Map<String, Value>) {
        let id = map
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        if id.is_empty() || id == "unknown" {
            return;
        }

        // Handle removal (download == 0 && upload == 0 on certain messages can mean close)
        // The mihomo WebSocket sends full snapshot with metadata on download/upload for closures
        let metadata = map.get("metadata");

        let entry = ConnEntry {
            id: id.to_string(),
            host: metadata
                .and_then(|m| m.get("host"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            network: metadata
                .and_then(|m| m.get("network"))
                .and_then(|v| v.as_str())
                .unwrap_or("tcp")
                .to_string(),
            conn_type: metadata
                .and_then(|m| m.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            source_ip: metadata
                .and_then(|m| m.get("sourceIP"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            destination_ip: metadata
                .and_then(|m| m.get("destinationIP"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            source_port: metadata
                .and_then(|m| m.get("sourcePort"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            destination_port: metadata
                .and_then(|m| m.get("destinationPort"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            dns_mode: metadata
                .and_then(|m| m.get("dnsMode"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            chains: map
                .get("chains")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            rule: map
                .get("rule")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            rule_payload: map
                .get("rulePayload")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            upload: map.get("upload").and_then(|v| v.as_u64()).unwrap_or(0),
            download: map.get("download").and_then(|v| v.as_u64()).unwrap_or(0),
            start: map
                .get("start")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            speed_up: 0,
            speed_down: 0,
        };

        self.connections.insert(entry.id.clone(), entry);
    }

    /// Get all connections sorted by upload+download (descending by default).
    pub fn all_sorted(&self) -> Vec<&ConnEntry> {
        let mut entries: Vec<&ConnEntry> = self.connections.values().collect();
        entries.sort_by(|a, b| {
            (b.upload + b.download).cmp(&(a.upload + a.download))
        });
        entries
    }

    /// Get total count.
    pub fn len(&self) -> usize {
        self.connections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

/// Shared connection store for cross-thread updates.
pub type SharedConnStore = Arc<Mutex<ConnectionStore>>;

/// Spawn a tokio task that connects to the mihomo WebSocket and updates the store.
pub fn spawn_connection_stream(
    base_url: &str,
    store: SharedConnStore,
) -> tokio::task::JoinHandle<()> {
    let url = format!(
        "ws{}://{}/connections",
        if base_url.starts_with("https") { "s" } else { "" },
        base_url
            .trim_start_matches("http://")
            .trim_start_matches("https://")
    );

    tokio::spawn(async move {
        loop {
            match connect_and_stream(&url, &store).await {
                Ok(()) => log::info!("Connection stream ended normally"),
                Err(e) => log::warn!("Connection stream error: {e}, retrying..."),
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    })
}

async fn connect_and_stream(url: &str, store: &SharedConnStore) -> Result<()> {
    use futures_util::StreamExt;
    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .context("connect to connection WebSocket")?;

    let (_, read) = ws_stream.split();
    let mut read = read;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                let mut guard = store.lock().await;
                if let Err(e) = guard.apply_message(&text) {
                    log::warn!("Failed to parse connection message: {e}");
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
            Err(e) => {
                log::warn!("WebSocket error: {e}");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
