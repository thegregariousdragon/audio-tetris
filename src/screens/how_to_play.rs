pub fn get_how_to_play_lines() -> Vec<&'static str> {
    vec![
        "Welcome to Audio Tetris, a fully accessible audio-first puzzle game!",
        "Controls:",
        "LEFT/RIGHT Arrows or A/D: Move piece left/right.",
        "DOWN Arrow or S: Soft drop.",
        "SPACE BAR: Hard drop.",
        "Z or Comma / X or Period: Rotate Counter-Clockwise / Clockwise.",
        "C Key or Slash: Hold piece.",
        "E or L Key: Radar Sweep.",
        "Q or K Key: Activate Zone Mode when meter is charged.",
        "F Key or Shift: Use acquired Item (Magnet, Nuke, Laser).",
        "V or Semicolon Key: Inspect current piece shape and column span.",
        "I, O, P Keys: Previous Track, Toggle Mute, Next Track.",
        "Escape Key: Pause game or return to previous menu.",
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
