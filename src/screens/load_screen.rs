use crate::db::SaveSlotInfo;

pub fn render_load_screen(selection: usize, slots: &[Option<SaveSlotInfo>]) -> (String, String) {
    let mut text = String::from("Load Game - Select a Slot\n\n");
    let mut spoken_options = Vec::with_capacity(6);

    for i in 1..=5 {
        let slot_info = slots.get(i - 1).and_then(|opt| opt.as_ref());
        let desc = match slot_info {
            Some(info) => format!(
                "Slot {}: Level {}, Score {}, Lines {}. Saved {}",
                i, info.level, info.score, info.lines, info.timestamp
            ),
            None => format!("Slot {}: Empty Slot", i),
        };
        spoken_options.push(desc.clone());
        if i - 1 == selection {
            text.push_str(&format!("->   {}\n", desc));
        } else {
            text.push_str(&format!("   {}\n", desc));
        }
    }

    spoken_options.push("Back".to_string());
    if selection == 5 {
        text.push_str("->   Back\n");
    } else {
        text.push_str("   Back\n");
    }

    let sel = selection.min(5);
    let spoken = format!(
        "{} {} of {}",
        spoken_options[sel],
        sel + 1,
        spoken_options.len()
    );
    (text, spoken)
}
