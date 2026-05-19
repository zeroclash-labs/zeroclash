//! Settings page with grouped sections and design tokens.

use crate::design::{
    FONT_MD, FONT_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SPACE_XXS, page_heading,
    palette,
};
use egui::{RichText, ScrollArea};
use zeroclash_core::config::VergeConfig;

pub fn settings_page_ui(
    ui: &mut egui::Ui,
    config: &mut VergeConfig,
    on_save: &mut dyn FnMut(&VergeConfig),
    on_toggle_system_proxy: &mut dyn FnMut(),
    on_toggle_auto_start: &mut dyn FnMut(),
) {
    let c = palette(ui.ctx());
    page_heading(ui, "Settings");
    ui.add_space(SPACE_LG);

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // ── Appearance ──
            settings_section(ui, "🎨", "Appearance", c, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Language")
                            .size(FONT_SM)
                            .color(c.text_secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut idx: usize = if config.language == "zh" { 0 } else { 1 };
                        egui::ComboBox::from_id_salt("lang")
                            .selected_text(&config.language)
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut idx, 0, "zh");
                                ui.selectable_value(&mut idx, 1, "en");
                            });
                        config.language = if idx == 0 { "zh".into() } else { "en".into() };
                    });
                });
                ui.add_space(SPACE_XXS);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Theme").size(FONT_SM).color(c.text_secondary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let themes = ["system", "light", "dark"];
                        let mut idx = themes
                            .iter()
                            .position(|&t| t == config.theme_mode)
                            .unwrap_or(0);
                        egui::ComboBox::from_id_salt("theme")
                            .selected_text(config.theme_mode.as_str())
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                for (i, name) in themes.iter().enumerate() {
                                    ui.selectable_value(&mut idx, i, *name);
                                }
                            });
                        config.theme_mode = themes[idx].into();
                    });
                });
            });

            ui.add_space(SPACE_MD);

            // ── Proxy Ports ──
            settings_section(ui, "🔌", "Proxy Ports", c, |ui| {
                port_row(ui, "HTTP", &mut config.http_port, c);
                ui.add_space(SPACE_XXS);
                port_row(ui, "SOCKS", &mut config.socks_port, c);
                ui.add_space(SPACE_XXS);
                port_row(ui, "Mixed", &mut config.mixed_port, c);
            });

            ui.add_space(SPACE_MD);

            // ── System ──
            settings_section(ui, "⚙", "System", c, |ui| {
                toggle_row(ui, "System Proxy", &mut config.enable_system_proxy, c);
                if config.enable_system_proxy {
                    ui.add_space(SPACE_XS);
                    if ui
                        .small_button(RichText::new("Apply System Proxy").size(FONT_SM))
                        .clicked()
                    {
                        on_toggle_system_proxy();
                    }
                }
                ui.add_space(SPACE_XXS);
                toggle_row(ui, "TUN Mode", &mut config.enable_tun, c);
                ui.add_space(SPACE_XXS);
                toggle_row(ui, "Auto Launch", &mut config.enable_auto_launch, c);
                if ui
                    .small_button(RichText::new("Toggle Auto Start").size(FONT_SM))
                    .clicked()
                {
                    on_toggle_auto_start();
                }
            });

            ui.add_space(SPACE_LG);

            // Save button — prominent
            let save_bg = if c.accent.a() > 0 {
                c.accent_dim
            } else {
                c.accent
            };
            let save_resp = egui::Frame::default()
                .fill(save_bg)
                .corner_radius(crate::design::RADIUS_SM)
                .inner_margin(egui::vec2(SPACE_XL, SPACE_SM + 4.0))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Save Settings")
                                .size(FONT_MD)
                                .color(c.accent)
                                .strong(),
                        );
                    });
                });
            if save_resp.response.clicked() {
                on_save(config);
            }
        });
}

/// Render a settings section with icon and accent left border.
fn settings_section(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    c: &'static crate::design::Colors,
    body: impl FnOnce(&mut egui::Ui),
) {
    let outer_margin = ui.ctx().global_style().spacing.item_spacing.y * 0.5;
    let frame = egui::Frame::default()
        .fill(c.surface)
        .corner_radius(crate::design::RADIUS_MD)
        .stroke(egui::Stroke::new(1.0, c.border))
        .inner_margin(egui::vec2(SPACE_LG, SPACE_MD))
        .outer_margin(egui::vec2(0.0, outer_margin));

    frame.show(ui, |ui| {
        // Accent left border
        let full_rect = ui.max_rect();
        let bar =
            egui::Rect::from_min_size(full_rect.left_top(), egui::vec2(3.0, full_rect.height()));
        ui.painter().rect_filled(bar, 1.5, c.accent);

        // Section header
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(FONT_MD));
            ui.add_space(SPACE_XS);
            ui.label(
                RichText::new(title)
                    .size(FONT_MD)
                    .color(c.text_primary)
                    .strong(),
            );
        });
        ui.add_space(SPACE_MD);
        body(ui);
    });
}

fn port_row(ui: &mut egui::Ui, label: &str, val: &mut u16, c: &'static crate::design::Colors) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{label} Port"))
                .size(FONT_SM)
                .color(c.text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut v = *val as u32;
            ui.add(
                egui::DragValue::new(&mut v)
                    .range(1..=65535)
                    .update_while_editing(false),
            );
            *val = v as u16;
        });
    });
}

fn toggle_row(ui: &mut egui::Ui, label: &str, val: &mut bool, c: &'static crate::design::Colors) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(FONT_SM).color(c.text_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(val, "");
        });
    });
}
