use crate::db::SaveSlotInfo;
use rust_i18n::t;

pub fn render_load_screen(selection: usize, slots: &[Option<SaveSlotInfo>]) -> (String, String) {
    let mut text = t!("save_load.load_title").to_string();
    let mut spoken_options = Vec::with_capacity(6);

    for i in 1..=5 {
        let slot_info = slots.get(i - 1).and_then(|opt| opt.as_ref());
        let desc = match slot_info {
            Some(info) => t!(
                "save_load.slot_filled",
                slot = i.to_string(),
                level = info.level.to_string(),
                score = info.score.to_string(),
                lines = info.lines.to_string(),
                time = &info.timestamp
            )
            .to_string(),
            None => t!("save_load.slot_empty", slot = i.to_string()).to_string(),
        };
        spoken_options.push(desc.clone());
        if i - 1 == selection {
            text.push_str(&format!("->   {}\n", desc));
        } else {
            text.push_str(&format!("   {}\n", desc));
        }
    }

    let back_str = t!("common.back").to_string();
    spoken_options.push(back_str.clone());
    if selection == 5 {
        text.push_str(&format!("->   {}\n", back_str));
    } else {
        text.push_str(&format!("   {}\n", back_str));
    }

    let sel = selection.min(5);
    let spoken = t!(
        "common.item_counter",
        item = &spoken_options[sel],
        current = (sel + 1).to_string(),
        total = spoken_options.len().to_string()
    )
    .to_string();
    (text, spoken)
}
