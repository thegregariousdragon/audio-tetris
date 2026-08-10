pub fn get_about_lines() -> Vec<&'static str> {
    vec![
        "Audio Tetris v1.0.1",
        "Built with Rust, wxDragon, Rodio, and Tolk.",
        "Designed for seamless accessibility with screen readers and positional audio.",
        "Features 5 save slots, high score tracking, Zone Mode, and power-up items.",
        "Created with care for blind and visually impaired gamers.",
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
        lines[idx].to_string()
    };

    (text, spoken)
}
