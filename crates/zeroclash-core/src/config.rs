//! Configuration types and management for ZeroClash.
//!
//! Uses the `Draft<T>` dual-state system from `zeroclash-draft` for
//! zero-copy reads with lazy copy-on-write editing.

use serde::{Deserialize, Serialize};
use zeroclash_draft::Draft;

/// Application-level configuration (equivalent to Clash Verge Rev's `IVerge`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VergeConfig {
    /// UI language (e.g., "en", "zh", "zh-TW")
    #[serde(default = "default_language")]
    pub language: String,

    /// Theme mode: "light", "dark", or "system"
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,

    /// HTTP proxy port for the clash core
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// SOCKS5 proxy port for the clash core
    #[serde(default = "default_socks_port")]
    pub socks_port: u16,

    /// Mixed (HTTP+SOCKS) proxy port
    #[serde(default = "default_mixed_port")]
    pub mixed_port: u16,

    /// Redir proxy port (Linux only)
    #[serde(default = "default_redir_port")]
    pub redir_port: u16,

    /// TProxy port (Linux only)
    #[serde(default = "default_tproxy_port")]
    pub tproxy_port: u16,

    /// Whether to enable system proxy
    #[serde(default)]
    pub enable_system_proxy: bool,

    /// Whether to enable TUN mode
    #[serde(default)]
    pub enable_tun: bool,

    /// Whether to auto-start the clash core
    #[serde(default = "default_true")]
    pub enable_auto_launch: bool,

    /// Whether to start on system boot
    #[serde(default)]
    pub enable_auto_start: bool,

    /// Clash core binary path (empty = use bundled)
    #[serde(default)]
    pub clash_core_path: String,
}

fn default_language() -> String {
    "zh".into()
}
fn default_theme_mode() -> String {
    "system".into()
}
fn default_http_port() -> u16 {
    7899
}
fn default_socks_port() -> u16 {
    7898
}
fn default_mixed_port() -> u16 {
    7897
}
fn default_redir_port() -> u16 {
    7895
}
fn default_tproxy_port() -> u16 {
    7896
}
fn default_true() -> bool {
    true
}

impl Default for VergeConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            theme_mode: default_theme_mode(),
            http_port: default_http_port(),
            socks_port: default_socks_port(),
            mixed_port: default_mixed_port(),
            redir_port: default_redir_port(),
            tproxy_port: default_tproxy_port(),
            enable_system_proxy: false,
            enable_tun: false,
            enable_auto_launch: true,
            enable_auto_start: false,
            clash_core_path: String::new(),
        }
    }
}

/// The global configuration state.
///
/// Manages all configuration types as `Draft<T>` for concurrent access.
pub struct Config {
    /// Application / verge settings
    pub verge: Draft<VergeConfig>,
}

impl Config {
    /// Create a new Config with defaults.
    pub fn new() -> Self {
        Self {
            verge: Draft::new(VergeConfig::default()),
        }
    }

    /// Create a Config from an existing VergeConfig.
    pub fn from_verge(verge: VergeConfig) -> Self {
        Self {
            verge: Draft::new(verge),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
