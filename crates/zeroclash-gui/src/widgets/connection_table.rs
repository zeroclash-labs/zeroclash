//! Connection table with striped rows, selection, and detail panel.

use crate::design::{FONT_SM, FONT_XS, SPACE_SM, SPACE_XS, SPACE_XXS, palette};
use egui::{Color32, RichText, ScrollArea};
use zeroclash_core::connection::ConnEntry;

/// Render a connection table with striped rows and selection.
pub fn connection_table_ui(
    ui: &mut egui::Ui,
    connections: &[ConnEntry],
    selected_id: &mut Option<String>,
    mut on_close: impl FnMut(&str),
) {
    let c = palette(ui.ctx());
    let total = connections.len();
    ui.label(
        RichText::new(format!("Active Connections ({total})"))
            .size(16.0)
            .color(c.text_primary)
            .strong(),
    );
    ui.add_space(SPACE_SM);

    if connections.is_empty() {
        ui.label(
            RichText::new("No active connections")
                .size(FONT_SM)
                .color(c.text_muted),
        );
        return;
    }

    // Column widths
    let aw = ui.available_width();
    let col_host = aw * 0.26;
    let col_type = aw * 0.07;
    let col_chain = aw * 0.14;
    let col_rule = aw * 0.14;
    let col_speed = aw * 0.18;
    let col_act = aw * 0.12;

    // Column header
    let hdr_frame = egui::Frame::default()
        .fill(c.surface_alt)
        .inner_margin(egui::vec2(SPACE_SM, SPACE_XS));
    hdr_frame.show(ui, |ui| {
        ui.set_height(24.0);
        header_cell(ui, col_host, "Host", c);
        header_cell(ui, col_type, "Type", c);
        header_cell(ui, col_chain, "Chain", c);
        header_cell(ui, col_rule, "Rule", c);
        header_cell(ui, col_speed, "DL / UL", c);
        header_cell(ui, col_act, "", c);
    });
    ui.add_space(SPACE_XXS);

    // Connection rows
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .max_height(ui.available_height() - 4.0)
        .show(ui, |ui| {
            for (i, conn) in connections.iter().enumerate() {
                let is_selected = selected_id.as_deref() == Some(&conn.id);
                let row_bg = if is_selected {
                    c.accent_dim
                } else if i % 2 == 1 {
                    c.surface_alt
                } else {
                    Color32::TRANSPARENT
                };

                let resp = egui::Frame::default()
                    .fill(row_bg)
                    .inner_margin(egui::vec2(SPACE_SM, SPACE_XS))
                    .show(ui, |ui| {
                        ui.set_height(24.0);
                        ui.horizontal(|ui| {
                            // Host
                            ui.add_sized(
                                [col_host, 22.0],
                                egui::Label::new(
                                    RichText::new(&conn.host)
                                        .size(FONT_XS)
                                        .color(c.text_primary),
                                ),
                            );
                            // Type
                            ui.add_sized(
                                [col_type, 22.0],
                                egui::Label::new(
                                    RichText::new(&conn.network)
                                        .size(FONT_XS)
                                        .color(c.text_secondary),
                                ),
                            );
                            // Chain
                            let chain_str = if conn.chains.is_empty() {
                                "-".into()
                            } else {
                                conn.chains.join(" → ")
                            };
                            ui.add_sized(
                                [col_chain, 22.0],
                                egui::Label::new(
                                    RichText::new(chain_str).size(FONT_XS).color(c.text_muted),
                                ),
                            );
                            // Rule
                            ui.add_sized(
                                [col_rule, 22.0],
                                egui::Label::new(
                                    RichText::new(&conn.rule)
                                        .size(FONT_XS)
                                        .color(c.text_secondary),
                                ),
                            );
                            // Speed with colored indicators
                            ui.add_sized(
                                [col_speed, 22.0],
                                egui::Label::new(
                                    RichText::new(format!(
                                        "↓{}  ↑{}",
                                        format_speed(conn.download),
                                        format_speed(conn.upload)
                                    ))
                                    .size(FONT_XS),
                                ),
                            );
                            // Close button
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button(
                                            RichText::new("✕").size(FONT_XS).color(c.text_muted),
                                        )
                                        .on_hover_text("Close connection")
                                        .clicked()
                                    {
                                        on_close(&conn.id);
                                    }
                                },
                            );
                        });

                        // Click-to-select
                        let sel =
                            ui.interact(ui.max_rect(), ui.next_auto_id(), egui::Sense::click());
                        if sel.clicked() {
                            *selected_id = Some(conn.id.clone());
                        }
                    });

                // Hover highlight
                if resp.response.hovered() && !is_selected {
                    ui.painter()
                        .rect_filled(resp.response.rect, 0.0, c.surface_hover);
                }

                ui.add_space(1.0);
            }
        });

    // Detail panel for selected connection
    if let Some(ref sel_id) = selected_id.clone() {
        if let Some(conn) = connections.iter().find(|c| c.id == *sel_id) {
            ui.add_space(SPACE_SM);
            connection_detail_ui(ui, conn, c);
        }
    }
}

fn connection_detail_ui(ui: &mut egui::Ui, conn: &ConnEntry, c: &'static crate::design::Colors) {
    egui::Frame::default()
        .fill(c.surface)
        .stroke(egui::Stroke::new(1.0, c.border))
        .corner_radius(crate::design::RADIUS_MD)
        .inner_margin(egui::vec2(SPACE_SM + 4.0, SPACE_SM))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Connection Detail")
                    .size(14.0)
                    .color(c.text_primary)
                    .strong(),
            );
            ui.add_space(SPACE_XS);
            detail_row(ui, "Host", &conn.host, c);
            detail_row(ui, "Network", &conn.network, c);
            detail_row(ui, "Type", &conn.conn_type, c);
            detail_row(
                ui,
                "Source",
                &format!("{}:{}", conn.source_ip, conn.source_port),
                c,
            );
            detail_row(
                ui,
                "Destination",
                &format!("{}:{}", conn.destination_ip, conn.destination_port),
                c,
            );
            detail_row(ui, "Rule", &conn.rule, c);
            if !conn.rule_payload.is_empty() && conn.rule_payload != "-" {
                detail_row(ui, "Rule Payload", &conn.rule_payload, c);
            }
            if !conn.chains.is_empty() {
                detail_row(ui, "Chain", &conn.chains.join(" → "), c);
            }
            detail_row(ui, "DNS Mode", &conn.dns_mode, c);
            detail_row(ui, "Download", &format_bytes(conn.download), c);
            detail_row(ui, "Upload", &format_bytes(conn.upload), c);
            detail_row(ui, "Started", &conn.start, c);
        });
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str, c: &'static crate::design::Colors) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{label}:"))
                .size(FONT_XS)
                .color(c.text_muted),
        );
        ui.add_space(SPACE_XS);
        ui.label(RichText::new(value).size(FONT_XS).color(c.text_primary));
    });
}

fn header_cell(ui: &mut egui::Ui, width: f32, label: &str, c: &'static crate::design::Colors) {
    ui.add_sized(
        [width, 20.0],
        egui::Label::new(
            RichText::new(label)
                .size(FONT_XS)
                .color(c.text_muted)
                .strong(),
        ),
    );
}

fn format_speed(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
