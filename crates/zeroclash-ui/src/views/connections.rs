use gpui::{Context, CursorStyle, MouseButton, SharedString, Window, div, prelude::*, px};
use zeroclash_core::ConnEntry;

use crate::design::{Colors, RADIUS_SM, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SPACE_XXS};
use crate::state::{AppState, UiCommand};
use crate::theme::Theme;

pub fn connections_page(state: &AppState, _w: &mut Window, cx: &mut Context<AppState>) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;
    let total = state.connections.len();

    div().size_full().p(px(SPACE_XL)).flex().flex_col()
        .child(div().text_color(c.text_primary).child(SharedString::from(format!("Active Connections ({total})"))))
        .child(match total == 0 {
            true => div().text_color(c.text_muted).child("No active connections").into_any_element(),
            false => {
                let mut rows: Vec<gpui::AnyElement> = Vec::new();
                rows.push(table_header(c).into_any_element());
                for (i, conn) in state.connections.iter().enumerate() {
                    let is_sel = state.selected_conn_id.as_deref() == Some(&conn.id);
                    let bg = if is_sel { c.accent_dim } else if i % 2 == 1 { c.surface_alt } else { gpui::transparent_black() };
                    let cid = conn.id.clone();
                    let cid2 = conn.id.clone();
                    rows.push(
                        div().bg(bg).px(px(SPACE_SM)).py(px(SPACE_XXS)).flex().items_center().gap(px(SPACE_SM))
                            .cursor(CursorStyle::PointingHand)
                            .on_mouse_up(MouseButton::Left, cx.listener(move |this, _e, _w, cx| {
                                this.selected_conn_id = Some(cid.clone());
                                cx.notify();
                            }))
                            .child(div().flex_1().text_color(c.text_primary).child(SharedString::from(&conn.host)))
                            .child(div().w(px(50.)).text_color(c.text_secondary).child(SharedString::from(&conn.network)))
                            .child(div().w(px(100.)).text_color(c.text_muted).child(SharedString::from(match conn.chains.is_empty() { true=>"-".into(), false=>conn.chains.join(" → ") })))
                            .child(div().w(px(80.)).text_color(c.text_secondary).child(SharedString::from(&conn.rule)))
                            .child(div().w(px(120.)).child(SharedString::from(format!("↓{} ↑{}", fmt_speed(conn.download), fmt_speed(conn.upload)))))
                            .child(div().text_color(c.danger).cursor(CursorStyle::PointingHand).child("✕")
                                .on_mouse_up(MouseButton::Left, cx.listener(move |this, _e, _w, cx| { this.push_command(UiCommand::CloseConnection(cid2.clone())); cx.notify(); })))
                            .into_any_element(),
                    );
                }
                if let Some(conn) = state.connections.iter().find(|c| c.id == state.selected_conn_id.as_deref().unwrap_or_default()) {
                    rows.push(conn_detail(c, conn).into_any_element());
                }
                div().flex_1().flex().flex_col().children(rows).into_any_element()
            }
        })
}

fn table_header(c: Colors) -> impl IntoElement {
    div().bg(c.surface_alt).px(px(SPACE_SM)).py(px(SPACE_XXS)).flex().items_center().gap(px(SPACE_SM))
        .child(div().flex_1().text_color(c.text_muted).child("Host"))
        .child(div().w(px(50.)).text_color(c.text_muted).child("Type"))
        .child(div().w(px(100.)).text_color(c.text_muted).child("Chain"))
        .child(div().w(px(80.)).text_color(c.text_muted).child("Rule"))
        .child(div().w(px(120.)).text_color(c.text_muted).child("DL / UL"))
        .child(div().text_color(c.text_muted).child(""))
}

fn conn_detail(c: Colors, conn: &ConnEntry) -> impl IntoElement {
    div().mt(px(SPACE_MD)).bg(c.surface).border_1().border_color(c.border).rounded(px(RADIUS_SM)).p(px(SPACE_MD)).flex().flex_col()
        .child(div().text_color(c.text_primary).child("Connection Detail"))
        .child(div().mt(px(SPACE_XS)).flex().gap(px(SPACE_XS))
            .child(div().text_color(c.text_muted).child(SharedString::from("Host:")))
            .child(div().text_color(c.text_primary).child(SharedString::from(&conn.host))))
        .child(div().flex().gap(px(SPACE_XS))
            .child(div().text_color(c.text_muted).child(SharedString::from("Network:")))
            .child(div().text_color(c.text_primary).child(SharedString::from(&conn.network))))
        .child(div().flex().gap(px(SPACE_XS))
            .child(div().text_color(c.text_muted).child(SharedString::from("Source:")))
            .child(div().text_color(c.text_primary).child(SharedString::from(format!("{}:{}", conn.source_ip, conn.source_port)))))
        .child(div().flex().gap(px(SPACE_XS))
            .child(div().text_color(c.text_muted).child(SharedString::from("Dest:")))
            .child(div().text_color(c.text_primary).child(SharedString::from(format!("{}:{}", conn.destination_ip, conn.destination_port)))))
        .child(div().flex().gap(px(SPACE_XS))
            .child(div().text_color(c.text_muted).child(SharedString::from("Rule:")))
            .child(div().text_color(c.text_primary).child(SharedString::from(&conn.rule))))
}

fn fmt_speed(bytes: u64) -> String {
    if bytes < 1024 { format!("{bytes}B") }
    else if bytes < 1048576 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1073741824 { format!("{:.1}M", bytes as f64 / 1048576.0) }
    else { format!("{:.1}G", bytes as f64 / 1073741824.0) }
}
