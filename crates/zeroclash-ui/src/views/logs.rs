use gpui::{
    Context, CursorStyle, KeyDownEvent, MouseButton, SharedString, Window, div, prelude::*, px,
};

use crate::components::card::page_heading;
use crate::components::log_viewer::{LogEntry, LogLevel};
use crate::design::{Colors, RADIUS_SM, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::state::AppState;
use crate::theme::Theme;

pub fn logs_page(
    state: &AppState,
    _w: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;
    let viewer = &state.log_viewer;
    let total = viewer.store.len();

    let filter_level = viewer.filter_level;
    let needle = viewer.search_text_lower.as_str();

    let filtered: Vec<&LogEntry> = viewer
        .store
        .entries()
        .filter(|e| {
            e.level <= filter_level
                && (needle.is_empty()
                    || e.message_lower.contains(needle)
                    || e.module_lower.contains(needle))
        })
        .collect();

    if viewer.auto_scroll {
        viewer.scroll.scroll_to_bottom();
    }

    let search_focus = state.logs_search_focus.clone();

    div()
        .size_full()
        .p(px(SPACE_XL))
        .flex()
        .flex_col()
        .id("logs-scroll")
        .track_focus(&search_focus)
        .key_context("LogsPage")
        .on_key_down(cx.listener(handle_logs_key_down))
        .child(page_heading(c, format!("Logs ({total})")))
        .child(toolbar(c, viewer, cx))
        .child(log_entries(c, &filtered, &viewer.scroll))
}

fn toolbar(
    c: Colors,
    viewer: &crate::components::log_viewer::LogViewer,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    div()
        .mb(px(SPACE_SM))
        .bg(c.surface)
        .rounded(px(RADIUS_SM))
        .p(px(SPACE_SM))
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .child(level_chip(
            c,
            "Errors",
            LogLevel::Error,
            viewer.filter_level,
            cx,
        ))
        .child(level_chip(
            c,
            "Warnings",
            LogLevel::Warn,
            viewer.filter_level,
            cx,
        ))
        .child(level_chip(
            c,
            "Info",
            LogLevel::Info,
            viewer.filter_level,
            cx,
        ))
        .child(level_chip(
            c,
            "All",
            LogLevel::Debug,
            viewer.filter_level,
            cx,
        ))
        .child(search_field(c, &viewer.search_text))
        .child(
            div()
                .text_color(if viewer.auto_scroll {
                    c.accent
                } else {
                    c.text_muted
                })
                .cursor(CursorStyle::PointingHand)
                .child("Auto")
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.log_viewer.auto_scroll = !this.log_viewer.auto_scroll;
                        if this.log_viewer.auto_scroll {
                            this.log_viewer.scroll.scroll_to_bottom();
                        }
                        cx.notify();
                    }),
                ),
        )
        .child(
            div()
                .text_color(c.danger)
                .cursor(CursorStyle::PointingHand)
                .child("Clear")
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.log_viewer.store.clear();
                        cx.notify();
                    }),
                ),
        )
}

fn search_field(c: Colors, query: &str) -> impl IntoElement {
    let display: SharedString = if query.is_empty() {
        SharedString::from("type to filter / Esc to clear")
    } else {
        SharedString::from(format!("/ {query}"))
    };
    div()
        .ml(px(SPACE_SM))
        .px(px(SPACE_SM))
        .py(px(2.0))
        .rounded(px(RADIUS_SM))
        .bg(c.input_bg)
        .text_color(if query.is_empty() {
            c.text_muted
        } else {
            c.text_primary
        })
        .font_family(crate::fonts::mono_family())
        .child(display)
}

fn log_entries(c: Colors, filtered: &[&LogEntry], scroll: &gpui::ScrollHandle) -> impl IntoElement {
    if filtered.is_empty() {
        return div()
            .flex_1()
            .text_color(c.text_muted)
            .child("No matching entries")
            .into_any_element();
    }
    div()
        .id("logs-entries")
        .flex_1()
        .flex()
        .flex_col()
        .overflow_y_scroll()
        .track_scroll(scroll)
        .children(filtered.iter().map(|e| log_entry(c, e).into_any_element()))
        .into_any_element()
}

fn level_chip(
    c: Colors,
    label: &str,
    level: LogLevel,
    current: LogLevel,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let active = current == level;
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
            LogLevel::Debug => gpui::transparent_black(),
        }
    } else {
        gpui::transparent_black()
    };
    div()
        .bg(bg)
        .rounded(px(RADIUS_SM))
        .px(px(SPACE_XS))
        .py(px(2.0))
        .text_color(fg)
        .cursor(CursorStyle::PointingHand)
        .child(SharedString::from(label))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _e, _w, cx| {
                this.log_viewer.filter_level = level;
                cx.notify();
            }),
        )
}

fn log_entry(c: Colors, e: &LogEntry) -> impl IntoElement {
    let lc = match e.level {
        LogLevel::Error => c.danger,
        LogLevel::Warn => c.warning,
        LogLevel::Info => c.accent,
        LogLevel::Debug => c.text_muted,
    };
    div()
        .flex()
        .gap(px(SPACE_XS))
        .py(px(1.0))
        .font_family(crate::fonts::mono_family())
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(&e.timestamp)),
        )
        .child(
            div()
                .text_color(lc)
                .child(SharedString::from(e.level.as_str())),
        )
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(&e.module)),
        )
        .child(
            div()
                .text_color(c.text_primary)
                .child(SharedString::from(&e.message)),
        )
}

fn handle_logs_key_down(
    state: &mut AppState,
    event: &KeyDownEvent,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) {
    let ks = &event.keystroke;

    match ks.key.as_str() {
        "escape" => {
            state.log_viewer.set_search(String::new());
            cx.notify();
            return;
        }
        "backspace" => {
            let mut s = state.log_viewer.search_text.clone();
            s.pop();
            state.log_viewer.set_search(s);
            cx.notify();
            return;
        }
        _ => {}
    }

    if let Some(ch) = ks.key_char.as_ref()
        && !ch.is_empty()
        && !ks.modifiers.platform
        && !ks.modifiers.control
        && !ks.modifiers.alt
    {
        let mut s = state.log_viewer.search_text.clone();
        s.push_str(ch);
        state.log_viewer.set_search(s);
        cx.notify();
    }
}
