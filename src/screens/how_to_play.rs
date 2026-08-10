pub fn get_how_to_play_lines() -> Vec<&'static str> {
    vec![
        "Welcome to Audio Tetris, a fully accessible audio-first puzzle game!",
        "Controls:",
        "LEFT/RIGHT Arrows: Move piece horizontally. Audio panning reflects position.",
        "DOWN Arrow: Soft drop.",
        "SPACE BAR: Hard drop with heavy impact thud.",
        "UP Arrow or Z: Rotate Clockwise / Counter-Clockwise.",
        "C Key: Hold piece.",
        "R Key: Play Radar Sweep to hear stack heights.",
        "R-SHIFT Key: Activate Zone Mode when meter is full.",
        "F Key: Use acquired Item (Magnet, Nuke, Laser).",
        "Escape Key: Pause game or return to menu.",
        "F1 Key: Keyboard Help Mode.",
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
