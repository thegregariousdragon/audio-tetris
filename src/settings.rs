use serde::{Deserialize, Serialize};
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
#[serde(default)]
pub struct Settings {
    pub sfx_volume: f32,   // 0.0 to 1.0
    pub bgm_volume: f32,   // 0.0 to 1.0
    pub voice_volume: f32, // 0.0 to 1.0 — informational; actual voice volume controlled by screen reader
    pub difficulty: Difficulty,

    pub piece_callouts_technical: bool,
    pub scoring_details_advanced: bool,
    pub zone_alerts: bool,

    pub bgm_enabled: bool,
    pub saved_bgm_volume: f32, // The volume to restore when toggled back on

    pub check_for_updates: bool,
    pub last_update_check_timestamp: u64,

    pub tutorial_completed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sfx_volume: 0.2,
            bgm_volume: 0.2,
            voice_volume: 1.0,
            difficulty: Difficulty::Moderate,

            piece_callouts_technical: false,
            scoring_details_advanced: true,
            zone_alerts: true,

            bgm_enabled: true,
            saved_bgm_volume: 0.2,

            check_for_updates: true,
            last_update_check_timestamp: 0,

            tutorial_completed: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = Path::new("settings.json");
        if path.exists()
            && let Ok(contents) = fs::read_to_string(path)
            && let Ok(settings) = serde_json::from_str(&contents)
        {
            return settings;
        }
        Settings::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write("settings.json", json);
        }
    }
}
