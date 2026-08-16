pub fn get_main_menu_options(in_prog: bool) -> Vec<&'static str> {
    if in_prog {
        vec![
            "Resume Game",
            "New Game",
            "Tutorial",
            "Save Game",
            "Load Game",
            "High Scores & Stats",
            "How to Play",
            "Settings",
            "About",
            "Update",
            "Quit",
        ]
    } else {
        vec![
            "New Game",
            "Tutorial",
            "Load Game",
            "High Scores & Stats",
            "How to Play",
            "Settings",
            "About",
            "Update",
            "Quit",
        ]
    }
}

pub fn render_main_menu(selection: usize, in_prog: bool) -> (String, String) {
    let options = get_main_menu_options(in_prog);
    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = String::from("Main Menu\n\n");
    for (i, opt) in options.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", opt));
        } else {
            text.push_str(&format!("   {}\n", opt));
        }
    }
    let spoken = format!("{} {} of {}", options[sel], sel + 1, options.len());
    (text, spoken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_menu_options_not_in_prog() {
        let opts = get_main_menu_options(false);
        assert_eq!(opts.len(), 9);
        assert_eq!(opts[0], "New Game");
        assert_eq!(opts[1], "Tutorial");
        assert_eq!(opts[8], "Quit");
    }

    #[test]
    fn test_main_menu_options_in_prog() {
        let opts = get_main_menu_options(true);
        assert_eq!(opts.len(), 11);
        assert_eq!(opts[0], "Resume Game");
        assert_eq!(opts[1], "New Game");
        assert_eq!(opts[2], "Tutorial");
        assert_eq!(opts[10], "Quit");
    }

    #[test]
    fn test_render_main_menu() {
        let (text, spoken) = render_main_menu(1, false);
        assert!(text.contains("->   Tutorial"));
        assert_eq!(spoken, "Tutorial 2 of 9");
    }
}
