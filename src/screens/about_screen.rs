pub fn get_about_lines() -> Vec<String> {
    vec![
        format!("Audio Tetris v{}", env!("APP_VERSION")),
        "Created by Gregory Lopez and Google Antigravity.".to_string(),
        "A fully accessible, screen-reader-first arcade puzzle game built in Rust.".to_string(),
        "Engineered with zero graphical reliance, high-precision stereo panning, and pitch-mapped elevation.".to_string(),
        "Features 5 save slots, high score tracking, Zone Mode, radar sweeps, power-ups, and auto-updater.".to_string(),
        "Powered by: wxDragon (GUI), Rodio (Audio), Tolk (Screen Readers), and Rusqlite (SQLite Database).".to_string(),
        "Additional Libraries: Serde (Serialization), Lofty (Audio Metadata), Rand (RNG), and WinRes (Build Manifest).".to_string(),
        "Copyright 2026 Gregory Lopez and Google Antigravity. Released under the MIT License.".to_string(),
    ]
}

pub fn render_about(scroll_line: usize, initial_load: bool) -> (String, String) {
    let lines = get_about_lines();
    let idx = scroll_line.min(lines.len().saturating_sub(1));
    let text = format!(
        "About Audio Tetris (Line {} of {})\n\n{}",
        idx + 1,
        lines.len(),
        lines[idx]
    );

    let spoken = if initial_load {
        format!(
            "About Audio Tetris. Use arrows to read line by line. Press Enter to read all. Press Escape to go back. {}",
            lines[idx]
        )
    } else {
        lines[idx].clone()
    };

    (text, spoken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_about_lines_not_empty() {
        let lines = get_about_lines();
        assert_eq!(lines.len(), 8);
        assert!(lines[0].starts_with("Audio Tetris v"));
        assert!(lines[1].contains("Gregory Lopez"));
        assert!(lines[5].contains("wxDragon"));
        assert!(lines[6].contains("Serde"));
        assert!(lines[7].contains("MIT License"));
    }

    #[test]
    fn test_render_about_initial_load() {
        let (text, spoken) = render_about(0, true);
        assert!(text.contains("About Audio Tetris (Line 1 of 8)"));
        assert!(spoken.contains("Use arrows to read line by line"));
        assert!(spoken.contains("Audio Tetris v"));
    }

    #[test]
    fn test_render_about_subsequent_line() {
        let (text, spoken) = render_about(1, false);
        assert!(text.contains("About Audio Tetris (Line 2 of 8)"));
        assert_eq!(spoken, "Created by Gregory Lopez and Google Antigravity.");
    }

    #[test]
    fn test_render_about_bounds() {
        let (text, spoken) = render_about(999, false);
        assert!(text.contains("About Audio Tetris (Line 8 of 8)"));
        assert!(spoken.contains("Released under the MIT License."));
    }
}
