//! Service management — install/run mihomo as a Windows service or Linux systemd unit.

use anyhow::Result;
use std::path::PathBuf;

/// Platform abstraction for service management.
pub struct ServiceManager {
    #[allow(dead_code)]
    core_path: PathBuf,
    #[allow(dead_code)]
    service_name: String,
}

impl ServiceManager {
    pub fn new(core_path: PathBuf) -> Self {
        Self {
            core_path,
            service_name: "zeroclash-core".into(),
        }
    }

    /// Install the core as a system service.
    pub fn install(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.install_windows()?;
        }
        #[cfg(target_os = "linux")]
        {
            self.install_systemd()?;
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            log::warn!("Service management only supported on Windows and Linux");
        }
        Ok(())
    }

    /// Uninstall the core service.
    #[allow(clippy::missing_const_for_fn)]
    pub fn uninstall(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.uninstall_windows()?;
        }
        #[cfg(target_os = "linux")]
        {
            self.uninstall_systemd()?;
        }
        Ok(())
    }

    /// Check if service is installed.
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_installed(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            self.check_windows_installed()
        }
        #[cfg(target_os = "linux")]
        {
            self.check_systemd_installed()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        false
    }

    #[cfg(target_os = "windows")]
    fn install_windows(&self) -> Result<()> {
        let core = self.core_path.display().to_string();
        let status = std::process::Command::new("sc")
            .args([
                "create",
                &self.service_name,
                "binPath=",
                &core,
                "start=",
                "auto",
                "DisplayName=",
                "ZeroClash Core Service",
            ])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to create Windows service (may need admin)");
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn uninstall_windows(&self) -> Result<()> {
        let status = std::process::Command::new("sc")
            .args(["delete", &self.service_name])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to delete Windows service (may need admin)");
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn check_windows_installed(&self) -> bool {
        std::process::Command::new("sc")
            .args(["query", &self.service_name])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    fn install_systemd(&self) -> Result<()> {
        let unit = format!(
            r"[Unit]
Description=ZeroClash Core Service
After=network.target

[Service]
Type=simple
ExecStart={core}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
",
            core = self.core_path.display()
        );

        let unit_path = format!("/etc/systemd/system/{}.service", self.service_name);
        std::fs::write(&unit_path, unit)?;

        let _ = std::process::Command::new("systemctl")
            .args(["daemon-reload"])
            .status()?;

        let _ = std::process::Command::new("systemctl")
            .args(["enable", &self.service_name])
            .status()?;

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[cfg(target_os = "linux")]
    fn uninstall_systemd(&self) -> Result<()> {
        let _ = std::process::Command::new("systemctl")
            .args(["stop", &self.service_name])
            .status();

        let _ = std::process::Command::new("systemctl")
            .args(["disable", &self.service_name])
            .status();

        let unit_path = format!("/etc/systemd/system/{}.service", self.service_name);
        let _ = std::fs::remove_file(unit_path);

        let _ = std::process::Command::new("systemctl")
            .args(["daemon-reload"])
            .status();

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn check_systemd_installed(&self) -> bool {
        let unit_path = format!("/etc/systemd/system/{}.service", self.service_name);
        std::path::Path::new(&unit_path).exists()
    }
}
