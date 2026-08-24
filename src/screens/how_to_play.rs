use rust_i18n::t;

pub fn get_how_to_play_lines() -> Vec<String> {
    vec![
        t!("how_to_play.line_0").to_string(),
        t!("how_to_play.line_1").to_string(),
        t!("how_to_play.line_2").to_string(),
        t!("how_to_play.line_3").to_string(),
        t!("how_to_play.line_4").to_string(),
        t!("how_to_play.line_5").to_string(),
        t!("how_to_play.line_6").to_string(),
        t!("how_to_play.line_7").to_string(),
        t!("how_to_play.line_8").to_string(),
        t!("how_to_play.line_9").to_string(),
        t!("how_to_play.line_10").to_string(),
        t!("how_to_play.line_11").to_string(),
        t!("how_to_play.line_12").to_string(),
        t!("how_to_play.line_13").to_string(),
    ]
}

pub fn render_how_to_play(scroll_line: usize, initial_load: bool) -> (String, String) {
    let lines = get_how_to_play_lines();
    let idx = scroll_line.min(lines.len().saturating_sub(1));
    let text = t!(
        "how_to_play.line_counter",
        current = (idx + 1).to_string(),
        total = lines.len().to_string(),
        line = &lines[idx]
    )
    .to_string();

    let spoken = if initial_load {
        t!("how_to_play.spoken_intro", line = &lines[idx]).to_string()
    } else {
        lines[idx].clone()
    };

    (text, spoken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_how_to_play_lines_count_and_content() {
        let lines = get_how_to_play_lines();
        assert_eq!(lines.len(), 14);
        assert!(lines[0].contains("Audio Tetris"));
        assert!(lines[1].contains("Tetromino"));
        assert!(lines[3].contains("10"));
        assert!(lines[7].contains("stereo") || lines[7].contains("Stereo"));
        assert!(lines[8].contains("pitch") || lines[8].contains("Pitch"));
        assert!(lines[9].contains("Radar"));
        assert!(lines[11].contains("Zone"));
        assert!(lines[13].contains("Keyboard"));
    }

    #[test]
    fn test_render_how_to_play_initial_load() {
        let (text, spoken) = render_how_to_play(0, true);
        assert!(text.contains("1"));
        assert!(spoken.contains("Audio Tetris"));
    }

    #[test]
    fn test_render_how_to_play_subsequent_line() {
        let (text, spoken) = render_how_to_play(3, false);
        assert!(text.contains("4"));
        assert!(spoken.contains("10"));
    }

    #[test]
    fn test_render_how_to_play_bounds() {
        let (text, spoken) = render_how_to_play(999, false);
        assert!(text.contains("14"));
        assert!(spoken.contains("Keyboard"));
    }
}
