use gpui::{prelude::*, px, white, div};

use crate::design::{Colors, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS};
use crate::state::{Page, UiCommand};

pub fn sidebar(
    current_page: &Page,
    core_running: bool,
    c: Colors,
    commands: &mut Vec<UiCommand>,
) -> impl IntoElement {
    let status_color = if core_running { c.success } else { c.text_muted };
    let status_text = if core_running {
        "Core Running"
    } else {
        "Core Stopped"
    };
    let dot = if core_running { "●" } else { "○" };

    div()
        .w(px(200.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(c.sidebar_bg)
        .px(px(SPACE_LG))
        .py(px(SPACE_LG))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_color(white()).child("ZeroClash"))
                .child(
                    div()
                        .mt(px(SPACE_XS))
                        .text_color(status_color)
                        .child(format!("{dot} {status_text}")),
                ),
        )
        .child(div().mt(px(SPACE_LG)).flex().flex_col().children({
            let cp = current_page.clone();
            let mut cmds: Vec<UiCommand> = Vec::new();
            let nav: Vec<(&str, Page)> = vec![
                ("Dashboard", Page::Home),
                ("Proxies", Page::Proxies),
                ("Profiles", Page::Profiles),
                ("Connections", Page::Connections),
                ("Logs", Page::Logs),
                ("Settings", Page::Settings),
            ];
            nav.into_iter()
                .map(move |(label, page)| {
                    let active = cp == page;
                    let text = if active { c.accent } else { c.sidebar_text };
                    let bg = if active { c.sidebar_active_bg } else { gpui::transparent_black() };
                    let p = page.clone();
                    div()
                        .flex()
                        .items_center()
                        .h(px(32.0))
                        .px(px(SPACE_MD))
                        .rounded(px(crate::design::RADIUS_SM))
                        .bg(bg)
                        .text_color(text)
                        .cursor_pointer()
                        .child(label)
                        .on_click(move |_event, _window, _cx| {
                            // commands are pushed by the caller
                        })
                })
                .collect::<Vec<_>>()
        }))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_end()
                .child(
                    div()
                        .text_color(c.sidebar_text_muted)
                        .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                ),
        )
}
