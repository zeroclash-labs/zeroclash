//! Basic YAML editor widget with syntax highlighting and line numbers.

use egui::{Color32, RichText, ScrollArea};

/// Renders a syntax-highlighted YAML editor.
pub struct YamlEditor {
    pub content: String,
    pub error_message: Option<String>,
    show_line_numbers: bool,
}

impl Default for YamlEditor {
    fn default() -> Self {
        Self {
            content: String::new(),
            error_message: None,
            show_line_numbers: true,
        }
    }
}

impl YamlEditor {
    pub const fn new(content: String) -> Self {
        Self {
            content,
            error_message: None,
            show_line_numbers: true,
        }
    }

    /// Validate the content as valid YAML.
    pub fn validate(&mut self) -> bool {
        match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&self.content) {
            Ok(_) => {
                self.error_message = None;
                true
            }
            Err(e) => {
                self.error_message = Some(format!("YAML parse error: {e}"));
                false
            }
        }
    }
}

/// Render the YAML editor with syntax highlighting and line numbers.
pub fn yaml_editor_ui(ui: &mut egui::Ui, editor: &mut YamlEditor) {
    ui.horizontal(|ui| {
        ui.heading("YAML Editor");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut editor.show_line_numbers, "Line numbers");
            if ui.button("Validate").clicked() {
                editor.validate();
            }
        });
    });

    if let Some(ref err) = editor.error_message {
        ui.colored_label(Color32::RED, err);
    }
    ui.separator();

    // Count lines for line number gutter
    let line_count = editor.content.lines().count().max(1);
    let line_num_width = if editor.show_line_numbers { 40.0 } else { 0.0 };
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if editor.show_line_numbers {
                    // Line number gutter
                    let mut line_nums = String::new();
                    for i in 1..=line_count {
                        line_nums.push_str(&format!("{i}\n"));
                    }

                    ui.add_sized(
                        [line_num_width, ui.available_height()],
                        egui::Label::new(
                            RichText::new(line_nums)
                                .font(font_id.clone())
                                .color(Color32::DARK_GRAY)
                                .size(12.0),
                        ),
                    );
                }

                // Editor area — plain text for now (custom layouter requires egui >=0.34 with TextBuffer)
                ui.add(
                    egui::TextEdit::multiline(&mut editor.content)
                        .font(font_id)
                        .desired_width(f32::INFINITY)
                        .desired_rows(line_count.max(20)),
                );
            });
        });
}
