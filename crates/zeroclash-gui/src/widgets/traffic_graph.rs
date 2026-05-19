//! Real-time upload/download traffic display with visual bar indicators.

use crate::design::{FONT_SM, FONT_XS, SPACE_LG, SPACE_SM, SPACE_XS, palette};

#[derive(Default)]
pub struct TrafficHistory {
    pub upload_speed: f64,
    pub download_speed: f64,
    pub upload_total: u64,
    pub download_total: u64,
}

impl TrafficHistory {
    pub fn update(&mut self, up: u64, down: u64, dt: f64) {
        if dt > 0.0 {
            self.upload_speed = up as f64 / dt / 1024.0;
            self.download_speed = down as f64 / dt / 1024.0;
        }
        self.upload_total += up;
        self.download_total += down;
    }
    pub fn is_idle(&self) -> bool {
        self.upload_speed < 0.1 && self.download_speed < 0.1
    }
}

/// Render traffic bars with background tracks for visual context.
pub fn traffic_summary_ui(ui: &mut egui::Ui, history: &TrafficHistory) {
    let c = palette(ui.ctx());
    let bar_h = 16.0;
    let max_kbs = 10.0 * 1024.0; // 10 MB/s scale

    // Upload row
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("↑").size(14.0).color(c.accent));
        ui.add_space(SPACE_SM);
        // Background track
        let full_w = (ui.available_width() - 80.0).max(4.0);
        let (track_rect, _) =
            ui.allocate_exact_size(egui::vec2(full_w, bar_h), egui::Sense::hover());
        ui.painter().rect_filled(track_rect, 3.0, c.surface_alt);
        let frac = (history.upload_speed / max_kbs).min(1.0) as f32;
        if frac > 0.001 {
            let bar_rect = egui::Rect::from_min_size(
                track_rect.left_top(),
                egui::vec2(track_rect.width() * frac, bar_h),
            );
            ui.painter().rect_filled(bar_rect, 3.0, c.accent);
        }
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(format_speed(history.upload_speed))
                .size(FONT_SM)
                .color(c.text_secondary),
        );
    });

    ui.add_space(SPACE_SM);

    // Download row
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("↓").size(14.0).color(c.success));
        ui.add_space(SPACE_SM);
        let full_w = (ui.available_width() - 80.0).max(4.0);
        let (track_rect, _) =
            ui.allocate_exact_size(egui::vec2(full_w, bar_h), egui::Sense::hover());
        ui.painter().rect_filled(track_rect, 3.0, c.surface_alt);
        let frac = (history.download_speed / max_kbs).min(1.0) as f32;
        if frac > 0.001 {
            let bar_rect = egui::Rect::from_min_size(
                track_rect.left_top(),
                egui::vec2(track_rect.width() * frac, bar_h),
            );
            ui.painter().rect_filled(bar_rect, 3.0, c.success);
        }
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(format_speed(history.download_speed))
                .size(FONT_SM)
                .color(c.text_secondary),
        );
    });

    ui.add_space(SPACE_SM);

    // Activity indicator
    let idle = history.is_idle();
    ui.horizontal(|ui| {
        let dot_color = if idle { c.text_muted } else { c.success };
        let label = if idle { "Idle" } else { "Active" };
        ui.label(
            egui::RichText::new(format!("● {label}"))
                .size(FONT_XS)
                .color(dot_color),
        );
        ui.add_space(SPACE_LG);
        ui.label(
            egui::RichText::new(format!(
                "Total  ↑ {}  ↓ {}",
                format_bytes(history.upload_total as f64),
                format_bytes(history.download_total as f64)
            ))
            .size(FONT_XS)
            .color(c.text_muted),
        );
    });
}

fn format_speed(kbs: f64) -> String {
    if kbs < 0.1 {
        "0 KB/s".into()
    } else if kbs < 1024.0 {
        format!("{kbs:.1} KB/s")
    } else {
        format!("{:.1} MB/s", kbs / 1024.0)
    }
}

fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{bytes:.0} B")
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else if bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    }
}
