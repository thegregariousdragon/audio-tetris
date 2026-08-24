use crate::db::{HighScoreEntry, PlayerStats};
use rust_i18n::t;

pub fn get_leaderboard_items_count(scores: &[HighScoreEntry]) -> usize {
    if scores.is_empty() {
        3
    } else {
        scores.len() + 2
    }
}

pub fn render_leaderboard(
    selection: usize,
    scores: &[HighScoreEntry],
    stats: &PlayerStats,
) -> (String, String) {
    let mut text = t!("leaderboard.title").to_string();
    let mut items = Vec::new();

    items.push(
        t!(
            "leaderboard.stats",
            games = stats.total_games_played.to_string(),
            lines = stats.total_lines_cleared.to_string(),
            high_score = stats.high_score.to_string()
        )
        .to_string(),
    );

    if scores.is_empty() {
        items.push(t!("leaderboard.no_scores").to_string());
    } else {
        for (i, entry) in scores.iter().enumerate() {
            items.push(
                t!(
                    "leaderboard.rank_entry",
                    rank = (i + 1).to_string(),
                    score = entry.score.to_string(),
                    level = entry.level.to_string(),
                    lines = entry.lines.to_string(),
                    diff = &entry.difficulty
                )
                .to_string(),
            );
        }
    }

    items.push(t!("common.back").to_string());

    let sel = selection.min(items.len().saturating_sub(1));
    for (i, item) in items.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", item));
        } else {
            text.push_str(&format!("   {}\n", item));
        }
    }

    let spoken = t!(
        "common.item_counter",
        item = &items[sel],
        current = (sel + 1).to_string(),
        total = items.len().to_string()
    )
    .to_string();
    (text, spoken)
}
