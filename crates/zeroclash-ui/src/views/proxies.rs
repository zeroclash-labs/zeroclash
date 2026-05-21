use gpui::{Context, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::design::{Colors, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::state::AppState;
use crate::theme::Theme;

pub fn proxies_page(state: &AppState, _w: &mut Window, cx: &mut Context<AppState>) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;

    div().size_full().p(px(SPACE_XL)).flex().flex_col()
        .child(page_heading(c, "Proxies"))
        .child(div().flex().flex_col().gap(px(SPACE_SM)).children(
            state.proxy_groups.iter().map(|g| proxy_card(c, g)),
        ))
}

fn proxy_card(c: Colors, g: &zeroclash_core::mihomo::ProxyGroup) -> impl IntoElement {
    div()
        .bg(c.surface).border_1().border_color(c.border)
        .rounded(px(crate::design::RADIUS_MD)).p(px(SPACE_MD))
        .flex().flex_col().gap(px(SPACE_XS))
        .child(
            div().flex().justify_between().items_center()
                .child(div().text_color(c.text_primary).child(SharedString::from(&g.name)))
                .child(div().text_color(c.text_muted).child(SharedString::from(&g.group_type))),
        )
        .child(
            div().text_color(c.accent).child(SharedString::from(g.now.as_deref().unwrap_or("none"))),
        )
        .child(
            div().flex().flex_wrap().gap(px(SPACE_XS)).children(
                g.all.iter().take(8).map(|name| {
                    div().bg(c.surface_alt).rounded(px(crate::design::RADIUS_SM))
                        .px(px(SPACE_SM)).py(px(2.0))
                        .text_color(c.text_secondary)
                        .child(SharedString::from(name.as_str()))
                }),
            ),
        )
        .child(
            div().text_color(c.text_muted).child(SharedString::from(format!("{} nodes", g.all.len()))),
        )
}
