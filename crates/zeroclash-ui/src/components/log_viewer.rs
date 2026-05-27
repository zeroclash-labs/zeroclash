use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Lowercase variants of `module` and `message` precomputed at push
    /// time so the search filter doesn't allocate per-frame.
    pub module_lower: String,
    pub message_lower: String,
}

pub struct LogStore {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
}

impl Default for LogStore {
    fn default() -> Self {
        Self {
            entries: VecDeque::with_capacity(1024),
            max_entries: 1000,
        }
    }
}

impl LogStore {
    pub fn push(&mut self, level: LogLevel, module: &str, message: &str) {
        let d = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs_total = d.as_secs();
        let secs = secs_total % 86400;
        let day = secs_total / 86400;
        // RFC3339-ish, day/seconds-of-day. We avoid pulling chrono just
        // for a timestamp; the day count is enough to disambiguate logs
        // captured across midnight.
        let timestamp = format!(
            "d{day:05} {:02}:{:02}:{:02}",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        );
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry {
            timestamp,
            level,
            module: module.to_string(),
            message: message.to_string(),
            module_lower: module.to_lowercase(),
            message_lower: message.to_lowercase(),
        });
    }
    pub fn entries(&self) -> impl ExactSizeIterator<Item = &LogEntry> {
        self.entries.iter()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub struct LogViewer {
    pub store: LogStore,
    pub filter_level: LogLevel,
    pub search_text: String,
    pub search_text_lower: String,
    pub auto_scroll: bool,
    /// Scroll handle for the log entries container. Updated by
    /// `set_offset` / `scroll_to_bottom` to honour `auto_scroll`.
    pub scroll: gpui::ScrollHandle,
}

impl Default for LogViewer {
    fn default() -> Self {
        Self {
            store: LogStore::default(),
            filter_level: LogLevel::Debug,
            search_text: String::new(),
            search_text_lower: String::new(),
            auto_scroll: true,
            scroll: gpui::ScrollHandle::default(),
        }
    }
}
impl LogViewer {
    pub fn push(&mut self, level: LogLevel, module: &str, message: &str) {
        self.store.push(level, module, message);
    }

    /// Set the active search query, recomputing the lowercase form once.
    pub fn set_search(&mut self, query: String) {
        self.search_text_lower = query.to_lowercase();
        self.search_text = query;
    }
}
