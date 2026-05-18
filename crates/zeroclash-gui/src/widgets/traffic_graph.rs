//! Real-time upload/download traffic display using native egui rendering.
//!
//! Uses simple progress bars and text since egui_plot has a version conflict
//! (depends on egui 0.33 while we target 0.34).

/// Stores traffic sample history for display.
#[derive(Default)]
pub struct TrafficHistory {
    pub upload_speed: f64,   // KB/s
    pub download_speed: f64, // KB/s
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

/// Render a traffic display bar.
pub fn traffic_bar_ui(ui: &mut egui::Ui, label: &str, speed_kbs: f64, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(11.0));
        ui.add_space(4.0);

        let max_width = 200.0;
        // Cap display at ~10 MB/s for reasonable bar width
        let fraction = (speed_kbs / (10.0 * 1024.0)).min(1.0) as f32;

        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(max_width * fraction, 12.0), egui::Sense::hover());

        let painter = ui.painter();
        painter.rect_filled(rect, egui::CornerRadius::same(3), color);

        ui.label(egui::RichText::new(format_speed(speed_kbs)).size(11.0));
    });
}

/// Render a full traffic summary.
pub fn traffic_summary_ui(ui: &mut egui::Ui, history: &TrafficHistory) {
    egui::Frame::default()
        .inner_margin(egui::vec2(8.0, 4.0))
        .show(ui, |ui| {
            traffic_bar_ui(
                ui,
                "↑",
                history.upload_speed,
                egui::Color32::from_rgb(66, 165, 245),
            );
            traffic_bar_ui(
                ui,
                "↓",
                history.download_speed,
                egui::Color32::from_rgb(76, 175, 80),
            );
            ui.separator();
            ui.label(format!(
                "Total: ↑ {} ↓ {}",
                format_bytes(history.upload_total as f64),
                format_bytes(history.download_total as f64)
            ));
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
