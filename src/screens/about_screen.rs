use rust_i18n::t;

pub fn get_about_lines() -> Vec<String> {
    vec![
        t!("about.line_0", version = env!("APP_VERSION")).to_string(),
        t!("about.line_1").to_string(),
        t!("about.line_2").to_string(),
        t!("about.line_3").to_string(),
        t!("about.line_4").to_string(),
        t!("about.line_5").to_string(),
        t!("about.line_6").to_string(),
        t!("about.line_7").to_string(),
        t!("about.line_8").to_string(),
    ]
}

pub fn render_about(scroll_line: usize, initial_load: bool) -> (String, String) {
    let lines = get_about_lines();
    let idx = scroll_line.min(lines.len().saturating_sub(1));
    let text = t!(
        "about.line_counter",
        current = (idx + 1).to_string(),
        total = lines.len().to_string(),
        line = &lines[idx]
    )
    .to_string();

    let spoken = if initial_load {
        t!("about.spoken_intro", line = &lines[idx]).to_string()
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
        assert_eq!(lines.len(), 9);
        assert!(lines[0].contains(env!("APP_VERSION")));
        assert!(lines[1].contains("Gregory Lopez"));
        assert!(lines[5].contains("wxDragon"));
        assert!(lines[6].contains("rust-i18n"));
        assert!(lines[7].contains("Lofty"));
        assert!(lines[8].contains("MIT"));
    }

    #[test]
    fn test_render_about_initial_load() {
        let (text, spoken) = render_about(0, true);
        assert!(text.contains('1'));
        assert!(!spoken.trim().is_empty());
    }

    #[test]
    fn test_render_about_subsequent_line() {
        let (text, spoken) = render_about(1, false);
        assert!(text.contains('2'));
        assert!(spoken.contains("Gregory Lopez"));
    }

    #[test]
    fn test_render_about_bounds() {
        let (text, spoken) = render_about(999, false);
        assert!(text.contains('9'));
        assert!(spoken.contains("MIT"));
    }
}
