//! Profile management page.

use crate::design::{
    FONT_SM, FONT_XS, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, card_frame, page_heading, palette,
};
use egui::{RichText, ScrollArea};
use zeroclash_core::profile::ProfilePreview;

pub struct ImportDialog {
    pub url: String,
    pub visible: bool,
}

impl ImportDialog {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            visible: false,
        }
    }
    pub fn show(&self) -> bool {
        self.visible
    }
    pub fn open(&mut self) {
        self.visible = true;
        self.url.clear();
    }
    pub fn close(&mut self) {
        self.visible = false;
        self.url.clear();
    }
}

pub fn profile_page_ui(
    ui: &mut egui::Ui,
    previews: &[ProfilePreview],
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
        let btn_resp = egui::Frame::default()
            .fill(c.accent_dim)
            .corner_radius(crate::design::RADIUS_SM)
            .inner_margin(egui::vec2(SPACE_MD, SPACE_XS + 2.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("➕").size(FONT_SM));
                    ui.label(
                        RichText::new("Import from URL")
                            .size(FONT_SM)
                            .color(c.accent),
                    );
                });
            });
        if btn_resp.response.clicked() {
            import_dialog.open();
        }
    });
    ui.add_space(SPACE_MD);

    // Import dialog
    if import_dialog.show() {
        let mut url = import_dialog.url.clone();
        let mut import_clicked = false;
        card_frame(ui).show(ui, |ui| {
            ui.label(
                RichText::new("Import Profile from URL")
                    .size(14.0)
                    .color(c.text_primary)
                    .strong(),
            );
            ui.add_space(SPACE_SM);
            ui.text_edit_singleline(&mut url);
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                let import_btn = egui::Frame::default()
                    .fill(c.success_dim)
                    .corner_radius(crate::design::RADIUS_SM)
                    .inner_margin(egui::vec2(SPACE_MD, SPACE_XS))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Import").size(FONT_SM).color(c.success));
                    });
                if import_btn.response.clicked() {
                    import_clicked = true;
                }
                if ui.button(RichText::new("Cancel").size(FONT_SM)).clicked() {
                    import_dialog.close();
                }
            });
        });
        import_dialog.url = url;
        if import_clicked {
            let u = import_dialog.url.clone();
            on_import_url(&u);
            import_dialog.close();
        }
        ui.add_space(SPACE_MD);
    }

    // Profile list
    if previews.is_empty() {
        card_frame(ui).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_LG);
                ui.label(RichText::new("📋").size(48.0));
                ui.add_space(SPACE_SM);
                ui.label(
                    RichText::new("No profiles yet")
                        .size(14.0)
                        .color(c.text_secondary),
                );
                ui.label(
                    RichText::new("Import a subscription URL to get started")
                        .size(FONT_SM)
                        .color(c.text_muted),
                );
                ui.add_space(SPACE_LG);
            });
        });
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for preview in previews {
                let mut activate = false;
                let mut delete = false;
                profile_card(ui, preview, &mut activate, &mut delete);
                if activate {
                    on_activate(&preview.uid);
                }
                if delete {
                    on_delete(&preview.uid);
                }
                ui.add_space(SPACE_SM);
            }
        });
}

fn profile_card(ui: &mut egui::Ui, p: &ProfilePreview, activate: &mut bool, delete: &mut bool) {
    let c = palette(ui.ctx());
    let border_color = if p.is_current { c.accent } else { c.border };
    let border_w = if p.is_current { 2.0 } else { 1.0 };

    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(0.0), 44.0),
        egui::Sense::click(),
    );

    // Card background
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect, crate::design::RADIUS_MD, c.surface_hover);
    } else {
        ui.painter()
            .rect_filled(rect, crate::design::RADIUS_MD, c.surface);
    }

    // Border with accent for active
    ui.painter().rect_stroke(
        rect,
        crate::design::RADIUS_MD,
        egui::Stroke::new(border_w, border_color),
        egui::StrokeKind::Middle,
    );

    // Content
    let inner = rect.shrink2(egui::vec2(SPACE_MD + 4.0, SPACE_SM + 2.0));
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // Type emoji
    let (emoji, color) = match p.itype.as_str() {
        "remote" => ("☁", c.accent),
        "local" => ("💻", c.warning),
        "merge" => ("🔀", c.text_muted),
        "script" => ("📜", c.success),
        _ => ("📄", c.text_muted),
    };
    child_ui.label(RichText::new(emoji).size(16.0));
    child_ui.add_space(SPACE_SM);

    // Active indicator
    if p.is_current {
        child_ui.label(RichText::new("●").color(c.accent).size(12.0));
        child_ui.add_space(SPACE_SM);
    }

    // Name
    child_ui.label(
        RichText::new(&p.name)
            .size(14.0)
            .color(c.text_primary)
            .strong(),
    );
    child_ui.add_space(SPACE_SM);

    // Type badge
    child_ui.label(RichText::new(&p.itype).size(FONT_XS).color(color));

    // Spacer
    child_ui.add_space(SPACE_SM);

    // Action buttons on the right
    child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui
            .small_button(RichText::new("🗑").size(FONT_SM))
            .on_hover_text("Delete profile")
            .clicked()
        {
            *delete = true;
        }
        if !p.is_current {
            ui.add_space(SPACE_XS);
            let act_btn = egui::Frame::default()
                .fill(c.accent_dim)
                .corner_radius(crate::design::RADIUS_SM)
                .inner_margin(egui::vec2(SPACE_SM, SPACE_XS))
                .show(ui, |ui| {
                    ui.label(RichText::new("Activate").size(FONT_XS).color(c.accent));
                });
            if act_btn.response.clicked() {
                *activate = true;
            }
        }
    });

    // URL line
    if let Some(ref url) = p.url {
        let inner2 = rect.shrink2(egui::vec2(SPACE_MD + 4.0, SPACE_SM + 2.0));
        // Paint URL text below the main row
        ui.painter().text(
            egui::pos2(inner2.left(), inner2.bottom() - 12.0),
            egui::Align2::LEFT_TOP,
            url,
            egui::FontId::proportional(FONT_XS),
            c.text_muted,
        );
    }
}
