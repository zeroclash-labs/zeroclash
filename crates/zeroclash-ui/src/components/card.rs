use gpui::{FontWeight, SharedString, div, prelude::*, px};

use crate::design::{Colors, RADIUS_MD, SPACE_LG};

pub fn card(c: Colors) -> gpui::Div {
    div()
        .bg(c.surface)
        .border_1()
        .border_color(c.border)
        .rounded(px(RADIUS_MD))
        .p(px(SPACE_LG))
}

pub fn section_title(c: Colors, text: impl Into<SharedString>) -> gpui::Div {
    div().text_color(c.text_muted).child(text.into())
}

pub fn page_heading(c: Colors, text: impl Into<SharedString>) -> gpui::Div {
    div()
        .text_2xl()
        .text_color(c.text_primary)
        .font_weight(FontWeight::BOLD)
        .mb(px(SPACE_LG))
        .child(text.into())
}
