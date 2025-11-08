use std::{collections::HashMap};
use egui::Color32;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::config::{read_json, write_json};

pub struct StatusColors {
    pub known: HashMap<String, Color32>,
}

impl StatusColors {
    pub fn load_or_create() -> Self {
        if let Some(parsed) = read_json::<HashMap<String, String>>("status_colors.json") {
            let mut known = HashMap::new();
            for (k, v) in parsed {
                if let Ok(c) = Color32::from_hex(&v) {
                    known.insert(k.to_lowercase(), c);
                }
            }
            return Self { known };
        }

        // Defaults
        let mut defaults = HashMap::new();
        defaults.insert("Functional".to_string(), "#00AA00".to_string());
        defaults.insert("Broken".to_string(), "#F44336".to_string());
        defaults.insert("Outdated".to_string(), "#F44336".to_string());
        defaults.insert("Not updated".to_string(), "#F44336".to_string());
        defaults.insert("Unmaintained".to_string(), "#888888".to_string());
        defaults.insert("Abandoned".to_string(), "#555555".to_string());
        defaults.insert("Semi-Functional".to_string(), "#FFA500".to_string());
        defaults.insert("Yes".to_string(), "#4CAF50".to_string());
        defaults.insert("No".to_string(), "#F44336".to_string());
        defaults.insert("WIP".to_string(), "#42A5F5".to_string());
        defaults.insert("Beta".to_string(), "#AB47BC".to_string());
        defaults.insert("Deprecated".to_string(), "#757575".to_string());
        defaults.insert("Stable".to_string(), "#388E3C".to_string());
        defaults.insert("Testing".to_string(), "#FFB300".to_string());
        defaults.insert("Compatible".to_string(), "#00897B".to_string());
        defaults.insert("Incompatible".to_string(), "#E57373".to_string());
        defaults.insert("Updated".to_string(), "#66BB6A".to_string());
        defaults.insert("Legacy".to_string(), "#8D6E63".to_string());

        let _ = write_json("status_colors.json", &defaults);

        let mut known = HashMap::new();
        for (k, v) in defaults {
            if let Ok(c) = Color32::from_hex(&v) {
                known.insert(k.to_lowercase(), c);
            }
        }

        Self { known }
    }

    pub fn color_for(&self, term: &str) -> Color32 {
        let key = term.to_lowercase();
        if let Some(c) = self.known.get(&key) {
            return *c;
        }

        // Fallback: deterministic hash → color
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let r = ((hash >> 16) & 0xFF) as u8;
        let g = ((hash >> 8) & 0xFF) as u8;
        let b = (hash & 0xFF) as u8;

        Color32::from_rgb(r, g, b)
    }

    pub fn save(&self) {
        let mut json_map: HashMap<String, String> = HashMap::new();
        for (k, v) in &self.known {
            let hex = format!("#{:02X}{:02X}{:02X}", v.r(), v.g(), v.b());
            json_map.insert(k.clone(), hex);
        }
        let _ = write_json("status_colors.json", &json_map);
    }
}
