//! Settings configuration page.

use egui::{Color32, Frame, RichText, ScrollArea};
use zeroclash_core::config::VergeConfig;

/// Render the full settings page.
pub fn settings_page_ui(
    ui: &mut egui::Ui,
    config: &mut VergeConfig,
    on_save: &mut dyn FnMut(&VergeConfig),
) {
    ui.heading("Settings");
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // ── Appearance ──
            settings_section(ui, "Appearance", |ui| {
                // Language
                ui.horizontal(|ui| {
                    ui.label("Language:");
                    let mut lang_idx = if config.language == "zh" { 0 } else { 1 };
                    egui::ComboBox::from_id_salt("lang")
                        .selected_text(&config.language)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut lang_idx, 0, "zh");
                            ui.selectable_value(&mut lang_idx, 1, "en");
                        });
                    if lang_idx == 0 {
                        config.language = "zh".into();
                    } else {
                        config.language = "en".into();
                    }
                });

                // Theme
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let themes = ["system", "light", "dark"];
                    let mut idx = themes.iter().position(|&t| t == config.theme_mode).unwrap_or(0);
                    egui::ComboBox::from_id_salt("theme")
                        .selected_text(config.theme_mode.as_str())
                        .show_ui(ui, |ui| {
                            for (i, name) in themes.iter().enumerate() {
                                ui.selectable_value(&mut idx, i, *name);
                            }
                        });
                    config.theme_mode = themes[idx].into();
                });
            });

            // ── Proxy Ports ──
            settings_section(ui, "Proxy Ports", |ui| {
                port_field(ui, "HTTP Port:", &mut config.http_port);
                port_field(ui, "SOCKS Port:", &mut config.socks_port);
                port_field(ui, "Mixed Port:", &mut config.mixed_port);
            });

            // ── System ──
            settings_section(ui, "System", |ui| {
                checkbox_field(ui, "System Proxy", &mut config.enable_system_proxy);
                checkbox_field(ui, "TUN Mode", &mut config.enable_tun);
                checkbox_field(ui, "Auto Launch", &mut config.enable_auto_launch);
            });

            ui.separator();

            // Save button
            if ui.button("Save Settings").clicked() {
                on_save(config);
            }
        });
}

fn settings_section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    Frame::default()
        .corner_radius(6)
        .stroke(egui::Stroke::new(1.0, Color32::DARK_GRAY))
        .inner_margin(egui::vec2(12.0, 8.0))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().size(14.0));
            ui.separator();
            body(ui);
        });
    ui.add_space(8.0);
}

fn port_field(ui: &mut egui::Ui, label: &str, value: &mut u16) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut val = *value as u32;
        if ui.add(egui::DragValue::new(&mut val).range(1..=65535)).changed() {
            *value = val as u16;
        }
    });
}

fn checkbox_field(ui: &mut egui::Ui, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.checkbox(value, label);
    });
}
