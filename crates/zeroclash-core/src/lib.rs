pub mod backup;
pub mod config;
pub mod connection;
pub mod constants;
pub mod core_installer;
pub mod enhance;
pub mod i18n;
#[cfg(target_os = "macos")]
mod macos_proxy;
pub mod media_unlock;
pub mod mihomo;
pub mod paths;
pub mod profile;
pub mod service;
pub mod sys;

pub use backup::BackupManager;
pub use config::Config;
pub use connection::{ConnEntry, ConnectionStore, SharedConnStore, spawn_connection_stream};
pub use media_unlock::{UnlockResult, UnlockStatus, check_all};
pub use mihomo::{CoreManager, CoreSource, MihomoClient, resolve_core_path};
pub use profile::{IProfiles, PrfItem, PrfOption, ProfileStore};
pub use service::ServiceManager;
pub use sys::{AutoStart, SystemProxy, acquire_singleton, notify};
