pub fn get_main_menu_options(in_prog: bool) -> Vec<&'static str> {
    if in_prog {
        vec![
            "Resume Game",
            "New Game",
            "Load Game",
            "High Scores & Stats",
            "How to Play",
            "Settings",
            "About",
            "Quit",
        ]
    } else {
        vec![
            "New Game",
            "Load Game",
            "High Scores & Stats",
            "How to Play",
            "Settings",
            "About",
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
