use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::{card, page_heading, section_title};
use crate::components::traffic_graph::traffic_summary;
use crate::design::{Colors, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn dashboard_page(
    state: &AppState,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;

    div()
        .size_full()
        .p(px(SPACE_XL))
        .child(page_heading(c, "Dashboard"))
        .child(
            div()
                .flex()
                .gap(px(SPACE_MD))
                .mb(px(SPACE_MD))
                .child(
                    div()
                        .flex_1()
                        .child(core_status_card(c, state.core_running, cx)),
                )
                .child(
                    div().w_full().child(
                        card(c).child(
                            div()
                                .flex()
                                .flex_col()
                                .child(section_title(c, "TRAFFIC MONITOR"))
                                .child(traffic_summary(&state.traffic, c)),
                        ),
                    ),
                ),
        )
        .child(
            div()
                .flex()
                .gap(px(SPACE_MD))
                .child(div().flex_1().child(system_info_card(c, state))),
        )
}

fn core_status_card(c: Colors, core_running: bool, cx: &mut Context<AppState>) -> impl IntoElement {
    let status_color = if core_running {
        c.success
    } else {
        c.text_muted
    };
    let status_text = if core_running { "Running" } else { "Stopped" };
    let dot = if core_running { "●" } else { "○" };
    let btn_text = if core_running {
        "Stop Core"
    } else {
        "Start Core"
    };
    let btn_color = if core_running { c.danger } else { c.success };

    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(section_title(c, "CORE STATUS"))
            .child(
                div()
                    .mt(px(SPACE_MD))
                    .flex()
                    .items_center()
                    .gap(px(SPACE_SM))
                    .child(
                        div()
                            .text_color(status_color)
                            .child(SharedString::from(format!("{dot} {status_text}"))),
                    ),
            )
            .child(
                div()
                    .mt(px(SPACE_MD))
                    .bg(if core_running {
                        c.danger_dim
                    } else {
                        c.success_dim
                    })
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_LG))
                    .py(px(SPACE_SM))
                    .text_color(btn_color)
                    .cursor(CursorStyle::PointingHand)
                    .child(SharedString::from(btn_text))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, _e, _w, cx| {
                            this.push_command(UiCommand::ToggleCore);
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .mt(px(SPACE_MD))
                    .flex()
                    .flex_col()
                    .gap(px(SPACE_XS))
                    .child(info_row(c, "HTTP", "127.0.0.1:7899"))
                    .child(info_row(c, "SOCKS", "127.0.0.1:7898")),
            ),
    )
}

fn system_info_card(c: Colors, state: &AppState) -> impl IntoElement {
    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(section_title(c, "SYSTEM"))
            .child(
                div()
                    .mt(px(SPACE_SM))
                    .flex()
                    .flex_col()
                    .gap(px(SPACE_XS))
                    .child(info_row(c, "Version", env!("CARGO_PKG_VERSION")))
                    .child(info_row(
                        c,
                        "System Proxy",
                        if state.enable_system_proxy {
                            "ON"
                        } else {
                            "OFF"
                        },
                    )),
            ),
    )
}

fn info_row(c: Colors, label: &str, value: &str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(label)),
        )
        .child(
            div()
                .text_color(c.text_secondary)
                .child(SharedString::from(value)),
        )
}
