use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::i18n::Language;

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

    pub fn localized_str(&self) -> String {
        match self {
            Difficulty::Easy => rust_i18n::t!("settings.difficulty_easy").to_string(),
            Difficulty::Moderate => rust_i18n::t!("settings.difficulty_moderate").to_string(),
            Difficulty::Difficult => rust_i18n::t!("settings.difficulty_difficult").to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WindowSizeMode {
    #[default]
    Standard,
    Large,
    Maximized,
}

impl WindowSizeMode {
    pub fn next(&self) -> Self {
        match self {
            WindowSizeMode::Standard => WindowSizeMode::Large,
            WindowSizeMode::Large => WindowSizeMode::Maximized,
            WindowSizeMode::Maximized => WindowSizeMode::Standard,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            WindowSizeMode::Standard => WindowSizeMode::Maximized,
            WindowSizeMode::Large => WindowSizeMode::Standard,
            WindowSizeMode::Maximized => WindowSizeMode::Large,
        }
    }

    pub fn localized_str(&self) -> String {
        match self {
            WindowSizeMode::Standard => rust_i18n::t!("settings.window_size_standard").to_string(),
            WindowSizeMode::Large => rust_i18n::t!("settings.window_size_large").to_string(),
            WindowSizeMode::Maximized => {
                rust_i18n::t!("settings.window_size_maximized").to_string()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FontScale {
    #[default]
    Standard,
    Large,
    ExtraLarge,
}

impl FontScale {
    pub fn next(&self) -> Self {
        match self {
            FontScale::Standard => FontScale::Large,
            FontScale::Large => FontScale::ExtraLarge,
            FontScale::ExtraLarge => FontScale::Standard,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            FontScale::Standard => FontScale::ExtraLarge,
            FontScale::Large => FontScale::Standard,
            FontScale::ExtraLarge => FontScale::Large,
        }
    }

    pub fn point_size(&self) -> i32 {
        match self {
            FontScale::Standard => 14,
            FontScale::Large => 18,
            FontScale::ExtraLarge => 22,
        }
    }

    pub fn localized_str(&self) -> String {
        match self {
            FontScale::Standard => rust_i18n::t!("settings.font_scale_standard").to_string(),
            FontScale::Large => rust_i18n::t!("settings.font_scale_large").to_string(),
            FontScale::ExtraLarge => rust_i18n::t!("settings.font_scale_extralarge").to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub sfx_volume: f32,   // 0.0 to 1.0
    pub bgm_volume: f32,   // 0.0 to 1.0
    pub voice_volume: f32, // 0.0 to 1.0 — informational; actual voice volume controlled by screen reader
    pub difficulty: Difficulty,

    pub theme: ThemePreference,
    pub window_size: WindowSizeMode,
    pub font_scale: FontScale,

    pub piece_callouts_technical: bool,
    pub scoring_details_advanced: bool,
    pub zone_alerts: bool,
    pub screen_reader_enabled: bool,

    pub bgm_enabled: bool,
    pub saved_bgm_volume: f32, // The volume to restore when toggled back on

    pub check_for_updates: bool,
    pub last_update_check_timestamp: u64,

    pub tutorial_completed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::from_system_locale(),
            sfx_volume: 0.2,
            bgm_volume: 0.2,
            voice_volume: 1.0,
            difficulty: Difficulty::Moderate,

            theme: ThemePreference::System,
            window_size: WindowSizeMode::Standard,
            font_scale: FontScale::Standard,

            piece_callouts_technical: false,
            scoring_details_advanced: true,
            zone_alerts: true,
            screen_reader_enabled: true,

            bgm_enabled: true,
            saved_bgm_volume: 0.2,

            check_for_updates: true,
            last_update_check_timestamp: 0,

            tutorial_completed: false,
        }
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_dark_mode() -> Option<bool> {
    type Hkey = *mut std::ffi::c_void;
    type Lstatus = i32;

    const HKEY_CURRENT_USER: Hkey = 0x80000001u32 as usize as Hkey;
    const RRF_RT_REG_DWORD: u32 = 0x00000010;

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegGetValueW(
            hkey: Hkey,
            lpSubKey: *const u16,
            lpValue: *const u16,
            dwFlags: u32,
            pdwType: *mut u32,
            pvData: *mut std::ffi::c_void,
            pcbData: *mut u32,
        ) -> Lstatus;
    }

    let sub_key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    let mut data: u32 = 0;
    let mut data_size = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut std::ffi::c_void,
            &mut data_size,
        )
    };

    if status == 0 {
        // AppsUseLightTheme == 0 means Dark Mode is active; 1 means Light Mode
        Some(data == 0)
    } else {
        None
    }
}

pub fn detect_system_dark_mode() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Some(is_dark) = detect_windows_dark_mode() {
            return is_dark;
        }
    }

    wxdragon::appearance::is_system_dark_mode()
}

impl Settings {
    #[allow(dead_code)]
    pub fn is_dark_mode(&self) -> bool {
        match self.theme {
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
            ThemePreference::System => detect_system_dark_mode(),
        }
    }

    pub fn get_theme_colors(&self) -> ((u8, u8, u8), (u8, u8, u8)) {
        ((18, 18, 24), (245, 245, 245))
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_size_cycling() {
        let size = WindowSizeMode::Standard;
        assert_eq!(size.next(), WindowSizeMode::Large);
        assert_eq!(size.next().next(), WindowSizeMode::Maximized);
        assert_eq!(size.next().next().next(), WindowSizeMode::Standard);

        assert_eq!(size.prev(), WindowSizeMode::Maximized);
    }

    #[test]
    fn test_font_scale_cycling_and_points() {
        let font = FontScale::Standard;
        assert_eq!(font.point_size(), 14);
        assert_eq!(font.next(), FontScale::Large);
        assert_eq!(font.next().point_size(), 18);
        assert_eq!(font.next().next(), FontScale::ExtraLarge);
        assert_eq!(font.next().next().point_size(), 22);
        assert_eq!(font.next().next().next(), FontScale::Standard);

        assert_eq!(font.prev(), FontScale::ExtraLarge);
    }

    #[test]
    fn test_theme_colors() {
        let mut s = Settings {
            theme: ThemePreference::Dark,
            ..Default::default()
        };
        let (bg, fg) = s.get_theme_colors();
        assert_eq!(bg, (18, 18, 24));
        assert_eq!(fg, (245, 245, 245));

        s.theme = ThemePreference::Light;
        let (bg_l, fg_l) = s.get_theme_colors();
        assert_eq!(bg_l, (18, 18, 24));
        assert_eq!(fg_l, (245, 245, 245));
    }

    #[test]
    fn test_system_dark_mode_detection() {
        let is_dark = detect_system_dark_mode();
        let s = Settings {
            theme: ThemePreference::System,
            ..Default::default()
        };
        assert_eq!(s.is_dark_mode(), is_dark);
    }
}
