use crate::logic::GameState;

pub fn render_in_game(gs: &GameState) -> (String, String) {
    let display_text = format!(
        "In Game\nLevel: {}\nScore: {}\nLines: {}\nZone Meter: {}%\n\nPress Escape to pause game.",
        gs.level, gs.score, gs.total_lines, gs.zone_meter
    );
    (display_text, "".to_string())
}
