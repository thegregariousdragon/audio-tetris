use crate::db::{HighScoreEntry, PlayerStats};

pub fn render_leaderboard(
    selection: usize,
    scores: &[HighScoreEntry],
    stats: &PlayerStats,
) -> (String, String) {
    let mut text = String::from("High Scores & Lifetime Statistics\n\n");
    let mut items = Vec::new();

    items.push(format!(
        "Lifetime Stats: Total Games Played {}, Total Lines Cleared {}, Highest Score {}",
        stats.total_games_played, stats.total_lines_cleared, stats.high_score
    ));

    if scores.is_empty() {
        items.push("No high scores recorded yet.".to_string());
    } else {
        for (i, entry) in scores.iter().enumerate() {
            items.push(format!(
                "Rank {}: Score {}, Level {}, Lines {}, Difficulty {}",
                i + 1,
                entry.score,
                entry.level,
                entry.lines,
                entry.difficulty
            ));
        }
    }

    items.push("Back".to_string());

    let sel = selection.min(items.len().saturating_sub(1));
    for (i, item) in items.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", item));
        } else {
            text.push_str(&format!("   {}\n", item));
        }
    }

    let spoken = format!("{} {} of {}", items[sel], sel + 1, items.len());
    (text, spoken)
}
