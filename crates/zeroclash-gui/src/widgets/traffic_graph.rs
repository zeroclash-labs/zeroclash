//! Real-time upload/download traffic display — redesigned with gradient progress bars.

use crate::design::{SPACE_SM, SPACE_XS, palette};

#[derive(Default)]
pub struct TrafficHistory {
    pub upload_speed: f64,
    pub download_speed: f64,
    pub upload_total: u64,
    pub download_total: u64,
}

impl TrafficHistory {
    pub fn update(&mut self, up: u64, down: u64, dt: f64) {
        if dt > 0.0 { self.upload_speed = up as f64 / dt / 1024.0; self.download_speed = down as f64 / dt / 1024.0; }
        self.upload_total += up;
        self.download_total += down;
    }
    pub fn is_idle(&self) -> bool { self.upload_speed < 0.1 && self.download_speed < 0.1 }
}

/// Render traffic bars in a card.
pub fn traffic_summary_ui(ui: &mut egui::Ui, history: &TrafficHistory) {
    let c = palette(ui.ctx());

    // Upload bar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("↑").size(14.0).color(c.accent));
        ui.add_space(SPACE_SM);
        let frac = (history.upload_speed / (10.0 * 1024.0)).min(1.0) as f32;
        let bar_w = ui.available_width() * frac;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 14.0), egui::Sense::hover());
        // Gradient bar: accent
        ui.painter().rect_filled(rect, 4.0, c.accent);
        ui.label(egui::RichText::new(format_speed(history.upload_speed)).size(12.0).color(c.text_secondary));
    });

    ui.add_space(SPACE_SM);

    // Download bar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("↓").size(14.0).color(c.success));
        ui.add_space(SPACE_SM);
        let frac = (history.download_speed / (10.0 * 1024.0)).min(1.0) as f32;
        let bar_w = ui.available_width() * frac;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 14.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, c.success);
        ui.label(egui::RichText::new(format_speed(history.download_speed)).size(12.0).color(c.text_secondary));
    });

    ui.add_space(SPACE_XS);
    ui.separator();

    // Totals
    ui.label(egui::RichText::new(format!("Total: ↑ {}  ↓ {}", format_bytes(history.upload_total as f64), format_bytes(history.download_total as f64))).size(11.0).color(c.text_muted));
}

fn format_speed(kbs: f64) -> String {
    if kbs < 0.1 { "0 KB/s".into() } else if kbs < 1024.0 { format!("{kbs:.1} KB/s") } else { format!("{:.1} MB/s", kbs / 1024.0) }
}

fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 { format!("{bytes:.0} B") } else if bytes < 1024.0 * 1024.0 { format!("{:.1} KB", bytes / 1024.0) } else if bytes < 1024.0 * 1024.0 * 1024.0 { format!("{:.1} MB", bytes / (1024.0 * 1024.0)) } else { format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0)) }
}
