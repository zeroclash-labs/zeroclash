use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::design::{Colors, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn proxies_page(
    state: &AppState,
    _w: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;

    div()
        .size_full()
        .p(px(SPACE_XL))
        .flex()
        .flex_col()
        .child(page_heading(c, "Proxies"))
        .child({
            let mut children: Vec<gpui::AnyElement> = Vec::new();
            for g in &state.proxy_groups {
                children.push(proxy_card(c, g, cx).into_any_element());
            }
            div().flex().flex_col().gap(px(SPACE_SM)).children(children)
        })
}

fn proxy_card(
    c: Colors,
    g: &zeroclash_core::mihomo::ProxyGroup,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let is_selector = g.group_type == "selector" || g.group_type == "select";
    div()
        .bg(c.surface)
        .border_1()
        .border_color(c.border)
        .rounded(px(crate::design::RADIUS_MD))
        .p(px(SPACE_MD))
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_color(c.text_primary)
                        .child(SharedString::from(&g.name)),
                )
                .child(
                    div()
                        .text_color(c.text_muted)
                        .child(SharedString::from(&g.group_type)),
                ),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_color(c.accent)
                        .child(SharedString::from(g.now.as_deref().unwrap_or("none"))),
                )
                .child({
                    let group_name = g.name.clone();
                    div()
                        .bg(c.surface_alt)
                        .rounded(px(crate::design::RADIUS_SM))
                        .px(px(SPACE_SM))
                        .py(px(2.0))
                        .text_color(c.text_secondary)
                        .cursor(CursorStyle::PointingHand)
                        .child("Test")
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _e, _w, cx| {
                                this.push_command(UiCommand::TestDelay(group_name.clone()));
                                cx.notify();
                            }),
                        )
                }),
        )
        .when(!g.history.is_empty(), |this| {
            let last = &g.history[g.history.len() - 1];
            this.child(
                div()
                    .text_color(c.success)
                    .child(SharedString::from(format!("{}ms", last.delay))),
            )
        })
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(SPACE_XS))
                .children(g.all.iter().take(8).map(|name| {
                    let is_current = g.now.as_deref() == Some(name.as_str());
                    let bg = if is_current {
                        c.accent_dim
                    } else {
                        c.surface_alt
                    };
                    let fg = if is_current {
                        c.accent
                    } else {
                        c.text_secondary
                    };
                    let group_name = g.name.clone();
                    let proxy_name = name.clone();
                    div()
                        .bg(bg)
                        .rounded(px(crate::design::RADIUS_SM))
                        .px(px(SPACE_SM))
                        .py(px(2.0))
                        .text_color(fg)
                        .when(is_selector, |this| {
                            this.cursor(CursorStyle::PointingHand).on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    this.push_command(UiCommand::SelectProxy(
                                        group_name.clone(),
                                        proxy_name.clone(),
                                    ));
                                    cx.notify();
                                }),
                            )
                        })
                        .child(SharedString::from(name.as_str()))
                })),
        )
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(format!("{} nodes", g.all.len()))),
        )
}
