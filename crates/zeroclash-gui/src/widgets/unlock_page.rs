//! Media unlock checker page UI.

use egui::{Color32, Frame, RichText};
use zeroclash_core::media_unlock::{UnlockResult, UnlockStatus, check_all};

/// Renders the media unlock checker page.
pub fn unlock_page_ui(ui: &mut egui::Ui, results: &mut Vec<UnlockResult>, checking: &mut bool) {
    ui.heading("Media Unlock Checker");
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Run Check").clicked() && !*checking {
            *checking = true;
            // We need to run this async; the actual check is triggered by the command system
        }
    });
    ui.separator();

    if results.is_empty() {
        ui.label("Click 'Run Check' to test media service accessibility via the current proxy.");
        return;
    }

    // Results grid
    egui::Grid::new("unlock_grid")
        .min_col_width(120.0)
        .striped(true)
        .show(ui, |ui| {
            ui.label(RichText::new("Service").strong());
            ui.label(RichText::new("Status").strong());
            ui.label(RichText::new("Region").strong());
            ui.end_row();

            for result in results.iter() {
                ui.label(format!("{} {}", result.icon, result.service));

                let status_color = match result.status {
                    UnlockStatus::Unlocked => Color32::GREEN,
                    UnlockStatus::Locked => Color32::RED,
                    UnlockStatus::Failed => Color32::YELLOW,
                    UnlockStatus::Checking => Color32::GRAY,
                };
                ui.label(
                    RichText::new(result.status.as_str())
                        .color(status_color)
                        .strong(),
                );

                ui.label(
                    result
                        .region
                        .as_deref()
                        .unwrap_or("-"),
                );
                ui.end_row();
            }
        });
}

/// Trigger an async media unlock check and return results.
pub async fn run_unlock_check() -> Vec<UnlockResult> {
    // Use localhost proxy for checking
    check_all("http://127.0.0.1:7899").await
}
