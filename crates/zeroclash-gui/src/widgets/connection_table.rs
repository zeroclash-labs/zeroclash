//! Virtual-scrolling connection table.

use egui::{Color32, RichText, ScrollArea};
use zeroclash_core::connection::ConnEntry;

/// Render a connection table with virtual scrolling support.
pub fn connection_table_ui(
    ui: &mut egui::Ui,
    connections: &[ConnEntry],
    selected_id: &mut Option<String>,
    mut on_close: impl FnMut(&str),
) {
    let total = connections.len();
    ui.heading(format!("Active Connections ({total})"));
    ui.separator();

    if connections.is_empty() {
        ui.label("No active connections");
        return;
    }

    // Column header
    let available_width = ui.available_width();
    let col_host = available_width * 0.28;
    let col_type = available_width * 0.08;
    let col_chain = available_width * 0.15;
    let col_rule = available_width * 0.15;
    let col_speed = available_width * 0.16;
    let col_act = available_width * 0.12;

    ui.horizontal(|ui| {
        ui.set_height(24.0);
        header_cell(ui, col_host, "Host");
        header_cell(ui, col_type, "Type");
        header_cell(ui, col_chain, "Chain");
        header_cell(ui, col_rule, "Rule");
        header_cell(ui, col_speed, "DL / UL");
        header_cell(ui, col_act, "");
    });
    ui.separator();

    // Connection rows
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .max_height(ui.available_height() - 4.0)
        .show(ui, |ui| {
            for conn in connections {
                let is_selected = selected_id.as_deref() == Some(&conn.id);
                let bg = if is_selected {
                    Color32::from_rgba_premultiplied(66, 133, 244, 40)
                } else {
                    Color32::TRANSPARENT
                };

                egui::Frame::default()
                    .fill(bg)
                    .inner_margin(egui::vec2(4.0, 2.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_height(22.0);

                            // Host cell
                            ui.add_sized(
                                [col_host, 22.0],
                                egui::Label::new(
                                    RichText::new(&conn.host).size(12.0),
                                ),
                            );

                            // Type cell
                            ui.add_sized(
                                [col_type, 22.0],
                                egui::Label::new(
                                    RichText::new(&conn.network).size(11.0),
                                ),
                            );

                            // Chain cell
                            let chain_str = if conn.chains.is_empty() {
                                "-".to_string()
                            } else {
                                conn.chains.join(" → ")
                            };
                            ui.add_sized(
                                [col_chain, 22.0],
                                egui::Label::new(
                                    RichText::new(chain_str).size(11.0).color(Color32::GRAY),
                                ),
                            );

                            // Rule cell
                            ui.add_sized(
                                [col_rule, 22.0],
                                egui::Label::new(
                                    RichText::new(&conn.rule).size(11.0),
                                ),
                            );

                            // Speed cell
                            ui.add_sized(
                                [col_speed, 22.0],
                                egui::Label::new(
                                    RichText::new(format!(
                                        "↓{} ↑{}",
                                        format_speed(conn.download),
                                        format_speed(conn.upload)
                                    ))
                                    .size(11.0),
                                ),
                            );

                            // Action cell
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("✕")
                                        .on_hover_text("Close connection")
                                        .clicked()
                                    {
                                        on_close(&conn.id);
                                    }
                                },
                            );
                        });

                        // Click to select for detail view
                        let resp = ui.interact(
                            ui.max_rect(),
                            ui.next_auto_id(),
                            egui::Sense::click(),
                        );
                        if resp.clicked() {
                            *selected_id = Some(conn.id.clone());
                        }
                    });

                ui.add_space(1.0);
            }
        });

    // Detail panel for selected connection
    if let Some(ref sel_id) = selected_id.clone() {
        if let Some(conn) = connections.iter().find(|c| c.id == *sel_id) {
            ui.separator();
            connection_detail_ui(ui, conn);
        }
    }
}

fn connection_detail_ui(ui: &mut egui::Ui, conn: &ConnEntry) {
    ui.heading("Connection Detail");
    ui.separator();

    detail_row(ui, "Host", &conn.host);
    detail_row(ui, "Network", &conn.network);
    detail_row(ui, "Type", &conn.conn_type);
    detail_row(ui, "Source", &format!("{}:{}", conn.source_ip, conn.source_port));
    detail_row(ui, "Destination", &format!("{}:{}", conn.destination_ip, conn.destination_port));
    detail_row(ui, "Rule", &conn.rule);
    detail_row(ui, "Rule Payload", &conn.rule_payload);
    if !conn.chains.is_empty() {
        detail_row(ui, "Chain", &conn.chains.join(" → "));
    }
    detail_row(ui, "DNS Mode", &conn.dns_mode);
    detail_row(ui, "Download", &format_bytes(conn.download));
    detail_row(ui, "Upload", &format_bytes(conn.upload));
    detail_row(ui, "Started", &conn.start);
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{label}:")).strong().size(12.0));
        ui.label(RichText::new(value).size(12.0));
    });
}

fn header_cell(ui: &mut egui::Ui, width: f32, label: &str) {
    ui.add_sized(
        [width, 20.0],
        egui::Label::new(RichText::new(label).strong().size(11.0)),
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
