use egui::{Color32, Ui};
use egui_extras::{TableBuilder, Column};

use crate::app::{GameEntry, format_timestamp};
use crate::status_colors::StatusColors;

pub struct ModsTable;

impl ModsTable {
    pub fn ui(
        game: &GameEntry,
        ui: &mut Ui,
        sort_by_id_asc: &mut bool,
        status_colors: &StatusColors, // 🔹 pass in reference
    ) {
        if game.mods.is_empty() {
            ui.label("No active Workshop mods found.");
            return;
        }

        // Make a sorted copy
        let mut mods_sorted = game.mods.clone();
        mods_sorted.sort_by(|a, b| a.id.cmp(&b.id));
        if !*sort_by_id_asc {
            mods_sorted.reverse();
        }

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .drag_to_scroll(true)
            .column(Column::initial(80.0))   // ID
            .column(Column::remainder())    // Name
            .column(Column::initial(100.0)) // Status
            .column(Column::remainder())    // Updated
            .column(Column::initial(120.0)) // State
            .header(20.0, |mut header| {
                header.col(|ui| {
                    if ui.button("ID").clicked() {
                        *sort_by_id_asc = !*sort_by_id_asc;
                    }
                });
                header.col(|ui| { ui.heading("Name"); });
                header.col(|ui| { ui.heading("CSV Status"); });
                header.col(|ui| { ui.heading("Last Updated"); });
                header.col(|ui| { ui.heading("State"); });
            })
            .body(|mut body| {
                for m in &mods_sorted {
                    body.row(20.0, |mut row| {
                        // ID with hyperlink
                        row.col(|ui| {
                            let url = format!(
                                "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                                m.id
                            );
                            let _ = ui.hyperlink_to(&m.id, url);
                        });

                        // Name
                        row.col(|ui| {
                            let _ = ui.label(m.name.as_deref().unwrap_or("<unknown>"));
                        });

                        // CSV Status with color
                        row.col(|ui| {
                            if let Some(status) = &m.status {
                                let color = status_colors.color_for(status);
                                let _ = ui.colored_label(color, status);
                            } else {
                                let _ = ui.label("?");
                            }
                        });

                        // Last Updated
                        row.col(|ui| {
                            let _ = ui.label(format_timestamp(m.last_updated.as_deref()));
                        });

                        // State (outdated check)
                        row.col(|ui| {
                            if let Some(outdated) = m.is_outdated() {
                                if outdated {
                                    let _ = ui.colored_label(Color32::RED, "⚠️ Outdated");
                                } else {
                                    let _ = ui.colored_label(Color32::GREEN, "✅ Up to date");
                                }
                            } else {
                                let _ = ui.label("-");
                            }
                        });
                    });
                }
            });
    }
}