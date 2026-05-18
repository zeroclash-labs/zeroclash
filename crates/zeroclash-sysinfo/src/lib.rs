use std::fmt::{Debug, Display};
use std::time::Instant;

#[cfg(windows)]
use deelevate::{PrivilegeLevel, Token};
use parking_lot::RwLock;
use sysinfo::{Networks, System};

pub struct SysInfo {
    pub system_name: String,
    pub system_version: String,
    pub system_kernel_version: String,
    pub system_arch: String,
}

impl Default for SysInfo {
    #[inline]
    fn default() -> Self {
        let system_name = System::name().unwrap_or_else(|| "Null".into());
        let system_version = System::long_os_version().unwrap_or_else(|| "Null".into());
        let system_kernel_version = System::kernel_version().unwrap_or_else(|| "Null".into());
        let system_arch = System::cpu_arch();
        Self {
            system_name,
            system_version,
            system_kernel_version,
            system_arch,
        }
    }
}

pub struct AppInfo {
    pub app_version: String,
    pub app_core_mode: String,
    pub app_startup_time: Instant,
    pub app_is_admin: bool,
}

impl Default for AppInfo {
    #[inline]
    fn default() -> Self {
        Self {
            app_version: "0.0.0".into(),
            app_core_mode: "NotRunning".into(),
            app_is_admin: false,
            app_startup_time: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct Platform {
    pub sysinfo: SysInfo,
    pub appinfo: AppInfo,
}

impl Debug for Platform {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Platform")
            .field("system_name", &self.sysinfo.system_name)
            .field("system_version", &self.sysinfo.system_version)
            .field("system_kernel_version", &self.sysinfo.system_kernel_version)
            .field("system_arch", &self.sysinfo.system_arch)
            .field("app_version", &self.appinfo.app_version)
            .field("app_core_mode", &self.appinfo.app_core_mode)
            .field("app_is_admin", &self.appinfo.app_is_admin)
            .finish()
    }
}

impl Display for Platform {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "System Name: {}\nSystem Version: {}\nSystem kernel Version: {}\nSystem Arch: {}\nVerge Version: {}\nRunning Mode: {}\nIs Admin: {}",
            self.sysinfo.system_name,
            self.sysinfo.system_version,
            self.sysinfo.system_kernel_version,
            self.sysinfo.system_arch,
            self.appinfo.app_version,
            self.appinfo.app_core_mode,
            self.appinfo.app_is_admin
        )
    }
}

impl Platform {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_app_info(version: impl Into<String>) -> Self {
        let is_admin = is_binary_admin();
        let mut platform = Self::new();
        platform.appinfo.app_version = version.into();
        platform.appinfo.app_is_admin = is_admin;
        platform
    }
}

#[inline]
pub fn is_binary_admin() -> bool {
    #[cfg(not(windows))]
    unsafe {
        libc::geteuid() == 0
    }
    #[cfg(windows)]
    Token::with_current_process()
        .and_then(|token| token.privilege_level())
        .map(|level| level != PrivilegeLevel::NotPrivileged)
        .unwrap_or(false)
}

#[inline]
#[cfg(unix)]
pub fn current_gid() -> u32 {
    unsafe { libc::getgid() }
}

#[inline]
pub fn list_network_interfaces() -> Vec<String> {
    let mut networks = Networks::new();
    networks.refresh(false);
    networks.keys().map(|name| name.to_owned()).collect()
}

/// Thread-safe platform state holder, usable without Tauri.
#[derive(Default)]
pub struct PlatformState {
    inner: RwLock<Platform>,
}

impl PlatformState {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Platform::new()),
        }
    }

    #[inline]
    pub fn new_with_app_info(version: impl Into<String>) -> Self {
        Self {
            inner: RwLock::new(Platform::with_app_info(version)),
        }
    }

    #[inline]
    pub fn set_core_mode(&self, mode: impl Into<String>) {
        self.inner.write().appinfo.app_core_mode = mode.into();
    }

    #[inline]
    pub fn uptime(&self) -> Instant {
        self.inner.read().appinfo.app_startup_time
    }

    #[inline]
    pub fn is_admin(&self) -> bool {
        self.inner.read().appinfo.app_is_admin
    }

    #[inline]
    pub fn platform_snapshot(&self) -> String {
        self.inner.read().to_string()
    }
}

impl Debug for PlatformState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlatformState")
            .field("platform", &*self.inner.read())
            .finish()
    }
}
