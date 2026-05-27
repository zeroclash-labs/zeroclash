use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::design::{
    Colors, FONT_MONO_FAMILY, RADIUS_SM, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SPACE_XXS,
};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;
use crate::util::CachedConn;

pub fn connections_page(
    state: &AppState,
    _w: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;
    let total = state.connections.len();

    div()
        .size_full()
        .p(px(SPACE_XL))
        .flex()
        .flex_col()
        .id("connections-scroll")
        .overflow_y_scroll()
        .child(
            div()
                .text_color(c.text_primary)
                .child(SharedString::from(format!("Active Connections ({total})"))),
        )
        .child(match total == 0 {
            true => div()
                .text_color(c.text_muted)
                .child("No active connections")
                .into_any_element(),
            false => connection_table(c, state, cx),
        })
}

fn connection_table(c: Colors, state: &AppState, cx: &mut Context<AppState>) -> gpui::AnyElement {
    let mut rows: Vec<gpui::AnyElement> = Vec::with_capacity(state.connections.len() + 2);
    rows.push(table_header(c).into_any_element());

    for (i, conn) in state.connections.iter().enumerate() {
        let is_sel = state.selected_conn_id.as_deref() == Some(&conn.entry.id);
        let bg = if is_sel {
            c.accent_dim
        } else if i.is_multiple_of(2) {
            gpui::transparent_black()
        } else {
            c.surface_alt
        };
        rows.push(connection_row(c, conn, bg, cx).into_any_element());
    }

    if let Some(selected) = state
        .selected_conn_id
        .as_deref()
        .and_then(|id| state.connections.iter().find(|c| c.entry.id == id))
    {
        rows.push(conn_detail(c, selected).into_any_element());
    }

    div()
        .flex_1()
        .flex()
        .flex_col()
        .children(rows)
        .into_any_element()
}

fn connection_row(
    c: Colors,
    conn: &CachedConn,
    bg: gpui::Hsla,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let select_id = conn.entry.id.clone();
    let close_id = conn.entry.id.clone();
    div()
        .id(SharedString::from(format!("conn-{}", conn.entry.id)))
        .bg(bg)
        .px(px(SPACE_SM))
        .py(px(SPACE_XXS))
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .cursor(CursorStyle::PointingHand)
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _e, _w, cx| {
                this.selected_conn_id = Some(select_id.clone());
                cx.notify();
            }),
        )
        .child(
            div()
                .flex_1()
                .text_color(c.text_primary)
                .child(SharedString::from(&conn.entry.host)),
        )
        .child(
            div()
                .w(px(50.))
                .text_color(c.text_secondary)
                .child(SharedString::from(&conn.entry.network)),
        )
        .child(
            div()
                .w(px(100.))
                .text_color(c.text_muted)
                .child(conn.chain_text.clone()),
        )
        .child(
            div()
                .w(px(80.))
                .text_color(c.text_secondary)
                .child(SharedString::from(&conn.entry.rule)),
        )
        .child(
            div()
                .w(px(120.))
                .font_family(FONT_MONO_FAMILY)
                .child(conn.speed_text.clone()),
        )
        .child(
            div()
                .text_color(c.danger)
                .cursor(CursorStyle::PointingHand)
                .child("✕")
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _e, _w, cx| {
                        this.push_command(UiCommand::CloseConnection(close_id.clone()));
                        cx.notify();
                    }),
                ),
        )
}

fn table_header(c: Colors) -> impl IntoElement {
    div()
        .bg(c.surface_alt)
        .px(px(SPACE_SM))
        .py(px(SPACE_XXS))
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .child(div().flex_1().text_color(c.text_muted).child("Host"))
        .child(div().w(px(50.)).text_color(c.text_muted).child("Type"))
        .child(div().w(px(100.)).text_color(c.text_muted).child("Chain"))
        .child(div().w(px(80.)).text_color(c.text_muted).child("Rule"))
        .child(div().w(px(120.)).text_color(c.text_muted).child("DL / UL"))
        .child(div().text_color(c.text_muted).child(""))
}

fn conn_detail(c: Colors, conn: &CachedConn) -> impl IntoElement {
    div()
        .mt(px(SPACE_MD))
        .bg(c.surface)
        .border_1()
        .border_color(c.border)
        .rounded(px(RADIUS_SM))
        .p(px(SPACE_MD))
        .flex()
        .flex_col()
        .child(div().text_color(c.text_primary).child("Connection Detail"))
        .child(detail_row(c, "Host:", SharedString::from(&conn.entry.host)))
        .child(detail_row(
            c,
            "Network:",
            SharedString::from(&conn.entry.network),
        ))
        .child(detail_row_mono(c, "Source:", conn.source_addr.clone()))
        .child(detail_row_mono(c, "Dest:", conn.dest_addr.clone()))
        .child(detail_row(c, "Rule:", SharedString::from(&conn.entry.rule)))
}

fn detail_row(c: Colors, label: &str, value: SharedString) -> impl IntoElement {
    div()
        .flex()
        .gap(px(SPACE_XS))
        .mt(px(SPACE_XS))
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(label.to_string())),
        )
        .child(div().text_color(c.text_primary).child(value))
}

fn detail_row_mono(c: Colors, label: &str, value: SharedString) -> impl IntoElement {
    div()
        .flex()
        .gap(px(SPACE_XS))
        .mt(px(SPACE_XS))
        .child(
            div()
                .text_color(c.text_muted)
                .child(SharedString::from(label.to_string())),
        )
        .child(
            div()
                .text_color(c.text_primary)
                .font_family(FONT_MONO_FAMILY)
                .child(value),
        )
}
