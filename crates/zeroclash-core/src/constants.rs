//! Constants shared across the zeroclash workspace.
//! Adapted from clash-verge-rev's `src-tauri/src/constants.rs`.

pub mod network {
    pub const DEFAULT_EXTERNAL_CONTROLLER: &str = "127.0.0.1:9097";

    pub const DEFAULT_MIXED: u16 = 7897;
    pub const DEFAULT_SOCKS: u16 = 7898;
    pub const DEFAULT_HTTP: u16 = 7899;
}

pub mod files {
    pub const RUNTIME_CONFIG: &str = "clash-verge.yaml";
    pub const DNS_CONFIG: &str = "dns_config.yaml";
}

#[allow(dead_code)]
pub mod tun {
    pub const DEFAULT_STACK: &str = "gvisor";
    pub const DNS_HIJACK: &[&str] = &["any:53"];
}
