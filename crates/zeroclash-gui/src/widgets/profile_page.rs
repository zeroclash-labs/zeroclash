//! Profile / subscription management page.

use egui::{Color32, Frame, RichText, ScrollArea};
use zeroclash_core::profile::ProfilePreview;

/// Import dialog state (handled externally to avoid borrow conflicts).
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

/// Renders the full profiles management page.
pub fn profile_page_ui(
    ui: &mut egui::Ui,
    previews: &[ProfilePreview],
    import_dialog: &mut ImportDialog,
    mut on_activate: impl FnMut(&str),
    mut on_delete: impl FnMut(&str),
    on_import_url: impl FnOnce(&str),
) {
    ui.heading("Profiles");
    ui.separator();

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("➕ Import from URL").clicked() {
            import_dialog.open();
        }
    });
    ui.separator();

    // Import dialog
    if import_dialog.show() {
        let mut url = import_dialog.url.clone();
        let mut import_clicked = false;

        Frame::default()
            .corner_radius(6)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(66, 133, 244)))
            .inner_margin(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.label("Import Profile from URL:");
                ui.text_edit_singleline(&mut url);
                ui.horizontal(|ui| {
                    if ui.button("Import").clicked() {
                        import_clicked = true;
                    }
                    if ui.button("Cancel").clicked() {
                        import_dialog.close();
                    }
                });
            });

        import_dialog.url = url;
        if import_clicked {
            let url_str = import_dialog.url.clone();
            on_import_url(&url_str);
            import_dialog.close();
        }
        ui.separator();
    }

    // Profile list
    if previews.is_empty() {
        ui.label(RichText::new("No profiles yet. Click 'Import from URL' to add one.").color(Color32::GRAY));
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for preview in previews {
                let mut activate = false;
                let mut delete = false;

                profile_card_ui(ui, preview, &mut activate, &mut delete);

                if activate {
                    on_activate(&preview.uid);
                }
                if delete {
                    on_delete(&preview.uid);
                }
                ui.add_space(4.0);
            }
        });
}

fn profile_card_ui(
    ui: &mut egui::Ui,
    preview: &ProfilePreview,
    activate: &mut bool,
    delete: &mut bool,
) {
    Frame::default()
        .corner_radius(6)
        .stroke(egui::Stroke::new(1.0, if preview.is_current {
            Color32::from_rgb(66, 133, 244)
        } else {
            Color32::DARK_GRAY
        }))
        .inner_margin(egui::vec2(10.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let type_color = match preview.itype.as_str() {
                    "remote" => Color32::from_rgb(52, 168, 83),
                    "local" => Color32::from_rgb(251, 188, 4),
                    "merge" => Color32::GRAY,
                    "script" => Color32::from_rgb(156, 39, 176),
                    _ => Color32::DARK_GRAY,
                };
                ui.label(
                    RichText::new(format!("[{}]", preview.itype))
                        .color(type_color)
                        .size(11.0),
                );

                if preview.is_current {
                    ui.label(RichText::new("● Active").color(Color32::from_rgb(66, 133, 244)).size(11.0));
                }

                ui.label(RichText::new(&preview.name).strong().size(14.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑").on_hover_text("Delete").clicked() {
                        *delete = true;
                    }
                    if !preview.is_current {
                        if ui.button("Activate").clicked() {
                            *activate = true;
                        }
                    }
                });
            });

            if let Some(ref url) = preview.url {
                ui.label(RichText::new(url).color(Color32::GRAY).size(11.0));
            }
            if let Some(ts) = preview.updated {
                ui.label(RichText::new(format!("Updated: {ts}")).color(Color32::DARK_GRAY).size(10.0));
            }
        });
}
