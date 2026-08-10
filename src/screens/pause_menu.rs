pub fn get_pause_menu_options() -> Vec<&'static str> {
    vec![
        "Resume Game",
        "Save Game",
        "Abandon Game (Unsaved progress will be lost)",
        "How to Play",
        "Settings",
    ]
}

pub fn render_pause_menu(selection: usize) -> (String, String) {
    let options = get_pause_menu_options();
    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = String::from("Pause Menu\n\n");
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
