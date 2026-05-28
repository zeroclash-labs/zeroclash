use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::{card, page_heading, section_title};
use crate::components::traffic_graph::traffic_summary;
use crate::design::{Colors, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::i18n::tr;
use crate::state::{AppState, CoreInstallState, UiCommand};
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
        .id("dashboard-scroll")
        .overflow_y_scroll()
        .p(px(SPACE_XL))
        .child(page_heading(c, tr("ui.nav.dashboard")))
        .child(
            div()
                .flex()
                .gap(px(SPACE_MD))
                .mb(px(SPACE_MD))
                .child(div().flex_1().child(core_status_card(c, state, cx)))
                .child(
                    div().flex_1().child(
                        card(c).child(
                            div()
                                .flex()
                                .flex_col()
                                .child(section_title(c, tr("ui.pages.dashboard.trafficMonitor")))
                                .child(traffic_summary(&state.traffic, c)),
                        ),
                    ),
                ),
        )
        .child(
            div()
                .flex()
                .gap(px(SPACE_MD))
                .child(div().flex_1().child(mode_card(c, cx)))
                .child(div().flex_1().child(tun_card(c, state, cx))),
        )
        .child(
            div()
                .flex()
                .gap(px(SPACE_MD))
                .mt(px(SPACE_MD))
                .child(div().flex_1().child(system_info_card(c, state))),
        )
}

fn core_status_card(c: Colors, state: &AppState, cx: &mut Context<AppState>) -> impl IntoElement {
    let core_running = state.core_running;
    let status_color = if core_running {
        c.success
    } else {
        c.text_muted
    };
    let status_text = if core_running {
        tr("ui.pages.dashboard.running")
    } else {
        tr("ui.pages.dashboard.stopped")
    };
    let dot = if core_running { "●" } else { "○" };
    let btn_text = if core_running {
        tr("ui.pages.dashboard.stopCore")
    } else {
        tr("ui.pages.dashboard.startCore")
    };
    let btn_color = if core_running { c.danger } else { c.success };

    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(section_title(c, tr("ui.pages.dashboard.coreStatus")))
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
                    .child(btn_text)
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, _e, _w, cx| {
                            this.push_command(UiCommand::ToggleCore);
                            cx.notify();
                        }),
                    ),
            )
            .child(install_section(c, state, cx))
            .child(
                div()
                    .mt(px(SPACE_MD))
                    .flex()
                    .flex_col()
                    .gap(px(SPACE_XS))
                    .child(info_row(
                        c,
                        SharedString::from("HTTP"),
                        SharedString::from("127.0.0.1:7899"),
                    ))
                    .child(info_row(
                        c,
                        SharedString::from("SOCKS"),
                        SharedString::from("127.0.0.1:7898"),
                    )),
            ),
    )
}

fn install_section(c: Colors, state: &AppState, cx: &mut Context<AppState>) -> impl IntoElement {
    if state.core_running {
        return div();
    }
    match &state.core_install_state {
        CoreInstallState::Installing(msg) => div()
            .mt(px(SPACE_MD))
            .bg(c.warning_dim)
            .rounded(px(RADIUS_SM))
            .px(px(SPACE_LG))
            .py(px(SPACE_SM))
            .text_color(c.warning)
            .child(SharedString::from(msg.clone())),
        CoreInstallState::Failed(err) => div()
            .mt(px(SPACE_MD))
            .flex()
            .flex_col()
            .gap(px(SPACE_XS))
            .child(div().text_color(c.danger).child(SharedString::from(format!(
                "{}: {err}",
                tr("ui.pages.dashboard.coreInstaller.failed")
            ))))
            .child(
                div()
                    .bg(c.success_dim)
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_LG))
                    .py(px(SPACE_SM))
                    .text_color(c.success)
                    .cursor(CursorStyle::PointingHand)
                    .child(tr("ui.pages.dashboard.coreInstaller.retry"))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.push_command(UiCommand::InstallCore);
                            cx.notify();
                        }),
                    ),
            ),
        CoreInstallState::Installed(version) => {
            div()
                .mt(px(SPACE_MD))
                .text_color(c.success)
                .child(SharedString::from(format!(
                    "{} v{version}",
                    tr("ui.pages.dashboard.coreInstaller.installed")
                )))
        }
        CoreInstallState::Idle => div()
            .mt(px(SPACE_MD))
            .bg(c.success_dim)
            .rounded(px(RADIUS_SM))
            .px(px(SPACE_LG))
            .py(px(SPACE_SM))
            .text_color(c.success)
            .cursor(CursorStyle::PointingHand)
            .child(tr("ui.pages.dashboard.coreInstaller.installCore"))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _e, _w, cx| {
                    this.push_command(UiCommand::InstallCore);
                    cx.notify();
                }),
            ),
    }
}

fn system_info_card(c: Colors, state: &AppState) -> impl IntoElement {
    let proxy_state = if state.enable_system_proxy {
        tr("ui.pages.dashboard.systemProxyOn")
    } else {
        tr("ui.pages.dashboard.systemProxyOff")
    };
    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(section_title(c, tr("ui.pages.dashboard.system")))
            .child(
                div()
                    .mt(px(SPACE_SM))
                    .flex()
                    .flex_col()
                    .gap(px(SPACE_XS))
                    .child(info_row(
                        c,
                        tr("ui.pages.dashboard.version"),
                        SharedString::from(env!("CARGO_PKG_VERSION")),
                    ))
                    .child(info_row(
                        c,
                        tr("ui.pages.dashboard.systemProxy"),
                        proxy_state,
                    )),
            ),
    )
}

fn mode_card(c: Colors, cx: &mut Context<AppState>) -> impl IntoElement {
    let modes = [
        ("rule", tr("ui.pages.dashboard.mode.rule")),
        ("global", tr("ui.pages.dashboard.mode.global")),
        ("direct", tr("ui.pages.dashboard.mode.direct")),
    ];
    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(section_title(c, tr("ui.pages.dashboard.outboundMode")))
            .child(div().mt(px(SPACE_SM)).flex().gap(px(SPACE_SM)).children(
                modes.into_iter().map(|(id, label)| {
                    div()
                        .bg(c.surface_alt)
                        .rounded(px(RADIUS_SM))
                        .px(px(SPACE_MD))
                        .py(px(SPACE_XS))
                        .text_color(c.text_secondary)
                        .cursor(CursorStyle::PointingHand)
                        .child(label)
                        .on_mouse_up(MouseButton::Left, {
                            let m = id.to_string();
                            cx.listener(move |this, _e, _w, cx| {
                                this.push_command(UiCommand::SwitchMode(m.clone()));
                                cx.notify();
                            })
                        })
                }),
            )),
    )
}

fn tun_card(c: Colors, state: &AppState, cx: &mut Context<AppState>) -> impl IntoElement {
    let enabled = state.config.verge.latest_arc().enable_tun;
    let status_color = if enabled { c.success } else { c.text_muted };
    let btn_text = if enabled {
        tr("ui.pages.dashboard.disableTun")
    } else {
        tr("ui.pages.dashboard.enableTun")
    };
    let active_label = if enabled {
        tr("ui.pages.dashboard.active")
    } else {
        tr("ui.pages.dashboard.inactive")
    };
    let dot = if enabled { "●" } else { "○" };
    let btn_color = if enabled { c.danger } else { c.success };
    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(section_title(c, tr("ui.pages.dashboard.tunMode")))
            .child(
                div()
                    .mt(px(SPACE_MD))
                    .flex()
                    .items_center()
                    .gap(px(SPACE_SM))
                    .child(
                        div()
                            .text_color(status_color)
                            .child(SharedString::from(format!("{dot} {active_label}"))),
                    ),
            )
            .child(
                div()
                    .mt(px(SPACE_MD))
                    .bg(if enabled { c.danger_dim } else { c.success_dim })
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_LG))
                    .py(px(SPACE_SM))
                    .text_color(btn_color)
                    .cursor(CursorStyle::PointingHand)
                    .child(btn_text)
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, _e, _w, cx| {
                            this.push_command(UiCommand::ToggleTun);
                            cx.notify();
                        }),
                    ),
            ),
    )
}

fn info_row(c: Colors, label: SharedString, value: SharedString) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .child(div().text_color(c.text_muted).child(label))
        .child(div().text_color(c.text_secondary).child(value))
}
