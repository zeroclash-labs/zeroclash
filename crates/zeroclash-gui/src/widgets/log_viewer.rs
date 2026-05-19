//! Log viewer with level filtering, search, and auto-scroll.
//! All colors use design tokens for dark/light mode compatibility.

use crate::design::{FONT_SM, FONT_XS, SPACE_SM, SPACE_XS, palette};
use egui::{RichText, ScrollArea};

/// Log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERR",
            Self::Warn => "WRN",
            Self::Info => "INF",
            Self::Debug => "DBG",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" | "ERR" => Some(Self::Error),
            "WARN" | "WARNING" | "WRN" => Some(Self::Warn),
            "INFO" | "INF" => Some(Self::Info),
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

/// Log viewer widget state.
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
    let c = palette(ui.ctx());

    // Header
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Logs")
                .size(16.0)
                .color(c.text_primary)
                .strong(),
        );
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(format!("({})", viewer.store.entries().len()))
                .size(FONT_SM)
                .color(c.text_muted),
        );
    });
    ui.add_space(SPACE_SM);

    // Toolbar
    let toolbar_frame = egui::Frame::default()
        .fill(c.surface)
        .corner_radius(crate::design::RADIUS_SM)
        .inner_margin(egui::vec2(SPACE_SM, SPACE_XS));
    toolbar_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Level filter chips
            let levels = [
                (LogLevel::Error, "Errors"),
                (LogLevel::Warn, "Warnings"),
                (LogLevel::Info, "Info"),
                (LogLevel::Debug, "All"),
            ];
            for (level, label) in &levels {
                let active = viewer.filter_level == *level;
                let lbl = level_label(ui, label, active, *level, c);
                if lbl.clicked() {
                    viewer.filter_level = *level;
                }
                ui.add_space(SPACE_XS);
            }

            ui.separator();
            // Search
            ui.label(RichText::new("🔍").size(FONT_XS));
            ui.add(
                egui::TextEdit::singleline(&mut viewer.search_text)
                    .hint_text("Filter...")
                    .desired_width(140.0),
            );
            if !viewer.search_text.is_empty()
                && ui.small_button(RichText::new("✕").size(FONT_XS)).clicked()
            {
                viewer.search_text.clear();
            }

            ui.separator();
            ui.checkbox(&mut viewer.auto_scroll, "Auto");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(RichText::new("Clear").size(FONT_XS))
                    .clicked()
                {
                    viewer.store.clear();
                }
            });
        });
    });
    ui.add_space(SPACE_XS);

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
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_SM * 3.0);
                ui.label(
                    RichText::new("No matching entries")
                        .size(FONT_SM)
                        .color(c.text_muted),
                );
            });
            return;
        }

        for entry in &filtered {
            ui.horizontal(|ui| {
                // Timestamp
                ui.label(
                    RichText::new(&entry.timestamp)
                        .size(FONT_XS)
                        .color(c.text_muted)
                        .monospace(),
                );
                ui.add_space(SPACE_XS);

                // Level badge
                let level_color = match entry.level {
                    LogLevel::Error => c.danger,
                    LogLevel::Warn => c.warning,
                    LogLevel::Info => c.accent,
                    LogLevel::Debug => c.text_muted,
                };
                ui.label(
                    RichText::new(entry.level.as_str())
                        .size(FONT_XS)
                        .color(level_color)
                        .strong()
                        .monospace(),
                );
                ui.add_space(SPACE_XS);

                // Module
                ui.label(
                    RichText::new(&entry.module)
                        .size(FONT_XS)
                        .color(c.text_muted)
                        .monospace(),
                );
                ui.add_space(SPACE_XS);

                // Message
                ui.label(RichText::new(&entry.message).size(FONT_XS).monospace());
            });
        }
    });
}

/// Render a selectable level filter label.
fn level_label(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    level: LogLevel,
    c: &'static crate::design::Colors,
) -> egui::Response {
    let fg = if active {
        match level {
            LogLevel::Error => c.danger,
            LogLevel::Warn => c.warning,
            LogLevel::Info => c.accent,
            LogLevel::Debug => c.text_primary,
        }
    } else {
        c.text_muted
    };
    let bg = if active {
        match level {
            LogLevel::Error => c.danger_dim,
            LogLevel::Warn => c.warning_dim,
            LogLevel::Info => c.accent_dim,
            LogLevel::Debug => Color32::TRANSPARENT,
        }
    } else {
        Color32::TRANSPARENT
    };

    egui::Frame::default()
        .fill(bg)
        .corner_radius(crate::design::RADIUS_SM)
        .inner_margin(egui::vec2(SPACE_XS + 2.0, 1.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(FONT_XS).color(fg))
        })
        .response
}

use egui::Color32;
