//! Standard platform directory paths for the ZeroClash application.
//!
//! Wraps the [`dirs`] crate so every crate uses the same app-specific
//! directories without ad-hoc path construction.

use std::path::PathBuf;

const APP_NAME: &str = "zeroclash";

fn or_cwd(opt: Option<PathBuf>) -> PathBuf {
    opt.unwrap_or_else(|| PathBuf::from("."))
}

/// `dirs::cache_dir()` → `<cache>/zeroclash`
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .map(|p| p.join(APP_NAME))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `dirs::config_dir()` → `<config>/zeroclash`
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join(APP_NAME))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `dirs::data_dir()` → `<data>/zeroclash`
pub fn data_dir() -> PathBuf {
    or_cwd(dirs::data_dir()).join(APP_NAME)
}

/// `dirs::data_local_dir()` → `<data_local>/zeroclash`
pub fn data_local_dir() -> PathBuf {
    or_cwd(dirs::data_local_dir()).join(APP_NAME)
}

/// Platform-appropriate log directory.
///
/// - **macOS:** `~/Library/Logs/zeroclash`
/// - **Linux:** `<data_local>/zeroclash/logs`
/// - **Windows:** `<data_local>/zeroclash/logs`
pub fn log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        or_cwd(dirs::home_dir()).join("Library/Logs").join(APP_NAME)
    }
    #[cfg(not(target_os = "macos"))]
    {
        data_local_dir().join("logs")
    }
}
