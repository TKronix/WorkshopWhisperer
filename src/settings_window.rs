use egui::{Color32, RichText};
use std::collections::HashMap;

pub enum SettingsAction {
    None,
    SteamPathChanged,
}

pub struct SettingsWindow {
    pub open: bool,
    editing_colors: HashMap<String, Color32>,
    new_term: String,
    new_color: Color32,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            editing_colors: HashMap::new(),
            new_term: String::new(),
            new_color: Color32::WHITE,
        }
    }

    pub fn load_from(&mut self, status_colors: &crate::status_colors::StatusColors) {
        self.editing_colors = status_colors.known.clone();
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        user_path: &mut String,
        status_colors: &mut crate::status_colors::StatusColors,
    ) -> SettingsAction {
        let mut action = SettingsAction::None;

        if self.open {
            egui::Window::new("Settings")
                .open(&mut self.open)
                .resizable(true)
                .vscroll(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.heading("General");

                            ui.horizontal(|ui| {
                                ui.label("Steam Path:");
                                if ui.button("Change…").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        *user_path = path.display().to_string();
                                        action = SettingsAction::SteamPathChanged;
                                    }
                                }
                                if !user_path.is_empty() {
                                    ui.label(user_path.clone());
                                }
                            });

                            ui.separator();
                            ui.heading("Status Colors");

                            let mut to_remove: Option<String> = None;

                            egui::Grid::new("status_colors_grid")
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Status").strong());
                                    ui.label(RichText::new("Color").strong());
                                    ui.label(""); // delete column
                                    ui.end_row();

                                    for (term, color) in &mut self.editing_colors {
                                        ui.label(term);
                                        ui.color_edit_button_srgba(color);
                                        if ui.button("❌").clicked() {
                                            to_remove = Some(term.clone());
                                        }
                                        ui.end_row();
                                    }
                                });

                            if let Some(key) = to_remove {
                                self.editing_colors.remove(&key);
                            }

                            ui.separator();
                            ui.heading("Add New Status Color");

                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_term)
                                        .desired_width(160.0),
                                );
                                ui.color_edit_button_srgba(&mut self.new_color);

                                if ui.button("➕ Add Status").clicked() {
                                    if !self.new_term.is_empty() {
                                        self.editing_colors
                                            .insert(self.new_term.clone(), self.new_color);
                                        self.new_term.clear();
                                        self.new_color = Color32::WHITE;
                                    }
                                }
                            });

                            ui.add_space(12.0);

                            if ui.button(RichText::new("💾 Save Status Colors").strong()).clicked() {
                                status_colors.known = self.editing_colors.clone();
                                status_colors.save();
                            }

                            ui.separator();
                        });
                });
        }

        action
    }
}