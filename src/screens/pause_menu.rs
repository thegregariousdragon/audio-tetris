use rust_i18n::t;

pub fn get_pause_menu_options() -> Vec<String> {
    vec![
        t!("pause_menu.resume_game").to_string(),
        t!("pause_menu.save_game").to_string(),
        t!("pause_menu.abandon_game").to_string(),
        t!("pause_menu.how_to_play").to_string(),
        t!("pause_menu.settings").to_string(),
    ]
}

pub fn render_pause_menu(selection: usize) -> (String, String) {
    let options = get_pause_menu_options();
    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = format!("{}\n\n", t!("pause_menu.title"));
    for (i, opt) in options.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", opt));
        } else {
            text.push_str(&format!("   {}\n", opt));
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
