//! Settings page — redesigned with design tokens.

use egui::{RichText, ScrollArea};
use zeroclash_core::config::VergeConfig;
use crate::design::{SPACE_LG, SPACE_MD, SPACE_SM, card_frame, page_heading, palette};

pub fn settings_page_ui(
    ui: &mut egui::Ui, config: &mut VergeConfig,
    on_save: &mut dyn FnMut(&VergeConfig),
    on_toggle_system_proxy: &mut dyn FnMut(),
    on_toggle_auto_start: &mut dyn FnMut(),
) {
    let c = palette(ui.ctx());
    page_heading(ui, "Settings");
    ui.add_space(SPACE_LG);

    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        // Appearance
        card_frame(ui).show(ui, |ui| {
            ui.label(RichText::new("Appearance").size(15.0).color(c.text_primary).strong());
            ui.separator();
            ui.add_space(SPACE_SM);

            ui.horizontal(|ui| {
                ui.label("Language:");
                let mut idx: usize = if config.language == "zh" { 0 } else { 1 };
                egui::ComboBox::from_id_salt("lang").selected_text(&config.language).show_ui(ui, |ui| {
                    ui.selectable_value(&mut idx, 0, "zh");
                    ui.selectable_value(&mut idx, 1, "en");
                });
                config.language = if idx == 0 { "zh".into() } else { "en".into() };
            });

            ui.horizontal(|ui| {
                ui.label("Theme:");
                let themes = ["system", "light", "dark"];
                let mut idx = themes.iter().position(|&t| t == config.theme_mode).unwrap_or(0);
                egui::ComboBox::from_id_salt("theme").selected_text(config.theme_mode.as_str()).show_ui(ui, |ui| {
                    for (i, name) in themes.iter().enumerate() { ui.selectable_value(&mut idx, i, *name); }
                });
                config.theme_mode = themes[idx].into();
            });
        });

        ui.add_space(SPACE_MD);

        // Ports
        card_frame(ui).show(ui, |ui| {
            ui.label(RichText::new("Proxy Ports").size(15.0).color(c.text_primary).strong());
            ui.separator();
            ui.add_space(SPACE_SM);
            port_row(ui, "HTTP", &mut config.http_port);
            port_row(ui, "SOCKS", &mut config.socks_port);
            port_row(ui, "Mixed", &mut config.mixed_port);
        });

        ui.add_space(SPACE_MD);

        // System
        card_frame(ui).show(ui, |ui| {
            ui.label(RichText::new("System").size(15.0).color(c.text_primary).strong());
            ui.separator();
            ui.add_space(SPACE_SM);
            toggle_row(ui, "System Proxy", &mut config.enable_system_proxy);
            if ui.button("Apply System Proxy").clicked() { on_toggle_system_proxy(); }
            toggle_row(ui, "TUN Mode", &mut config.enable_tun);
            toggle_row(ui, "Auto Launch", &mut config.enable_auto_launch);
            if ui.button("Toggle Auto Start").clicked() { on_toggle_auto_start(); }
        });

        ui.add_space(SPACE_LG);
        if ui.button(egui::RichText::new("  Save Settings  ").size(14.0)).clicked() {
            on_save(config);
        }
    });
}

fn port_row(ui: &mut egui::Ui, label: &str, val: &mut u16) {
    ui.horizontal(|ui| {
        ui.label(format!("{label} Port:"));
        let mut v = *val as u32;
        ui.add(egui::DragValue::new(&mut v).range(1..=65535));
        *val = v as u16;
    });
}

fn toggle_row(ui: &mut egui::Ui, label: &str, val: &mut bool) {
    ui.horizontal(|ui| {
        ui.checkbox(val, label);
    });
}
