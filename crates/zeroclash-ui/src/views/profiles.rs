use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};
use zeroclash_core::profile::ProfilePreview;

use crate::components::card::{card, page_heading};
use crate::design::{Colors, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn profiles_page(
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
        .child(page_heading(c, "Profiles"))
        .child(
            div().flex().mb(px(SPACE_MD)).child(
                div()
                    .bg(c.accent_dim)
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_MD))
                    .py(px(SPACE_XS))
                    .text_color(c.accent)
                    .cursor(CursorStyle::PointingHand)
                    .child(SharedString::from("+ Import from URL"))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, _e, _w, cx| {
                            this.import_dialog_visible = true;
                            this.import_url.clear();
                            cx.notify();
                        }),
                    ),
            ),
        )
        .child(match state.import_dialog_visible {
            true => import_dialog(c, &state.import_url, cx).into_any_element(),
            false => div().into_any_element(),
        })
        .child(match state.profile_previews.is_empty() {
            true => card(c)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .p(px(SPACE_LG))
                        .child(div().child("No profiles yet"))
                        .child(
                            div()
                                .text_color(c.text_muted)
                                .mt(px(SPACE_SM))
                                .child("Import a subscription URL to get started"),
                        ),
                )
                .into_any_element(),
            false => {
                let mut rows: Vec<gpui::AnyElement> = Vec::new();
                for p in &state.profile_previews {
                    rows.push(profile_row(c, p, cx).into_any_element());
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(SPACE_SM))
                    .children(rows)
                    .into_any_element()
            }
        })
}

fn import_dialog(c: Colors, url: &str, cx: &mut Context<AppState>) -> impl IntoElement {
    card(c).child(
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_color(c.text_primary)
                    .child("Import Profile from URL"),
            )
            .child(div().mt(px(SPACE_SM)).child("Subscription URL"))
            .child(
                div()
                    .mt(px(SPACE_XS))
                    .bg(c.input_bg)
                    .border_1()
                    .border_color(if url.is_empty() { c.border } else { c.accent })
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_MD))
                    .py(px(SPACE_XS))
                    .text_color(if url.is_empty() {
                        c.text_muted
                    } else {
                        c.text_primary
                    })
                    .child(if url.is_empty() {
                        SharedString::from("Click Paste to fill from clipboard")
                    } else {
                        SharedString::from(url)
                    }),
            )
            .child(
                div()
                    .mt(px(SPACE_SM))
                    .flex()
                    .gap(px(SPACE_SM))
                    .child(
                        div()
                            .bg(c.surface_alt)
                            .rounded(px(RADIUS_SM))
                            .px(px(SPACE_MD))
                            .py(px(SPACE_XS))
                            .text_color(c.text_secondary)
                            .cursor(CursorStyle::PointingHand)
                            .child("Paste")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new()
                                        && let Ok(text) = clipboard.get_text()
                                    {
                                        this.import_url = text;
                                        cx.notify();
                                    }
                                }),
                            ),
                    )
                    .child(
                        div()
                            .bg(c.success_dim)
                            .rounded(px(RADIUS_SM))
                            .px(px(SPACE_MD))
                            .py(px(SPACE_XS))
                            .text_color(c.success)
                            .cursor(CursorStyle::PointingHand)
                            .child("Import")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    let u = this.import_url.clone();
                                    if !u.is_empty() {
                                        this.push_command(UiCommand::ImportProfile(u));
                                    }
                                    this.import_dialog_visible = false;
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .bg(c.surface_alt)
                            .rounded(px(RADIUS_SM))
                            .px(px(SPACE_MD))
                            .py(px(SPACE_XS))
                            .text_color(c.text_muted)
                            .cursor(CursorStyle::PointingHand)
                            .child("Cancel")
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    this.import_dialog_visible = false;
                                    cx.notify();
                                }),
                            ),
                    ),
            ),
    )
}

fn profile_row(c: Colors, p: &ProfilePreview, cx: &mut Context<AppState>) -> impl IntoElement {
    let emoji = match p.itype.as_str() {
        "remote" => "☁",
        "local" => "💻",
        "merge" => "🔀",
        "script" => "📜",
        _ => "📄",
    };
    let uid_a = p.uid.clone();
    let uid_d = p.uid.clone();
    div()
        .bg(if p.is_current {
            c.accent_dim
        } else {
            c.surface
        })
        .border_1()
        .border_color(if p.is_current { c.accent } else { c.border })
        .rounded(px(RADIUS_SM))
        .p(px(SPACE_MD))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(SPACE_SM))
                .child(SharedString::from(emoji))
                .child(match p.is_current {
                    true => div()
                        .text_color(c.accent)
                        .child("● Active")
                        .into_any_element(),
                    false => div().into_any_element(),
                })
                .child(
                    div()
                        .text_color(c.text_primary)
                        .child(SharedString::from(&p.name)),
                )
                .child(
                    div()
                        .text_color(c.text_muted)
                        .child(SharedString::from(&p.itype)),
                ),
        )
        .child(
            div()
                .flex()
                .gap(px(SPACE_SM))
                .child(match p.is_current {
                    false => div()
                        .bg(c.accent_dim)
                        .rounded(px(RADIUS_SM))
                        .px(px(SPACE_SM))
                        .py(px(2.0))
                        .text_color(c.accent)
                        .cursor(CursorStyle::PointingHand)
                        .child("Activate")
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _e, _w, cx| {
                                this.push_command(UiCommand::ActivateProfile(uid_a.clone()));
                                cx.notify();
                            }),
                        )
                        .into_any_element(),
                    true => div().into_any_element(),
                })
                .child(
                    div()
                        .text_color(c.danger)
                        .cursor(CursorStyle::PointingHand)
                        .child("Delete")
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _e, _w, cx| {
                                this.push_command(UiCommand::DeleteProfile(uid_d.clone()));
                                cx.notify();
                            }),
                        ),
                ),
        )
}
