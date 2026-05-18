//! Proxy group display and selection UI — redesigned with design tokens.

use egui::{RichText, ScrollArea};
use zeroclash_core::mihomo::ProxyGroup;
use crate::design::{SPACE_SM, SPACE_MD, SPACE_LG, card_frame, page_heading, palette};

pub fn proxy_page_ui(
    ui: &mut egui::Ui, groups: &[ProxyGroup],
    _traffic: Option<&zeroclash_core::mihomo::Traffic>,
    on_select: &dyn Fn(&str, &str),
) {
    let c = palette(ui.ctx());
    page_heading(ui, "Proxies");
    ui.add_space(SPACE_LG);

    if groups.is_empty() {
        card_frame(ui).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🌐").size(48.0));
                ui.add_space(SPACE_SM);
                ui.label(RichText::new("No proxy groups available").size(14.0).color(c.text_secondary));
                ui.label(RichText::new("Start the core to load proxy data").size(12.0).color(c.text_muted));
            });
        });
        return;
    }

    // Search
    let mut filter = String::new();
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(14.0));
        ui.add(egui::TextEdit::singleline(&mut filter).hint_text("Filter groups...").desired_width(200.0));
    });
    ui.add_space(SPACE_MD);

    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        let filtered: Vec<&ProxyGroup> = groups.iter().filter(|g| filter.is_empty() || g.name.to_lowercase().contains(&filter.to_lowercase())).collect();
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
            ui.label(RichText::new(&group.name).size(15.0).color(c.text_primary).strong());
            ui.add_space(SPACE_SM);

            // Type label
            ui.label(RichText::new(&group.group_type).size(11.0).color(badge_color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Current selection
                if let Some(ref now) = group.now {
                    ui.label(RichText::new(format!("▶ {now}")).size(13.0).color(c.accent));
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

        // Delay history
        if !group.history.is_empty() {
            ui.add_space(SPACE_SM);
            let last = &group.history[group.history.len().saturating_sub(1)];
            let delay = last.delay;
            let (color, icon) = if delay < 200 { (c.success, "🟢") } else if delay < 500 { (c.warning, "🟡") } else { (c.danger, "🔴") };
            ui.label(RichText::new(format!("{icon} {delay}ms")).size(12.0).color(color));
        }
    });
}
