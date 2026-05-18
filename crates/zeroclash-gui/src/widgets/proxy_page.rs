//! Proxy group display and selection UI.

use egui::{Color32, Frame, RichText, ScrollArea};

use zeroclash_core::mihomo::{ProxyGroup, Traffic};

/// Render a proxy group card with selector dropdown.
pub fn proxy_group_ui(
    ui: &mut egui::Ui,
    group: &ProxyGroup,
    on_select: &dyn Fn(&str, &str),
) {
    Frame::default()
        .corner_radius(6)
        .stroke(egui::Stroke::new(1.0, Color32::DARK_GRAY))
        .inner_margin(egui::vec2(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&group.name).strong().size(14.0));

                let type_color = match group.group_type.as_str() {
                    "Selector" | "select" => Color32::from_rgb(66, 133, 244),
                    "URLTest" | "url-test" => Color32::from_rgb(52, 168, 83),
                    "Fallback" | "fallback" => Color32::from_rgb(251, 188, 4),
                    "LoadBalance" | "load-balance" => Color32::from_rgb(234, 67, 53),
                    _ => Color32::GRAY,
                };
                ui.label(
                    RichText::new(format!("[{}]", group.group_type))
                        .color(type_color)
                        .size(11.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref now) = group.now {
                        ui.label(RichText::new(now).color(Color32::LIGHT_BLUE).size(12.0));
                    }

                    let current = group.now.clone().unwrap_or_default();
                    egui::ComboBox::from_id_salt(&group.name)
                        .selected_text(&current)
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            for proxy in &group.all {
                                if ui
                                    .selectable_label(
                                        Some(proxy) == group.now.as_ref(),
                                        proxy.as_str(),
                                    )
                                    .clicked()
                                {
                                    on_select(&group.name, proxy);
                                }
                            }
                        });
                });
            });

            if !group.history.is_empty() {
                let last = &group.history[group.history.len().saturating_sub(1)];
                let delay_ms = last.delay;
                let color = if delay_ms < 200 {
                    Color32::GREEN
                } else if delay_ms < 500 {
                    Color32::YELLOW
                } else {
                    Color32::RED
                };
                ui.label(RichText::new(format!("{delay_ms}ms")).color(color).size(12.0));
            }
        });
}

/// Render the full proxy page.
pub fn proxy_page_ui(
    ui: &mut egui::Ui,
    groups: &[ProxyGroup],
    traffic: Option<&Traffic>,
    on_select: &dyn Fn(&str, &str),
) {
    ui.heading("Proxies");

    if let Some(t) = traffic {
        ui.horizontal(|ui| {
            ui.label(format!("↑ {}", format_bytes(t.up as f64)));
            ui.label(format!("↓ {}", format_bytes(t.down as f64)));
        });
        ui.separator();
    }

    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        for group in groups {
            proxy_group_ui(ui, group, on_select);
            ui.add_space(4.0);
        }
    });
}

fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{bytes:.0} B")
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    }
}
