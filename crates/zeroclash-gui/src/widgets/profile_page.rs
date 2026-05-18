//! Profile management page — redesigned with design tokens.

use egui::{RichText, ScrollArea};
use zeroclash_core::profile::ProfilePreview;
use crate::design::{SPACE_LG, SPACE_MD, SPACE_SM, card_frame, page_heading, palette};

pub struct ImportDialog {
    pub url: String,
    pub visible: bool,
}

impl ImportDialog {
    pub fn new() -> Self { Self { url: String::new(), visible: false } }
    pub fn show(&self) -> bool { self.visible }
    pub fn open(&mut self) { self.visible = true; self.url.clear(); }
    pub fn close(&mut self) { self.visible = false; self.url.clear(); }
}

pub fn profile_page_ui(
    ui: &mut egui::Ui, previews: &[ProfilePreview],
    import_dialog: &mut ImportDialog,
    mut on_activate: impl FnMut(&str),
    mut on_delete: impl FnMut(&str),
    on_import_url: impl FnOnce(&str),
) {
    let c = palette(ui.ctx());
    page_heading(ui, "Profiles");
    ui.add_space(SPACE_LG);

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("➕ Import from URL").clicked() { import_dialog.open(); }
    });
    ui.add_space(SPACE_MD);

    // Import dialog
    if import_dialog.show() {
        let mut url = import_dialog.url.clone();
        let mut import_clicked = false;
        card_frame(ui).show(ui, |ui| {
            ui.label(RichText::new("Import Profile from URL").size(14.0).color(c.text_primary).strong());
            ui.add_space(SPACE_SM);
            ui.text_edit_singleline(&mut url);
            ui.horizontal(|ui| {
                if ui.button("Import").clicked() { import_clicked = true; }
                if ui.button("Cancel").clicked() { import_dialog.close(); }
            });
        });
        import_dialog.url = url;
        if import_clicked { let u = import_dialog.url.clone(); on_import_url(&u); import_dialog.close(); }
        ui.add_space(SPACE_MD);
    }

    // Profile list
    if previews.is_empty() {
        card_frame(ui).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("📋").size(48.0));
                ui.label(RichText::new("No profiles yet").size(14.0).color(c.text_secondary));
                ui.label(RichText::new("Import a subscription URL to get started").size(12.0).color(c.text_muted));
            });
        });
        return;
    }

    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        for preview in previews {
            let mut activate = false;
            let mut delete = false;
            profile_card(ui, preview, &mut activate, &mut delete);
            if activate { on_activate(&preview.uid); }
            if delete { on_delete(&preview.uid); }
            ui.add_space(SPACE_SM);
        }
    });
}

fn profile_card(ui: &mut egui::Ui, p: &ProfilePreview, activate: &mut bool, delete: &mut bool) {
    let c = palette(ui.ctx());
    let border = if p.is_current { c.accent } else { c.border };
    egui::Frame::default()
        .fill(c.surface).rounding(design::RADIUS_MD)
        .stroke(egui::Stroke::new(if p.is_current { 2.0 } else { 1.0 }, border))
        .inner_margin(egui::vec2(SPACE_MD + 4.0, SPACE_SM + 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (emoji, color) = match p.itype.as_str() {
                    "remote" => ("☁", c.accent), "local" => ("💻", c.warning),
                    "merge" => ("🔀", c.text_muted), "script" => ("📜", c.success),
                    _ => ("📄", c.text_muted),
                };
                ui.label(RichText::new(emoji).size(16.0));
                ui.add_space(SPACE_SM);
                if p.is_current { ui.label(RichText::new("●").color(c.accent).size(12.0)); ui.add_space(SPACE_SM); }
                ui.label(RichText::new(&p.name).size(14.0).color(c.text_primary).strong());
                ui.add_space(SPACE_SM);
                ui.label(RichText::new(&p.itype).size(11.0).color(color));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").clicked() { *delete = true; }
                    if !p.is_current && ui.small_button("Activate").clicked() { *activate = true; }
                });
            });
            if let Some(ref url) = p.url { ui.label(RichText::new(url).size(11.0).color(c.text_muted)); }
        });
}

use crate::design;
