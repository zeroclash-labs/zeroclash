//! System integration utilities: auto-start, singleton instance, system proxy, notifications.

use anyhow::{Context as _, Result};
use std::path::PathBuf;

// ── Auto‑start ─────────────────────────────────────────────────────────────

/// Configure auto‑start on system boot.
pub struct AutoStart {
    app_name: String,
    app_path: PathBuf,
}

impl AutoStart {
    pub fn new(app_name: &str, app_path: PathBuf) -> Self {
        Self {
            app_name: app_name.to_string(),
            app_path,
        }
    }

    /// Enable auto‑start.
    pub fn enable(&self) -> Result<()> {
        let auto = auto_launch::AutoLaunchBuilder::new()
            .set_app_name(&self.app_name)
            .set_app_path(&self.app_path.to_string_lossy())
            .set_use_launch_agent(true)
            .build()
            .context("build auto-launch")?;

        if auto.is_enabled().unwrap_or(false) {
            return Ok(());
        }
        auto.enable().context("enable auto-launch")?;
        Ok(())
    }

    /// Disable auto‑start.
    pub fn disable(&self) -> Result<()> {
        let auto = auto_launch::AutoLaunchBuilder::new()
            .set_app_name(&self.app_name)
            .set_app_path(&self.app_path.to_string_lossy())
            .set_use_launch_agent(true)
            .build()
            .context("build auto-launch")?;

        if auto.is_enabled().unwrap_or(false) {
            auto.disable().context("disable auto-launch")?;
        }
        Ok(())
    }

    /// Check if auto‑start is enabled.
    pub fn is_enabled(&self) -> bool {
        auto_launch::AutoLaunchBuilder::new()
            .set_app_name(&self.app_name)
            .set_app_path(&self.app_path.to_string_lossy())
            .set_use_launch_agent(true)
            .build()
            .map(|a| a.is_enabled().unwrap_or(false))
            .unwrap_or(false)
    }
}

// ── Singleton instance ────────────────────────────────────────────────────

/// Acquire a singleton lock. Returns Ok(true) if this is the first instance,
/// Ok(false) if another instance is already running.
pub fn acquire_singleton(app_name: &str) -> Result<bool> {
    let lock_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(app_name);
    std::fs::create_dir_all(&lock_dir).ok();

    let lock_path = lock_dir.join("singleton.lock");

    match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&lock_path)
    {
        Ok(_) => Ok(true), // We are the first instance
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(anyhow::anyhow!("singleton lock error: {e}")),
    }
}

/// Release the singleton lock.
pub fn release_singleton(app_name: &str) {
    let lock_path = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(app_name)
        .join("singleton.lock");
    let _ = std::fs::remove_file(lock_path);
}

// ── System proxy (macOS only via sysproxy-rs) ──────────────────────────────

/// Platform-specific system proxy operations.
pub struct SystemProxy;

impl SystemProxy {
    /// Enable system HTTP/HTTPS/SOCKS proxy pointing to localhost:port.
    pub fn enable(http_port: u16, _socks_port: u16) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let proxy = sysproxy::Sysproxy {
                enable: true,
                host: "127.0.0.1".into(),
                port: http_port,
                bypass: "localhost,127.0.0.1,::1".into(),
            };
            proxy.set_system_proxy()?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (http_port, _socks_port);
            log::warn!("System proxy is only supported on macOS");
        }
        Ok(())
    }

    /// Disable system proxy.
    pub fn disable() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let proxy = sysproxy::Sysproxy {
                enable: false,
                ..Default::default()
            };
            proxy.set_system_proxy()?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::warn!("System proxy is only supported on macOS");
        }
        Ok(())
    }
}

// ── Desktop notifications ──────────────────────────────────────────────────

/// Send a desktop notification.
pub fn notify(title: &str, body: &str) {
    let result = notify_rust::Notification::new()
        .appname("ZeroClash")
        .summary(title)
        .body(body)
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show();

    if let Err(e) = result {
        log::warn!("Failed to send notification: {e}");
    }
}
