use compact_str::CompactString;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use flexi_logger::DeferredNow;
use flexi_logger::filter::LogLineFilter;
use flexi_logger::writers::FileLogWriter;
use flexi_logger::writers::LogWriter as _;
use log::Level;
use log::Record;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

pub type SharedWriter = Arc<Mutex<FileLogWriter>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Cmd,
    Core,
    Config,
    Setup,
    System,
    SystemSignal,
    Service,
    Hotkey,
    Window,
    Tray,
    Timer,
    Frontend,
    Backup,
    File,
    Lightweight,
    Network,
    ProxyMode,
    Validate,
    ZeroClash,
}

impl fmt::Display for Type {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cmd => write!(f, "[Cmd]"),
            Self::Core => write!(f, "[Core]"),
            Self::Config => write!(f, "[Config]"),
            Self::Setup => write!(f, "[Setup]"),
            Self::System => write!(f, "[System]"),
            Self::SystemSignal => write!(f, "[SysSignal]"),
            Self::Service => write!(f, "[Service]"),
            Self::Hotkey => write!(f, "[Hotkey]"),
            Self::Window => write!(f, "[Window]"),
            Self::Tray => write!(f, "[Tray]"),
            Self::Timer => write!(f, "[Timer]"),
            Self::Frontend => write!(f, "[Frontend]"),
            Self::Backup => write!(f, "[Backup]"),
            Self::File => write!(f, "[File]"),
            Self::Lightweight => write!(f, "[Lightweight]"),
            Self::Network => write!(f, "[Network]"),
            Self::ProxyMode => write!(f, "[ProxMode]"),
            Self::Validate => write!(f, "[Validate]"),
            Self::ZeroClash => write!(f, "[ZeroClash]"),
        }
    }
}

#[macro_export]
macro_rules! logging {
    ($level:ident, $type:expr, $($arg:tt)*) => {
        log::$level!(target: "app", "{} {}", $type, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! logging_error {
    ($type:expr, $expr:expr) => {
        if let Err(err) = $expr {
            log::error!(target: "app", "[{}] {}", $type, err);
        }
    };
    ($type:expr, $fmt:literal $(, $arg:expr)*) => {
        log::error!(target: "app", "[{}] {}", $type, format_args!($fmt $(, $arg)*));
    };
}

#[inline]
pub fn write_sidecar_log(
    writer: MutexGuard<'_, FileLogWriter>,
    now: &mut DeferredNow,
    level: Level,
    message: &CompactString,
) {
    let args = format_args!("{}", message);
    let record = Record::builder()
        .args(args)
        .level(level)
        .target("sidecar")
        .build();
    let _ = writer.write(now, &record);
}

pub struct NoModuleFilter<'a>(pub Vec<&'a str>);

impl<'a> NoModuleFilter<'a> {
    #[inline]
    pub fn filter(&self, record: &Record) -> bool {
        if let Some(module) = record.module_path() {
            for blocked in self.0.iter() {
                if module.len() >= blocked.len()
                    && module.as_bytes()[..blocked.len()] == blocked.as_bytes()[..]
                {
                    return false;
                }
            }
        }
        true
    }
}

impl LogLineFilter for NoModuleFilter<'_> {
    #[inline]
    fn write(
        &self,
        now: &mut DeferredNow,
        record: &Record,
        writer: &dyn flexi_logger::filter::LogLineWriter,
    ) -> std::io::Result<()> {
        if !self.filter(record) {
            return Ok(());
        }
        writer.write(now, record)
    }
}

/// Initialize the file-based log backend.
///
/// After this call, all `log` crate macros (`log::info!`, `log::warn!`, etc.)
/// write to rotating daily log files under `log_dir`. Log files are kept for
/// 7 days.
///
/// # Errors
///
/// Returns an error if the log directory cannot be created or the logger
/// cannot be initialized (typically because one is already set).
pub fn init_logger(log_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(log_dir)?;

    flexi_logger::Logger::try_with_str("info")?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(log_dir)
                .basename("zeroclash"),
        )
        .rotate(
            flexi_logger::Criterion::Age(flexi_logger::Age::Day),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(7),
        )
        .start()?;

    Ok(())
}
