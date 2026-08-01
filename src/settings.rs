use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Difficulty {
    Easy,
    Moderate,
    Difficult,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Moderate => "Moderate",
            Difficulty::Difficult => "Difficult",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub sfx_volume: f32, // 0.0 to 1.0
    pub bgm_volume: f32, // 0.0 to 1.0
    pub difficulty: Difficulty,
    pub controller_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sfx_volume: 0.2,
            bgm_volume: 0.2,
            difficulty: Difficulty::Moderate,
            controller_enabled: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = Path::new("settings.json");
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str(&contents) {
                    return settings;
                }
            }
        }
        Settings::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write("settings.json", json);
        }
    }
}
