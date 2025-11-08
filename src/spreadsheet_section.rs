use egui::Ui;
use crate::{app::GameEntry, spreadsheet};

pub struct SpreadsheetSection;

impl SpreadsheetSection {
    pub fn ui(game: &mut GameEntry, ui: &mut Ui) {
        egui::CollapsingHeader::new("Spreadsheet/CSV Data Settings")
            .default_open(false)
            .show(ui, |ui| {
                // Load from URL
                ui.horizontal(|ui| {
                    ui.label("Spreadsheet link:");
                    ui.text_edit_singleline(&mut game.spreadsheet.sheet_url);
                    if ui.button("Load").clicked() {
                        if let Some(csv_url) = spreadsheet::to_csv_url(&game.spreadsheet.sheet_url) {
                            game.sheet_data = spreadsheet::load_csv_from_url(&csv_url);
                            if game.spreadsheet.header_row_index.is_none() {
                                game.spreadsheet.id_col = None;
                                game.spreadsheet.status_col = None;
                                game.spreadsheet.name_col = None;
                            }
                            game.apply_spreadsheet();
                        }
                    }
                });

                // Load from file
                ui.horizontal(|ui| {
                    if ui.button("Choose CSV file…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("CSV files", &["csv"])
                            .pick_file()
                        {
                            if let Some(path_str) = path.to_str() {
                                game.spreadsheet.sheet_file = Some(path_str.to_string());
                                game.sheet_data = spreadsheet::load_csv_from_file(path_str);
                                if game.spreadsheet.header_row_index.is_none() {
                                    game.spreadsheet.id_col = None;
                                    game.spreadsheet.status_col = None;
                                    game.spreadsheet.name_col = None;
                                }
                                game.apply_spreadsheet();
                            }
                        }
                    }

                    if let Some(path) = &game.spreadsheet.sheet_file {
                        ui.label(format!("Loaded: {}", path));
                    }
                });

                // If spreadsheet data is loaded
                if let Some(rows) = &game.sheet_data {
                    if rows.is_empty() {
                        ui.label("Spreadsheet loaded but empty");
                    } else {
                        // 🔹 Track changes instead of mutating immediately
                        let mut header_changed = None;
                        let mut id_col_changed = None;
                        let mut status_col_changed = None;
                        let mut name_col_changed = None;

                        // Let user pick header row (show first 5 rows)
                        for (ri, row) in rows.iter().take(5).enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Row {}:", ri));
                                for cell in row {
                                    ui.label(cell);
                                }
                                if ui.button("Use as header").clicked() {
                                    header_changed = Some(ri);
                                }
                            });
                        }

                        // If header row chosen, show ComboBoxes
                        if let Some(header_idx) = game.spreadsheet.header_row_index {
                            let headers = &rows[header_idx];

                            egui::ComboBox::from_label("Mod ID column")
                                .selected_text(
                                    game.spreadsheet.id_col
                                        .and_then(|i| headers.get(i))
                                        .map_or("<select>", |s| s.as_str()),
                                )
                                .show_ui(ui, |ui| {
                                    for (ci, h) in headers.iter().enumerate() {
                                        if ui
                                            .selectable_label(game.spreadsheet.id_col == Some(ci), h)
                                            .clicked()
                                        {
                                            id_col_changed = Some(ci);
                                        }
                                    }
                                });

                            egui::ComboBox::from_label("Status column")
                                .selected_text(
                                    game.spreadsheet.status_col
                                        .and_then(|i| headers.get(i))
                                        .map_or("<select>", |s| s.as_str()),
                                )
                                .show_ui(ui, |ui| {
                                    for (ci, h) in headers.iter().enumerate() {
                                        if ui
                                            .selectable_label(
                                                game.spreadsheet.status_col == Some(ci),
                                                h,
                                            )
                                            .clicked()
                                        {
                                            status_col_changed = Some(ci);
                                        }
                                    }
                                });

                            egui::ComboBox::from_label("Name column (optional)")
                                .selected_text(
                                    game.spreadsheet.name_col
                                        .and_then(|i| headers.get(i))
                                        .map_or("<none>", |s| s.as_str()),
                                )
                                .show_ui(ui, |ui| {
                                    for (ci, h) in headers.iter().enumerate() {
                                        if ui
                                            .selectable_label(
                                                game.spreadsheet.name_col == Some(ci),
                                                h,
                                            )
                                            .clicked()
                                        {
                                            name_col_changed = Some(ci);
                                        }
                                    }
                                });
                        }

                        // 🔹 Apply changes after UI borrows are done
                        if let Some(new_header) = header_changed {
                            game.spreadsheet.header_row_index = Some(new_header);
                        }
                        if let Some(new_id) = id_col_changed {
                            game.spreadsheet.id_col = Some(new_id);
                        }
                        if let Some(new_status) = status_col_changed {
                            game.spreadsheet.status_col = Some(new_status);
                        }
                        if let Some(new_name) = name_col_changed {
                            game.spreadsheet.name_col = Some(new_name);
                        }

                        if header_changed.is_some()
                            || id_col_changed.is_some()
                            || status_col_changed.is_some()
                            || name_col_changed.is_some()
                        {
                            game.apply_spreadsheet();
                        }
                    }
                }
            });
    }
}