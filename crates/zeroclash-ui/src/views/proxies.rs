use gpui::{
    Context, CursorStyle, KeyDownEvent, MouseButton, SharedString, Window, div, prelude::*, px,
};

use crate::components::card::page_heading;
use crate::design::{Colors, RADIUS_SM, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn proxies_page(
    state: &AppState,
    _w: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;
    let needle = state.proxies_filter_lower.as_str();
    let search_focus = state.proxies_search_focus.clone();

    div()
        .size_full()
        .p(px(SPACE_XL))
        .flex()
        .flex_col()
        .id("proxies-scroll")
        .track_focus(&search_focus)
        .key_context("ProxiesPage")
        .on_key_down(cx.listener(handle_proxies_key_down))
        .overflow_y_scroll()
        .child(page_heading(c, "Proxies"))
        .child(search_field(c, &state.proxies_filter))
        .child({
            let mut children: Vec<gpui::AnyElement> = Vec::new();
            for g in &state.proxy_groups {
                if !group_matches(g, needle) {
                    continue;
                }
                children.push(proxy_card(c, g, needle, cx).into_any_element());
            }
            div().flex().flex_col().gap(px(SPACE_SM)).children(children)
        })
}

fn group_matches(g: &zeroclash_core::mihomo::ProxyGroup, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if g.name.to_lowercase().contains(needle) {
        return true;
    }
    g.all.iter().any(|n| n.to_lowercase().contains(needle))
}

fn search_field(c: Colors, query: &str) -> impl IntoElement {
    let display: SharedString = if query.is_empty() {
        SharedString::from("type to filter groups / nodes (Esc to clear)")
    } else {
        SharedString::from(format!("/ {query}"))
    };
    div()
        .mb(px(SPACE_SM))
        .bg(c.input_bg)
        .rounded(px(RADIUS_SM))
        .px(px(SPACE_MD))
        .py(px(SPACE_XS))
        .text_color(if query.is_empty() {
            c.text_muted
        } else {
            c.text_primary
        })
        .font_family(crate::fonts::mono_family())
        .child(display)
}

fn proxy_card(
    c: Colors,
    g: &zeroclash_core::mihomo::ProxyGroup,
    needle: &str,
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
        .child(node_grid(c, g, needle, is_selector, cx))
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(format!("{} nodes", g.all.len()))),
        )
}

fn node_grid(
    c: Colors,
    g: &zeroclash_core::mihomo::ProxyGroup,
    needle: &str,
    is_selector: bool,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(px(SPACE_XS))
        .children(g.all.iter().filter_map(|name| {
            if !needle.is_empty() && !name.to_lowercase().contains(needle) {
                return None;
            }
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
            Some(
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
                    .child(SharedString::from(name.as_str())),
            )
        }))
}

fn handle_proxies_key_down(
    state: &mut AppState,
    event: &KeyDownEvent,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) {
    let ks = &event.keystroke;

    match ks.key.as_str() {
        "escape" => {
            state.set_proxies_filter(String::new());
            cx.notify();
            return;
        }
        "backspace" => {
            let mut s = state.proxies_filter.clone();
            s.pop();
            state.set_proxies_filter(s);
            cx.notify();
            return;
        }
        _ => {}
    }

    if let Some(ch) = ks.key_char.as_ref()
        && !ch.is_empty()
        && !ks.modifiers.platform
        && !ks.modifiers.control
        && !ks.modifiers.alt
    {
        let mut s = state.proxies_filter.clone();
        s.push_str(ch);
        state.set_proxies_filter(s);
        cx.notify();
    }
}
