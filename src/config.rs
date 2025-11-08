use std::{collections::HashMap, fs, path::PathBuf};
use serde::{Serialize, Deserialize};

fn ensure_data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let data_dir = exe_dir.join("data");
    let _ = fs::create_dir_all(&data_dir);
    data_dir
}

pub fn read_json<T: for<'de> Deserialize<'de>>(filename: &str) -> Option<T> {
    let path = ensure_data_dir().join(filename);
    fs::read_to_string(path).ok().and_then(|data| serde_json::from_str(&data).ok())
}

pub fn write_json<T: Serialize>(filename: &str, value: &T) -> std::io::Result<()> {
    let path = ensure_data_dir().join(filename);
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json)
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GeneralConfig {
    pub steam_path: String,
}

pub fn save_general_config(cfg: &GeneralConfig) {
    let _ = write_json("general.json", cfg);
}

pub fn load_general_config() -> GeneralConfig {
    read_json("general.json").unwrap_or_default()
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SpreadsheetConfig {
    pub sheet_url: String,
    pub sheet_file: Option<String>,
    pub header_row_index: Option<usize>,
    pub id_col: Option<usize>,
    pub status_col: Option<usize>,
    pub name_col: Option<usize>,
}

impl SpreadsheetConfig {
    pub fn is_default(&self) -> bool {
        self.sheet_url.is_empty()
            && self.sheet_file.is_none()
            && self.header_row_index.is_none()
            && self.id_col.is_none()
            && self.status_col.is_none()
            && self.name_col.is_none()
    }
}

pub fn save_spreadsheet_configs(configs: &HashMap<String, SpreadsheetConfig>) {
    let _ = write_json("spreadsheet_configs.json", configs);
}

pub fn load_spreadsheet_configs() -> HashMap<String, SpreadsheetConfig> {
    read_json("spreadsheet_configs.json").unwrap_or_default()
}
