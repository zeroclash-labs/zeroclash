use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::components::settings_group::{info_row, settings_section, toggle_row};
use crate::design::{RADIUS_SM, SPACE_LG, SPACE_SM, SPACE_XL};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn settings_page(
    state: &AppState,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;

    div()
        .size_full()
        .p(px(SPACE_XL))
        .child(page_heading(c, "Settings"))
        .child(settings_section(c, "Appearance").child(
            div()
                .flex()
                .flex_col()
                .child(info_row(c, "Theme", &state.theme_mode()))
                .child(info_row(c, "Language", "English")),
        ))
        .child(settings_section(c, "System").child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .cursor(CursorStyle::PointingHand)
                        .on_mouse_up(MouseButton::Left, cx.listener(move |this, _e, _w, cx| {
                            this.push_command(UiCommand::ToggleSystemProxy);
                            cx.notify();
                        }))
                        .child(toggle_row(c, "System Proxy", state.enable_system_proxy)),
                )
                .child(
                    div()
                        .cursor(CursorStyle::PointingHand)
                        .on_mouse_up(MouseButton::Left, cx.listener(move |this, _e, _w, cx| {
                            this.push_command(UiCommand::ToggleAutoStart);
                            cx.notify();
                        }))
                        .child(toggle_row(c, "Auto Start", false)),
                ),
        ))
        .child(
            div()
                .mt(px(SPACE_LG))
                .bg(c.accent)
                .rounded(px(RADIUS_SM))
                .px(px(SPACE_LG))
                .py(px(SPACE_SM))
                .text_color(gpui::white())
                .cursor(CursorStyle::PointingHand)
                .child(SharedString::from("Save Settings")),
        )
}

impl AppState {
    fn theme_mode(&self) -> String {
        "System".into()
    }
}
