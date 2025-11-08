use eframe::egui;
use std::{collections::HashMap, sync::{mpsc::{Sender, Receiver}, Arc}};
use tokio::runtime::Runtime;

use crate::steam;
use crate::config::{GeneralConfig, save_general_config, load_general_config, SpreadsheetConfig, save_spreadsheet_configs, load_spreadsheet_configs};
use crate::spreadsheet;
use crate::mods_table::ModsTable;
use crate::spreadsheet_section::SpreadsheetSection;
use crate::status_colors::StatusColors;
use crate::settings_window::{SettingsWindow, SettingsAction};

#[derive(Clone, Debug)]
pub struct WorkshopMod {
    pub id: String,
    pub name: Option<String>,   
    pub status: Option<String>, 
    pub last_updated: Option<String>,
    pub local_updated: Option<String>,
}

impl WorkshopMod {
    pub fn is_outdated(&self) -> Option<bool> {
        let local = self.local_updated.as_ref()?.parse::<u64>().ok()?;
        let remote = self.last_updated.as_ref()?.parse::<u64>().ok()?;
        Some(local < remote)
    }
}

pub struct GameEntry {
    pub appid: String,
    pub name: String,
    pub path: String,

    pub spreadsheet: SpreadsheetConfig,
    pub sheet_data: Option<Vec<Vec<String>>>,
    pub mods: Vec<WorkshopMod>,
}

pub struct MyApp {
    pub games: Option<Vec<GameEntry>>,
    pub user_path: String,
    pub selected: Option<usize>,
    pub tx: Sender<(String, String, String)>, // (mod_id, title, updated)
    pub rx: Receiver<(String, String, String)>,
    pub rt: Arc<Runtime>,
    pub fetching_all: bool,
    pub progress: Option<(usize, usize)>, // (done, total)
    pub sort_by_id_asc: bool,
    pub show_all_games: bool,
    pub status_colors: StatusColors,
    pub settings: SettingsWindow,
}

impl GameEntry {
    pub fn apply_spreadsheet(&mut self) {
        if let Some(rows) = &self.sheet_data {
            if let (Some(header_idx), Some(id_col), Some(status_col)) =
                (self.spreadsheet.header_row_index, self.spreadsheet.id_col, self.spreadsheet.status_col)
            {
                if header_idx < rows.len() {
                    let mut status_map = HashMap::new();
                    let mut name_map = HashMap::new();

                    for row in rows.iter().skip(header_idx + 1) {
                        if let Some(id) = row.get(id_col) {
                            if let Some(status) = row.get(status_col) {
                                status_map.insert(id.clone(), status.clone());
                            }
                            if let Some(name_col) = self.spreadsheet.name_col {
                                if let Some(name) = row.get(name_col) {
                                    name_map.insert(id.clone(), name.clone());
                                }
                            }
                        }
                    }

                    for m in &mut self.mods {
                        if let Some(status) = status_map.get(&m.id) {
                            m.status = Some(status.clone());
                        }
                        if let Some(name) = name_map.get(&m.id) {
                            if m.name.is_none() {
                                m.name = Some(name.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

impl MyApp {
    pub fn reload_games(&mut self) {
        let configs = load_spreadsheet_configs();

        self.games = steam::get_installed_games(&self.user_path)
            .map(|list| {
                list.into_iter()
                    .map(|(appid, name, path)| {
                        let local_mods = steam::get_active_mods(&path, &appid);

                        let mods = local_mods.into_iter()
                            .map(|(id, local_time)| WorkshopMod {
                                id,
                                name: None,
                                status: None,
                                last_updated: None,
                                local_updated: Some(local_time.to_string()),
                            })
                            .collect();

                        GameEntry {
                            appid: appid.clone(),
                            name,
                            path,
                            spreadsheet: configs.get(&appid).cloned().unwrap_or_default(),
                            sheet_data: None,
                            mods,
                        }
                    })
                    .collect()
            });
        self.selected = None;
    }
    
    pub fn reload_mods_for_selected(&mut self) {
        if let (Some(i), Some(games)) = (self.selected, self.games.as_mut()) {
            let game = &mut games[i];
            let local_mods = steam::get_active_mods(&game.path, &game.appid);
            let mut existing: HashMap<String, WorkshopMod> =
                game.mods.drain(..).map(|m| (m.id.clone(), m)).collect();

            game.mods = local_mods
                .into_iter()
                .map(|(id, local_time)| {
                    if let Some(mut m) = existing.remove(&id) {
                        m.local_updated = Some(local_time.to_string());
                        m
                    } else {
                        WorkshopMod {
                            id,
                            name: None,
                            status: None,
                            last_updated: None,
                            local_updated: Some(local_time.to_string()),
                        }
                    }
                })
                .collect();
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let general = load_general_config();
        let default_path = if general.steam_path.is_empty() {
            steam::default_steam_path()
                .unwrap_or_else(|| "C:/Program Files (x86)/Steam".into())
                .to_string_lossy()
                .to_string()
        } else {
            general.steam_path
        };

        let (tx, rx) = std::sync::mpsc::channel();
        let rt = Arc::new(Runtime::new().expect("Tokio runtime"));

        let mut app = Self {
            games: None,
            user_path: default_path.to_string(),
            selected: None,
            tx,
            rx,
            rt,
            fetching_all: false,
            progress: None,
            sort_by_id_asc: true,
            show_all_games: false,
            status_colors: StatusColors::load_or_create(),
            settings: SettingsWindow::new(),
        };

        app.reload_games();
        app
    }
}

impl eframe::App for MyApp {

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(games) = &self.games {
            let configs: HashMap<String, SpreadsheetConfig> = games
                .iter()
                .filter_map(|g| {
                    if !g.spreadsheet.is_default() {
                        Some((g.appid.clone(), g.spreadsheet.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            if !configs.is_empty() {
                save_spreadsheet_configs(&configs);
            }
        }
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        while let Ok((id, title, updated)) = self.rx.try_recv() {
            if id == "__done__" {
                self.fetching_all = false;
                self.progress = None;
                continue;
            }
            if id == "__progress__" {
                if let (Ok(done), Ok(total)) = (title.parse::<usize>(), updated.parse::<usize>()) {
                    self.progress = Some((done, total));
                }
                continue;
            }
            if let Some(games) = &mut self.games {
                for g in games {
                    for m in &mut g.mods {
                        if m.id == id {
                            m.name = Some(title.clone());
                            m.last_updated = Some(updated.clone());
                        }
                    }
                }
            }
            ctx.request_repaint();
        }
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(games) = &self.games {
                    ui.label(format!("Total games: {}", games.len()));
                } else {
                    ui.label("No games loaded");
                }
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                    ui.label("Made by TKronix");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.settings.load_from(&self.status_colors);
                        self.settings.open = true;
                    }
                });
            });
        });
            // Left: game list
        egui::SidePanel::left("game_list")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Games");

                // 🔹 Checkbox toggle
                ui.checkbox(&mut self.show_all_games, "Show all games");

                if let Some(games) = &self.games {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, game) in games.iter().enumerate() {
                            if self.show_all_games || !game.mods.is_empty() {
                                if ui
                                    .selectable_label(self.selected == Some(i), &game.name)
                                    .clicked()
                                {
                                    self.selected = Some(i);
                                }
                            }
                        }
                    });
                } else {
                    ui.label("No games found");
                }
            });

        let ids: Option<Vec<String>> = self.selected
                        .and_then(|i| self.games.as_ref().map(|g| {
                            g[i].mods.iter().map(|m| m.id.clone()).collect()
                        }));

        // Right: details panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(games) = &mut self.games {
                if let Some(i) = self.selected {
                    let game = &mut games[i];
                    if game.sheet_data.is_none() {
                        if !game.spreadsheet.sheet_url.is_empty() {
                            if let Some(csv_url) = spreadsheet::to_csv_url(&game.spreadsheet.sheet_url) {
                                game.sheet_data = spreadsheet::load_csv_from_url(&csv_url);
                            }
                        } else if let Some(path) = &game.spreadsheet.sheet_file {
                            game.sheet_data = spreadsheet::load_csv_from_file(path);
                        }
                    }

                    game.apply_spreadsheet();

                    ui.heading(&game.name);
                    ui.label(format!("AppID: {}", game.appid));
                    ui.label(format!("Install dir: {}", game.path));
                    ui.separator();

                    SpreadsheetSection::ui(game, ui);

                    ui.separator();

                    let mut reload_mods = false;

                    ui.horizontal(|ui| {
                        if ui.button("Reload Mods List").clicked() {
                            reload_mods = true;
                        }

                        if self.fetching_all {
                            if let Some((done, total)) = self.progress {
                                ui.label(format!("Fetching mods data {}/{}", done, total));
                            } else {
                                ui.label("Fetching mods data…");
                            }
                        } else if ui.button("Check all mods against Steam Workshop (batched)").clicked() {
                            if let Some(ids) = ids {
                                self.fetching_all = true;
                                self.progress = Some((0, ids.len()));
                                let tx = self.tx.clone();
                                let rt = self.rt.clone();
                                rt.spawn(async move {
                                    let details = crate::steam::fetch_mods_details(&ids).await;
                                    let total = details.len();
                                    for (i, (id, title, updated)) in details.into_iter().enumerate() {
                                        let _ = tx.send((id, title, updated));
                                        let _ = tx.send((
                                            "__progress__".into(),
                                            (i + 1).to_string(),
                                            total.to_string(),
                                        ));
                                    }
                                    let _ = tx.send(("__done__".into(), String::new(), String::new()));
                                });
                            }
                        }
                    });

                    ui.separator();

                    ModsTable::ui(game, ui, &mut self.sort_by_id_asc, &self.status_colors);

                    if reload_mods {
                        self.reload_mods_for_selected();
                    }
                } else {
                    ui.label("Select a game to see details");
                }
            } else {
                ui.label("Could not find Steam automatically.");
                ui.horizontal(|ui| {
                    if ui.button("Choose Steam folder…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            if let Some(path_str) = path.to_str() {
                                self.user_path = path_str.to_string();
                                self.reload_games();
                                save_general_config(&GeneralConfig {
                                    steam_path: self.user_path.clone(),
                                });
                            }
                        }
                    }
                    ui.label(format!("Current path: {}", self.user_path));
                });
            }
        });
        match self.settings.show(ctx, &mut self.user_path, &mut self.status_colors) {
            SettingsAction::SteamPathChanged => {
                self.reload_games();
                save_general_config(&GeneralConfig {
                    steam_path: self.user_path.clone(),
                });
            }
            SettingsAction::None => {}
        }

        if !self.settings.open {
            self.status_colors = StatusColors::load_or_create();
        }
    }
}

pub fn format_timestamp(raw: Option<&str>) -> String {
    if let Some(raw) = raw {
        if let Ok(ts) = raw.parse::<i64>() {
            if let Ok(dt) = time::OffsetDateTime::from_unix_timestamp(ts) {
                if let Ok(fmt) = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]") {
                    return dt.format(&fmt).unwrap_or_default();
                }
            }
        }
        raw.to_string()
    } else {
        String::new()
    }
}