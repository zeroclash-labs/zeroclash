use gpui::{SharedString, div, prelude::*, px};

use crate::design::{Colors, RADIUS_MD, SPACE_LG, SPACE_SM, SPACE_XS};

pub fn toggle_row(c: Colors, label: &str, enabled: bool) -> gpui::Div {
    let toggle_bg = if enabled { c.success } else { c.text_muted };
    let dot_offset = if enabled { px(20.0) } else { px(2.0) };

    div()
        .flex()
        .justify_between()
        .items_center()
        .py(px(SPACE_XS))
        .child(div().text_color(c.text_secondary).child(SharedString::from(label)))
        .child(
            div()
                .w(px(40.0))
                .h(px(22.0))
                .bg(toggle_bg)
                .rounded(px(11.0))
                .flex()
                .items_center()
                .child(
                    div()
                        .w(px(18.0))
                        .h(px(18.0))
                        .bg(gpui::white())
                        .rounded(px(9.0))
                        .ml(dot_offset),
                ),
        )
}

pub fn settings_section(c: Colors, title: &str) -> gpui::Div {
    div()
        .bg(c.surface)
        .border_1()
        .border_color(c.border)
        .rounded(px(RADIUS_MD))
        .p(px(SPACE_LG))
        .mb(px(SPACE_SM))
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_color(c.accent)
                        .mb(px(SPACE_SM))
                        .child(SharedString::from(title)),
                ),
        )
}

pub fn info_row(c: Colors, label: &str, value: &str) -> gpui::Div {
    div()
        .flex()
        .justify_between()
        .py(px(SPACE_XS))
        .child(div().text_color(c.text_muted).child(SharedString::from(label)))
        .child(div().text_color(c.text_secondary).child(SharedString::from(value)))
}
