use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::components::settings_group::{info_row, settings_section, toggle_row};
use crate::design::SPACE_XL;
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
        .p(px(SPACE_XL))
        .child(page_heading(c, "Settings"))
        .child(
            settings_section(c, "Appearance").child(
                div()
                    .flex()
                    .flex_col()
                    .child(clickable_info_row(c, "Theme", &verge.theme_mode, cx))
                    .child(info_row(c, "Language", &verge.language)),
            ),
        )
        .child(
            settings_section(c, "System").child(
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
                            .child(toggle_row(c, "System Proxy", state.enable_system_proxy)),
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
                            .child(toggle_row(c, "TUN Mode", verge.enable_tun)),
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
                            .child(toggle_row(c, "Auto Start", verge.enable_auto_start)),
                    ),
            ),
        )
        .child(
            settings_section(c, "Ports").child(
                div()
                    .flex()
                    .flex_col()
                    .child(info_row(
                        c,
                        "HTTP",
                        &format!("127.0.0.1:{}", verge.http_port),
                    ))
                    .child(info_row(
                        c,
                        "SOCKS",
                        &format!("127.0.0.1:{}", verge.socks_port),
                    ))
                    .child(info_row(
                        c,
                        "Mixed",
                        &format!("127.0.0.1:{}", verge.mixed_port),
                    )),
            ),
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
