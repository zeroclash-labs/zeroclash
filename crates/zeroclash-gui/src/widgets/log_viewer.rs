//! Log viewer with level filtering and text search.

use egui::{Color32, RichText, ScrollArea};

/// Log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Self::Error => Color32::RED,
            Self::Warn => Color32::from_rgb(251, 188, 4),
            Self::Info => Color32::LIGHT_BLUE,
            Self::Debug => Color32::GRAY,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" | "ERR" => Some(Self::Error),
            "WARN" | "WARNING" => Some(Self::Warn),
            "INFO" => Some(Self::Info),
            "DEBUG" | "DBG" => Some(Self::Debug),
            _ => None,
        }
    }
}

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

/// Ring buffer for log entries.
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
        let entry = LogEntry {
            timestamp: now_timestamp(),
            level,
            module: module.to_string(),
            message: message.to_string(),
        };
        self.entries.push(entry);
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

// Placeholder chrono usage; remove if we drop the dep.
/// Simple time formatting without chrono dependency.
fn now_timestamp() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs() % 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// Log viewer widget.
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

/// Render the log viewer.
pub fn log_viewer_ui(ui: &mut egui::Ui, viewer: &mut LogViewer) {
    ui.heading("Logs");
    ui.separator();

    // Toolbar
    ui.horizontal(|ui| {
        ui.label("Level:");
        for level in &[LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug] {
            if ui
                .selectable_label(viewer.filter_level == *level, level.as_str())
                .clicked()
            {
                viewer.filter_level = *level;
            }
        }
        ui.separator();
        ui.label("Search:");
        ui.text_edit_singleline(&mut viewer.search_text);
        if ui.button("✕").on_hover_text("Clear search").clicked() {
            viewer.search_text.clear();
        }
        ui.separator();
        ui.checkbox(&mut viewer.auto_scroll, "Auto-scroll");
        if ui.button("Clear").clicked() {
            viewer.store.clear();
        }
    });
    ui.separator();

    // Filtered entries
    let filtered: Vec<&LogEntry> = viewer
        .store
        .entries()
        .iter()
        .filter(|e| {
            e.level <= viewer.filter_level
                && (viewer.search_text.is_empty()
                    || e.message
                        .to_lowercase()
                        .contains(&viewer.search_text.to_lowercase())
                    || e.module
                        .to_lowercase()
                        .contains(&viewer.search_text.to_lowercase()))
        })
        .collect();

    let scroll = ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(viewer.auto_scroll);

    scroll.show(ui, |ui| {
        if filtered.is_empty() {
            ui.label(
                RichText::new("No log entries matching filter")
                    .color(Color32::GRAY)
                    .size(12.0),
            );
            return;
        }

        for entry in &filtered {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&entry.timestamp)
                        .color(Color32::DARK_GRAY)
                        .size(11.0)
                        .monospace(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(entry.level.as_str())
                        .color(entry.level.color())
                        .size(11.0)
                        .strong()
                        .monospace(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&entry.module)
                        .color(Color32::GRAY)
                        .size(11.0)
                        .monospace(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&entry.message)
                        .size(11.0)
                        .monospace(),
                );
            });
        }
    });
}
