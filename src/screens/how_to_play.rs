pub fn get_how_to_play_lines() -> Vec<&'static str> {
    vec![
        "Welcome to Audio Tetris! Here is everything you need to know to master the game.",
        "The Objective: Geometric blocks called Tetrominoes fall one at a time from the top of a 10-column grid called the Matrix.",
        "Moving & Rotating: Move falling pieces left and right, rotate them to fit into open slots, and drop them into place.",
        "Clearing Lines: When you fill a complete horizontal row of 10 blocks with no gaps, that row clears and disappears.",
        "Gravity: Any blocks above a cleared line fall down to fill the empty space.",
        "Scoring & Combos: Clearing 1, 2, 3, or 4 lines at once (a Tetris!) scores big points, increases your combo multiplier, and charges your Zone meter.",
        "Game Over: If your stack reaches the very top of the board and blocks new pieces from entering, the game ends.",
        "Audio Orientation (Panning): Audio is in full stereo. Pieces sound in your left ear for left columns, centered for middle columns, and right ear for right columns.",
        "Audio Orientation (Pitch): Sound pitch indicates vertical height. Higher pitch means the piece is high up; lower pitch means it is near the bottom or landing on the stack.",
        "Radar Sweep: Activate Radar at any time to hear a 10-tone stereo sweep revealing the current height of every column on the board.",
        "Hold Slot: Swap your active piece into the Hold slot to save it for a better moment.",
        "Zone Mode: When your Zone meter is charged, activate Zone to freeze gravity and clear as many lines as possible before time resumes!",
        "Power-Up Items: Clearing special rows awards items like Magnet, Laser, or Nuke to help you clean up difficult stacks.",
        "Keyboard Controls: To explore every key and its action interactively, press H from the Main Menu to enter Keyboard Help Mode.",
    ]
}

pub fn render_how_to_play(scroll_line: usize, initial_load: bool) -> (String, String) {
    let lines = get_how_to_play_lines();
    let idx = scroll_line.min(lines.len().saturating_sub(1));
    let text = format!(
        "How to Play (Line {} of {})\n\n{}",
        idx + 1,
        lines.len(),
        lines[idx]
    );

    let spoken = if initial_load {
        format!(
            "How to play. Use arrows to read line by line. Press Enter to read all. Press Escape to go back. {}",
            lines[idx]
        )
    } else {
        lines[idx].to_string()
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
        assert!(lines[0].contains("Welcome to Audio Tetris"));
        assert!(lines[1].contains("The Objective"));
        assert!(lines[3].contains("Clearing Lines"));
        assert!(lines[7].contains("Audio Orientation (Panning)"));
        assert!(lines[8].contains("Audio Orientation (Pitch)"));
        assert!(lines[9].contains("Radar Sweep"));
        assert!(lines[11].contains("Zone Mode"));
        assert!(lines[13].contains("Keyboard Controls"));
    }

    #[test]
    fn test_render_how_to_play_initial_load() {
        let (text, spoken) = render_how_to_play(0, true);
        assert!(text.contains("How to Play (Line 1 of 14)"));
        assert!(spoken.contains("How to play. Use arrows to read line by line."));
        assert!(spoken.contains("Welcome to Audio Tetris"));
    }

    #[test]
    fn test_render_how_to_play_subsequent_line() {
        let (text, spoken) = render_how_to_play(3, false);
        assert!(text.contains("How to Play (Line 4 of 14)"));
        assert_eq!(
            spoken,
            "Clearing Lines: When you fill a complete horizontal row of 10 blocks with no gaps, that row clears and disappears."
        );
    }

    #[test]
    fn test_render_how_to_play_bounds() {
        let (text, spoken) = render_how_to_play(999, false);
        assert!(text.contains("How to Play (Line 14 of 14)"));
        assert!(spoken.contains("Keyboard Controls:"));
    }
}
