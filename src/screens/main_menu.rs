use rust_i18n::t;

const MENU_WIDTH: usize = 44;

fn center_line(line: &str) -> String {
    if line.len() >= MENU_WIDTH {
        return line.to_string();
    }
    format!("{}{}", " ".repeat((MENU_WIDTH - line.len()) / 2), line)
}

pub fn get_main_menu_options(in_prog: bool) -> Vec<String> {
    if in_prog {
        vec![
            t!("main_menu.resume_game").to_string(),
            t!("main_menu.new_game").to_string(),
            t!("main_menu.tutorial").to_string(),
            t!("main_menu.save_game").to_string(),
            t!("main_menu.load_game").to_string(),
            t!("main_menu.leaderboard").to_string(),
            t!("main_menu.how_to_play").to_string(),
            t!("main_menu.settings").to_string(),
            t!("main_menu.about").to_string(),
            t!("main_menu.update").to_string(),
            t!("main_menu.quit").to_string(),
        ]
    } else {
        vec![
            t!("main_menu.new_game").to_string(),
            t!("main_menu.tutorial").to_string(),
            t!("main_menu.load_game").to_string(),
            t!("main_menu.leaderboard").to_string(),
            t!("main_menu.how_to_play").to_string(),
            t!("main_menu.settings").to_string(),
            t!("main_menu.about").to_string(),
            t!("main_menu.update").to_string(),
            t!("main_menu.quit").to_string(),
        ]
    }
}

pub fn render_main_menu(selection: usize, in_prog: bool) -> (String, String) {
    let options = get_main_menu_options(in_prog);
    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = String::new();
    text.push_str(&center_line("############################################"));
    text.push('\n');
    text.push_str(&center_line(&format!("# {} #", t!("main_menu.title"))));
    text.push('\n');
    text.push_str(&center_line("############################################"));
    text.push_str("\n\n");
    for (i, opt) in options.iter().enumerate() {
        if i == sel {
            text.push_str(&center_line(&format!("-> [ {} ]", opt)));
            text.push('\n');
        } else {
            text.push_str(&center_line(&format!("   {}", opt)));
            text.push('\n');
        }
    }
    let spoken = t!(
        "common.item_counter",
        item = &options[sel],
        current = (sel + 1).to_string(),
        total = options.len().to_string()
    )
    .to_string();
    (text, spoken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_menu_options_not_in_prog() {
        let opts = get_main_menu_options(false);
        assert_eq!(opts.len(), 9);
        assert!(!opts[0].is_empty());
        assert!(!opts[1].is_empty());
        assert!(!opts[8].is_empty());
    }

    #[test]
    fn test_main_menu_options_in_prog() {
        let opts = get_main_menu_options(true);
        assert_eq!(opts.len(), 11);
        assert!(!opts[0].is_empty());
        assert!(!opts[1].is_empty());
        assert!(!opts[2].is_empty());
        assert!(!opts[10].is_empty());
    }

    #[test]
    fn test_render_main_menu() {
        let (text, spoken) = render_main_menu(1, false);
        assert!(text.contains("->"));
        assert!(spoken.contains("2") && spoken.contains("9"));
    }
}
