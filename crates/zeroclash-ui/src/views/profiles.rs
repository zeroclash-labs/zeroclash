use gpui::{
    Context, CursorStyle, FocusHandle, KeyDownEvent, MouseButton, SharedString, Window, div,
    prelude::*, px,
};
use zeroclash_core::profile::ProfilePreview;

use crate::components::card::{card, page_heading};
use crate::components::log_viewer::LogLevel;
use crate::design::{Colors, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS};
use crate::i18n::tr;
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn profiles_page(
    state: &AppState,
    _w: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;
    let import_focus = state.import_focus.clone();

    div()
        .size_full()
        .p(px(SPACE_XL))
        .flex()
        .flex_col()
        .id("profiles-scroll")
        .overflow_y_scroll()
        .child(page_heading(c, tr("ui.pages.profiles.title")))
        .child(
            div().flex().mb(px(SPACE_MD)).child(
                div()
                    .bg(c.accent_dim)
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_MD))
                    .py(px(SPACE_XS))
                    .text_color(c.accent)
                    .cursor(CursorStyle::PointingHand)
                    .child(tr("ui.pages.profiles.importFromUrl"))
                    .on_mouse_up(MouseButton::Left, {
                        let focus = import_focus.clone();
                        cx.listener(move |this, _e, w, cx| {
                            this.import_dialog_visible = true;
                            this.import_url.clear();
                            this.import_url_error = None;
                            w.focus(&focus, cx);
                            cx.notify();
                        })
                    }),
            ),
        )
        .child(match state.import_dialog_visible {
            true => import_dialog(c, state, &import_focus, cx).into_any_element(),
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
                        .child(div().child(tr("ui.pages.profiles.noProfiles")))
                        .child(
                            div()
                                .text_color(c.text_muted)
                                .mt(px(SPACE_SM))
                                .child(tr("ui.pages.profiles.noProfilesHint")),
                        ),
                )
                .into_any_element(),
            false => {
                let mut rows: Vec<gpui::AnyElement> = Vec::new();
                for p in &state.profile_previews {
                    let pending = state.pending_delete_uid.as_deref() == Some(p.uid.as_str());
                    rows.push(profile_row(c, p, pending, cx).into_any_element());
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

fn import_dialog(
    c: Colors,
    state: &AppState,
    focus: &FocusHandle,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let url = state.import_url.as_str();
    let error = state.import_url_error.clone();
    let placeholder_color = if url.is_empty() {
        c.text_muted
    } else {
        c.text_primary
    };
    let display: SharedString = if url.is_empty() {
        tr("ui.pages.profiles.urlPlaceholder")
    } else {
        SharedString::from(format!("{url}|"))
    };

    card(c).child(
        div()
            .id("import-dialog")
            .track_focus(focus)
            .key_context("ImportDialog")
            .on_key_down(cx.listener(handle_import_key_down))
            .flex()
            .flex_col()
            .child(
                div()
                    .text_color(c.text_primary)
                    .child(tr("ui.pages.profiles.importTitle")),
            )
            .child(
                div()
                    .mt(px(SPACE_SM))
                    .child(tr("ui.pages.profiles.subscriptionUrl")),
            )
            .child(
                div()
                    .mt(px(SPACE_XS))
                    .bg(c.input_bg)
                    .border_1()
                    .border_color(if error.is_some() {
                        c.danger
                    } else if url.is_empty() {
                        c.border
                    } else {
                        c.accent
                    })
                    .rounded(px(RADIUS_SM))
                    .px(px(SPACE_MD))
                    .py(px(SPACE_XS))
                    .text_color(placeholder_color)
                    .font_family(crate::fonts::mono_family())
                    .child(display),
            )
            .children(error.map(|msg| {
                div()
                    .mt(px(SPACE_XS))
                    .text_color(c.danger)
                    .child(SharedString::from(msg))
            }))
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
                            .child(tr("ui.pages.profiles.paste"))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new()
                                        && let Ok(text) = clipboard.get_text()
                                    {
                                        this.import_url = text;
                                        this.import_url_error = None;
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
                            .child(tr("ui.pages.profiles.import"))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    submit_import(this, cx);
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
                            .child(tr("ui.pages.profiles.cancel"))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _e, _w, cx| {
                                    this.import_dialog_visible = false;
                                    this.import_url_error = None;
                                    cx.notify();
                                }),
                            ),
                    ),
            ),
    )
}

fn handle_import_key_down(
    state: &mut AppState,
    event: &KeyDownEvent,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) {
    let ks = &event.keystroke;

    if ks.modifiers.platform && ks.key == "v" {
        if let Ok(mut clipboard) = arboard::Clipboard::new()
            && let Ok(text) = clipboard.get_text()
        {
            state.import_url.push_str(text.trim());
            state.import_url_error = None;
            cx.notify();
        }
        return;
    }

    match ks.key.as_str() {
        "enter" => submit_import(state, cx),
        "escape" => {
            state.import_dialog_visible = false;
            state.import_url_error = None;
            cx.notify();
        }
        "backspace" => {
            state.import_url.pop();
            state.import_url_error = None;
            cx.notify();
        }
        _ => {
            if let Some(ch) = ks.key_char.as_ref()
                && !ch.is_empty()
                && !ks.modifiers.platform
                && !ks.modifiers.control
                && !ks.modifiers.alt
            {
                state.import_url.push_str(ch);
                state.import_url_error = None;
                cx.notify();
            }
        }
    }
}

fn submit_import(state: &mut AppState, cx: &mut Context<AppState>) {
    let url = state.import_url.trim().to_string();
    if url.is_empty() {
        state.import_url_error =
            Some(zeroclash_i18n::translate("ui.pages.profiles.errors.emptyUrl").into_owned());
        cx.notify();
        return;
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        state.import_url_error =
            Some(zeroclash_i18n::translate("ui.pages.profiles.errors.invalidScheme").into_owned());
        state.log_viewer.store.push(
            LogLevel::Warn,
            "profile",
            &format!("rejected import URL: {url}"),
        );
        cx.notify();
        return;
    }
    state.push_command(UiCommand::ImportProfile(url));
    state.import_dialog_visible = false;
    state.import_url_error = None;
    cx.notify();
}

fn profile_row(
    c: Colors,
    p: &ProfilePreview,
    pending: bool,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let label = match p.itype.as_str() {
        "remote" => tr("ui.pages.profiles.typeRemote"),
        "local" => tr("ui.pages.profiles.typeLocal"),
        "merge" => tr("ui.pages.profiles.typeMerge"),
        "script" => tr("ui.pages.profiles.typeScript"),
        other => match other {
            "" => tr("ui.pages.profiles.typeProfile"),
            _ => tr("ui.pages.profiles.typeOther"),
        },
    };
    // `set_current` only makes sense for top-level profile types — merge,
    // script, rules, proxies, and groups items are referenced by the
    // active profile's chain rather than activated standalone.
    let activatable = matches!(p.itype.as_str(), "remote" | "local") && !p.is_current;

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
        .child(profile_meta(c, p, label))
        .child(profile_actions(c, p, pending, activatable, cx))
}

fn profile_meta(c: Colors, p: &ProfilePreview, label: SharedString) -> impl IntoElement {
    let mut info_line: Vec<gpui::AnyElement> = Vec::new();
    if let Some(url) = p.url.as_deref()
        && !url.is_empty()
    {
        info_line.push(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(truncate(url, 64)))
                .into_any_element(),
        );
    }
    if let Some(ts) = p.updated {
        let age = format_age(ts);
        info_line.push(
            div()
                .text_color(c.text_muted)
                .child(crate::i18n::tr_arg(
                    "ui.pages.profiles.updatedPrefix",
                    "age",
                    &age,
                ))
                .into_any_element(),
        );
    }

    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(SPACE_SM))
                .child(div().text_color(c.text_muted).child(label))
                .child(match p.is_current {
                    true => div()
                        .text_color(c.accent)
                        .child(tr("ui.pages.profiles.activeBadge"))
                        .into_any_element(),
                    false => div().into_any_element(),
                })
                .child(
                    div()
                        .text_color(c.text_primary)
                        .child(SharedString::from(&p.name)),
                ),
        )
        .when(!info_line.is_empty(), move |this| {
            this.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(SPACE_MD))
                    .children(info_line),
            )
        })
}

fn profile_actions(
    c: Colors,
    p: &ProfilePreview,
    pending: bool,
    activatable: bool,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let mut buttons: Vec<gpui::AnyElement> = Vec::new();

    if activatable {
        let uid_a = p.uid.clone();
        buttons.push(
            div()
                .bg(c.accent_dim)
                .rounded(px(RADIUS_SM))
                .px(px(SPACE_SM))
                .py(px(2.0))
                .text_color(c.accent)
                .cursor(CursorStyle::PointingHand)
                .child(tr("ui.pages.profiles.activate"))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.push_command(UiCommand::ActivateProfile(uid_a.clone()));
                        cx.notify();
                    }),
                )
                .into_any_element(),
        );
    }

    if pending {
        let uid_confirm = p.uid.clone();
        buttons.push(
            div()
                .bg(c.danger_dim)
                .rounded(px(RADIUS_SM))
                .px(px(SPACE_SM))
                .py(px(2.0))
                .text_color(c.danger)
                .cursor(CursorStyle::PointingHand)
                .child(tr("ui.pages.profiles.confirmDelete"))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.push_command(UiCommand::DeleteProfile(uid_confirm.clone()));
                        this.pending_delete_uid = None;
                        cx.notify();
                    }),
                )
                .into_any_element(),
        );
        buttons.push(
            div()
                .text_color(c.text_muted)
                .cursor(CursorStyle::PointingHand)
                .child(tr("ui.pages.profiles.cancel"))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.pending_delete_uid = None;
                        cx.notify();
                    }),
                )
                .into_any_element(),
        );
    } else if !p.is_current {
        let uid_d = p.uid.clone();
        buttons.push(
            div()
                .text_color(c.danger)
                .cursor(CursorStyle::PointingHand)
                .child(tr("ui.pages.profiles.delete"))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.pending_delete_uid = Some(uid_d.clone());
                        cx.notify();
                    }),
                )
                .into_any_element(),
        );
    } else {
        // Active profile is intentionally not deletable directly —
        // surface why instead of hiding the action entirely.
        buttons.push(
            div()
                .text_color(c.text_muted)
                .child(tr("ui.pages.profiles.activeMarker"))
                .into_any_element(),
        );
    }

    div().flex().gap(px(SPACE_SM)).children(buttons)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max - 1).collect();
    format!("{truncated}…")
}

fn format_age(unix_ts_secs: usize) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let then = unix_ts_secs as u64;
    if now <= then {
        return "just now".into();
    }
    let diff = now - then;
    if diff < 60 {
        format!("{diff}s ago")
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}
