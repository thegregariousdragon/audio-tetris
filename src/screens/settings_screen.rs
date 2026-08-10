use crate::settings::Settings;

pub fn render_settings(selection: usize, s: &Settings) -> (String, String) {
    let options = [
        format!("Difficulty: {}", s.difficulty.as_str()),
        "Speech Verbosity".to_string(),
        format!(
            "Voice Cues Volume: {}% (controlled by your screen reader)",
            (s.voice_volume * 100.0) as i32
        ),
        format!("Sound Effects Volume: {}%", (s.sfx_volume * 100.0) as i32),
        format!(
            "Background Music: {}",
            if s.bgm_enabled { "ON" } else { "OFF" }
        ),
        format!(
            "Background Music Volume: {}%",
            (s.bgm_volume * 100.0) as i32
        ),
        "Back".to_string(),
    ];

    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = String::from("Settings\nUse Left and Right arrows to adjust values.\n\n");
    for (i, opt) in options.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", opt));
        } else {
            text.push_str(&format!("   {}\n", opt));
        }
    }
    let spoken = format!("{} {} of {}", options[sel], sel + 1, options.len());
    (text, spoken)
}

pub fn render_speech_verbosity(selection: usize, s: &Settings) -> (String, String) {
    let options = [
        format!(
            "Piece Callouts: {}",
            if s.piece_callouts_technical {
                "Terse"
            } else {
                "Descriptive"
            }
        ),
        format!(
            "Scoring Details: {}",
            if s.scoring_details_advanced {
                "Advanced"
            } else {
                "Simple"
            }
        ),
        format!("Zone Alerts: {}", if s.zone_alerts { "On" } else { "Off" }),
        "Back".to_string(),
    ];

    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = String::from("Speech Verbosity Settings\n\n");
    for (i, opt) in options.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", opt));
        } else {
            text.push_str(&format!("   {}\n", opt));
        }
    }
    let spoken = format!("{} {} of {}", options[sel], sel + 1, options.len());
    (text, spoken)
}
