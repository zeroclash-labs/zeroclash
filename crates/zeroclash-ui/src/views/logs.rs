use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::components::log_viewer::LogLevel;
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

    let filtered: Vec<_> = viewer
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

    div()
        .size_full()
        .p(px(SPACE_XL))
        .flex()
        .flex_col()
        .child(page_heading(
            c,
            format!("Logs ({})", viewer.store.entries().len()),
        ))
        // Toolbar
        .child(
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
                // Auto-scroll toggle
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
                                cx.notify();
                            }),
                        ),
                )
                // Clear button
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
                ),
        )
        // Log entries
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .children(match filtered.is_empty() {
                    true => vec![
                        div()
                            .text_color(c.text_muted)
                            .child("No matching entries")
                            .into_any_element(),
                    ],
                    false => filtered
                        .into_iter()
                        .map(|e| log_entry(c, e).into_any_element())
                        .collect(),
                }),
        )
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

fn log_entry(c: Colors, e: &crate::components::log_viewer::LogEntry) -> impl IntoElement {
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
