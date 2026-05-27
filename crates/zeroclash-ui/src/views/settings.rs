use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::components::settings_group::{info_row, settings_section, toggle_row};
use crate::design::SPACE_XL;
use crate::i18n::tr;
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn settings_page(
    state: &AppState,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;
    let verge = state.config.verge.latest_arc();

    div()
        .size_full()
        .id("settings-scroll")
        .overflow_y_scroll()
        .p(px(SPACE_XL))
        .child(page_heading(c, tr("ui.pages.settings.title")))
        .child(
            settings_section(c, tr("ui.pages.settings.appearance").as_ref()).child(
                div()
                    .flex()
                    .flex_col()
                    .child(clickable_info_row(
                        c,
                        tr("ui.pages.settings.theme").as_ref(),
                        &verge.theme_mode,
                        cx,
                    ))
                    .child(language_row(
                        c,
                        tr("ui.pages.settings.language").as_ref(),
                        &verge.language,
                        cx,
                    )),
            ),
        )
        .child(
            settings_section(c, tr("ui.pages.settings.system").as_ref()).child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .cursor(CursorStyle::PointingHand)
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    this.push_command(UiCommand::ToggleSystemProxy);
                                    cx.notify();
                                }),
                            )
                            .child(toggle_row(
                                c,
                                tr("ui.pages.settings.systemProxy").as_ref(),
                                state.enable_system_proxy,
                            )),
                    )
                    .child(
                        div()
                            .cursor(CursorStyle::PointingHand)
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    this.push_command(UiCommand::ToggleTun);
                                    cx.notify();
                                }),
                            )
                            .child(toggle_row(
                                c,
                                tr("ui.pages.settings.tunMode").as_ref(),
                                verge.enable_tun,
                            )),
                    )
                    .child(
                        div()
                            .cursor(CursorStyle::PointingHand)
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    this.push_command(UiCommand::ToggleAutoStart);
                                    cx.notify();
                                }),
                            )
                            .child(toggle_row(
                                c,
                                tr("ui.pages.settings.autoStart").as_ref(),
                                verge.enable_auto_start,
                            )),
                    ),
            ),
        )
        .child(
            settings_section(c, tr("ui.pages.settings.ports").as_ref()).child(
                div()
                    .flex()
                    .flex_col()
                    .child(info_row(
                        c,
                        tr("ui.pages.settings.http").as_ref(),
                        &format!("127.0.0.1:{}", verge.http_port),
                    ))
                    .child(info_row(
                        c,
                        tr("ui.pages.settings.socks").as_ref(),
                        &format!("127.0.0.1:{}", verge.socks_port),
                    ))
                    .child(info_row(
                        c,
                        tr("ui.pages.settings.mixed").as_ref(),
                        &format!("127.0.0.1:{}", verge.mixed_port),
                    )),
            ),
        )
}

/// Locales the user can cycle through in the Settings page. The value
/// stored in `verge.language` may be empty (system default) or any of
/// these explicit codes — clicking the row advances to the next one and
/// loops back to "" (system) at the end.
const LANGUAGES: &[&str] = &["", "en", "zh", "zhtw", "jp", "ko", "de", "ru", "es"];

fn language_row(
    c: crate::design::Colors,
    label: &str,
    value: &str,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let next_lang = {
        let idx = LANGUAGES.iter().position(|l| *l == value).unwrap_or(0);
        let next = (idx + 1) % LANGUAGES.len();
        LANGUAGES[next].to_string()
    };
    let display = if value.is_empty() { "system" } else { value };

    div()
        .flex()
        .justify_between()
        .cursor(CursorStyle::PointingHand)
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _e, _w, cx| {
                let next = next_lang.clone();
                this.config.verge.edit_draft(|v| v.language = next.clone());
                this.config.verge.apply();
                this.save_config();
                if next.is_empty() {
                    zeroclash_i18n::sync_locale(None);
                } else {
                    zeroclash_i18n::set_locale(&next);
                }
                cx.notify();
            }),
        )
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(label.to_string())),
        )
        .child(
            div()
                .text_color(c.accent)
                .child(SharedString::from(display.to_string())),
        )
}

fn clickable_info_row(
    c: crate::design::Colors,
    label: &str,
    value: &str,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let next_theme = match value {
        "dark" => "light",
        "light" => "system",
        _ => "dark",
    };
    let next = next_theme.to_string();
    div()
        .flex()
        .justify_between()
        .cursor(CursorStyle::PointingHand)
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _e, _w, cx| {
                this.config
                    .verge
                    .edit_draft(|v| v.theme_mode = next.clone());
                this.config.verge.apply();
                this.save_config();
                let theme = Theme::parse_theme(&next);
                cx.set_global(theme);
                cx.notify();
            }),
        )
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(label)),
        )
        .child(div().text_color(c.accent).child(SharedString::from(value)))
}
