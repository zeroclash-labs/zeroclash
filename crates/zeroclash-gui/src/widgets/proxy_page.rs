//! Proxy group display and selection UI.

use crate::design::{
    FONT_MD, FONT_SM, FONT_XS, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, card_frame, page_heading,
    palette,
};
use egui::{RichText, ScrollArea};
use zeroclash_core::mihomo::ProxyGroup;

pub fn proxy_page_ui(
    ui: &mut egui::Ui,
    groups: &[ProxyGroup],
    _traffic: Option<&zeroclash_core::mihomo::Traffic>,
    on_select: &dyn Fn(&str, &str),
) {
    let c = palette(ui.ctx());
    page_heading(ui, "Proxies");
    ui.add_space(SPACE_LG);

    if groups.is_empty() {
        card_frame(ui).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_LG);
                ui.label(RichText::new("🌐").size(48.0));
                ui.add_space(SPACE_SM);
                ui.label(
                    RichText::new("No proxy groups available")
                        .size(FONT_MD)
                        .color(c.text_secondary),
                );
                ui.label(
                    RichText::new("Start the core to load proxy data")
                        .size(FONT_SM)
                        .color(c.text_muted),
                );
                ui.add_space(SPACE_LG);
            });
        });
        return;
    }

    // Search bar
    let mut filter = String::new();
    egui::Frame::default()
        .fill(c.input_bg)
        .corner_radius(crate::design::RADIUS_SM)
        .inner_margin(egui::vec2(SPACE_MD, SPACE_XS + 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(FONT_SM));
                ui.add(
                    egui::TextEdit::singleline(&mut filter)
                        .hint_text("Filter groups...")
                        .desired_width(ui.available_width() - 40.0),
                );
            });
        });
    ui.add_space(SPACE_MD);

    // Group cards
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let filtered: Vec<&ProxyGroup> = groups
                .iter()
                .filter(|g| {
                    filter.is_empty() || g.name.to_lowercase().contains(&filter.to_lowercase())
                })
                .collect();

            if filtered.is_empty() && !filter.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(SPACE_MD);
                    ui.label(
                        RichText::new("No matching groups")
                            .size(FONT_SM)
                            .color(c.text_muted),
                    );
                });
                return;
            }

            for group in &filtered {
                proxy_group_card(ui, group, on_select);
                ui.add_space(SPACE_SM);
            }
        });
}

fn proxy_group_card(ui: &mut egui::Ui, group: &ProxyGroup, on_select: &dyn Fn(&str, &str)) {
    let c = palette(ui.ctx());
    card_frame(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            // Type badge
            let (emoji, badge_color) = match group.group_type.as_str() {
                "Selector" | "select" => ("🎯", c.accent),
                "URLTest" | "url-test" => ("⚡", c.success),
                "Fallback" | "fallback" => ("🔄", c.warning),
                "LoadBalance" | "load-balance" => ("⚖", c.danger),
                _ => ("📡", c.text_muted),
            };
            ui.label(RichText::new(emoji).size(16.0));
            ui.add_space(SPACE_SM);

            // Group name
            ui.label(
                RichText::new(&group.name)
                    .size(15.0)
                    .color(c.text_primary)
                    .strong(),
            );
            ui.add_space(SPACE_SM);

            // Type label
            ui.label(
                RichText::new(&group.group_type)
                    .size(FONT_XS)
                    .color(badge_color),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Current selection
                if let Some(ref now) = group.now {
                    ui.label(
                        RichText::new(format!("▶ {now}"))
                            .size(FONT_SM)
                            .color(c.accent),
                    );
                    ui.add_space(SPACE_SM);
                }

                // Selector dropdown
                if !group.all.is_empty() {
                    let current = group.now.clone().unwrap_or_default();
                    egui::ComboBox::from_id_salt(&group.name)
                        .selected_text(current)
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for proxy in &group.all {
                                let sel = Some(proxy) == group.now.as_ref();
                                if ui.selectable_label(sel, proxy.as_str()).clicked() {
                                    on_select(&group.name, proxy);
                                }
                            }
                        });
                }
            });
        });

        // Delay indicator — visual bar
        if !group.history.is_empty() {
            ui.add_space(SPACE_SM);
            let last = &group.history[group.history.len().saturating_sub(1)];
            let delay = last.delay;
            let (color, icon) = if delay < 200 {
                (c.success, "🟢")
            } else if delay < 500 {
                (c.warning, "🟡")
            } else {
                (c.danger, "🔴")
            };

            ui.horizontal(|ui| {
                // Mini delay bar
                let frac = (delay.min(1000) as f32) / 1000.0;
                let bar_w = 80.0 * (1.0 - frac) + 4.0;
                let (track_rect, _) =
                    ui.allocate_exact_size(egui::vec2(80.0, 6.0), egui::Sense::hover());
                ui.painter().rect_filled(track_rect, 3.0, c.surface_alt);
                let bar_rect =
                    egui::Rect::from_min_size(track_rect.left_top(), egui::vec2(bar_w, 6.0));
                ui.painter().rect_filled(bar_rect, 3.0, color);

                ui.add_space(SPACE_XS);
                ui.label(
                    RichText::new(format!("{icon} {delay}ms"))
                        .size(FONT_XS)
                        .color(color),
                );
            });
        }
    });
}
