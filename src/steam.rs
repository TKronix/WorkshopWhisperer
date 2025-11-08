use std::{collections::HashMap, fs};
use std::path::PathBuf;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use crate::parser::parse_vdf;

pub fn default_steam_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(r"C:\Program Files (x86)\Steam"))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|h| h.join(".local/share/Steam"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join("Library/Application Support/Steam"))
    }
}

pub fn get_installed_games(steam_path: &str) -> Option<Vec<(String, String, String)>> {
    let path = PathBuf::from(steam_path).join("steamapps").join("libraryfolders.vdf");
    let text = fs::read_to_string(&path).ok()?;
    println!("✅ Loaded libraryfolders.vdf from: {}", path.display());

    let parsed = parse_vdf(&text);
    let mut games = Vec::new();

    if let Some(folders) = parsed.get("libraryfolders").and_then(|v| v.as_object()) {
        for (_id, folder) in folders {
            if let Some(lib_path) = folder.get("path").and_then(|v| v.as_str()) {
                if let Some(apps) = folder.get("apps").and_then(|v| v.as_object()) {
                    for (appid, _) in apps {
                        // Look for appmanifest_<appid>.acf
                        let manifest = PathBuf::from(lib_path)
                            .join("steamapps")
                            .join(format!("appmanifest_{}.acf", appid));

                        let mut game_name = String::from("<unknown>");
                        if let Ok(text) = fs::read_to_string(&manifest) {
                            let manifest_parsed = parse_vdf(&text);
                            if let Some(name) = manifest_parsed
                                .get("AppState")
                                .and_then(|v| v.get("name"))
                                .and_then(|v| v.as_str())
                            {
                                game_name = name.to_string();
                            }
                        }

                        println!("🎮 AppID {} -> {} (in {})", appid, game_name, lib_path);
                        games.push((appid.clone(), game_name, lib_path.to_string()));
                    }
                }
            }
        }
    }

    Some(games)
}

pub fn get_active_mods(lib_root: &str, appid: &str) -> HashMap<String, u64> {
    let manifest_path = format!("{}/steamapps/workshop/appworkshop_{}.acf", lib_root, appid);
    let data = match std::fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

    let vdf: Value = parse_vdf(&data);

    let mut mods = HashMap::new();
    if let Some(items) = vdf
        .get("AppWorkshop")
        .and_then(|v| v.get("WorkshopItemsInstalled"))
        .and_then(|v| v.as_object())
    {
        for (mod_id, entry) in items {
            if let Some(obj) = entry.as_object() {
                if let Some(time) = obj.get("timeupdated").and_then(|v| v.as_str()) {
                    if let Ok(ts) = time.parse::<u64>() {
                        mods.insert(mod_id.clone(), ts);
                    }
                }
            }
        }
    }
    mods
}

pub async fn fetch_mods_details(mod_ids: &[String]) -> Vec<(String, String, String)> {
    let client = Client::new();
    let mut results = Vec::new();

    // Process in chunks of 50
    for chunk in mod_ids.chunks(50) {
        let mut params: Vec<(String, String)> = Vec::new();
        params.push(("itemcount".to_string(), chunk.len().to_string()));

        for (i, id) in chunk.iter().enumerate() {
            params.push((format!("publishedfileids[{}]", i), id.clone()));
        }

        if let Ok(resp) = client
            .post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
            .form(&params) // reqwest accepts Vec<(String, String)>
            .send()
            .await
        {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(items) = json["response"]["publishedfiledetails"].as_array() {
                    for item in items {
                        let id = item["publishedfileid"].as_str().unwrap_or("").to_string();
                        let title = item["title"].as_str().unwrap_or("").to_string();
                        let updated = item["time_updated"]
                            .as_i64()
                            .map(|ts| ts.to_string())
                            .unwrap_or_default();
                        results.push((id, title, updated));
                    }
                }
            }
        }

        // Delay between batches
        sleep(Duration::from_secs(1)).await;
    }

    results
}