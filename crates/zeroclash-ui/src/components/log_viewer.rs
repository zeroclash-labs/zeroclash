use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERR",
            Self::Warn => "WRN",
            Self::Info => "INF",
            Self::Debug => "DBG",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

pub struct LogStore {
    entries: Vec<LogEntry>,
    max_entries: usize,
}

impl Default for LogStore {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000,
        }
    }
}

impl LogStore {
    pub fn push(&mut self, level: LogLevel, module: &str, message: &str) {
        let d = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = d.as_secs() % 86400;
        self.entries.push(LogEntry {
            timestamp: format!(
                "{:02}:{:02}:{:02}",
                secs / 3600,
                (secs % 3600) / 60,
                secs % 60
            ),
            level,
            module: module.to_string(),
            message: message.to_string(),
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub struct LogViewer {
    pub store: LogStore,
    pub filter_level: LogLevel,
    pub search_text: String,
    pub auto_scroll: bool,
}

impl Default for LogViewer {
    fn default() -> Self {
        Self {
            store: LogStore::default(),
            filter_level: LogLevel::Debug,
            search_text: String::new(),
            auto_scroll: true,
        }
    }
}
impl LogViewer {
    pub fn push(&mut self, level: LogLevel, module: &str, message: &str) {
        self.store.push(level, module, message);
    }
}
