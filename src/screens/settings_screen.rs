use crate::settings::Settings;
use rust_i18n::t;

pub fn render_settings(selection: usize, s: &Settings) -> (String, String) {
    let on_str = t!("common.on");
    let off_str = t!("common.off");

    let options = [
        t!("settings.language", name = s.language.display_name()).to_string(),
        t!("settings.difficulty", value = s.difficulty.localized_str()).to_string(),
        t!("settings.speech_verbosity").to_string(),
        t!(
            "settings.voice_volume",
            value = ((s.voice_volume * 100.0) as i32).to_string()
        )
        .to_string(),
        t!(
            "settings.sfx_volume",
            value = ((s.sfx_volume * 100.0) as i32).to_string()
        )
        .to_string(),
        t!(
            "settings.bgm_enabled",
            status = if s.bgm_enabled { &on_str } else { &off_str }
        )
        .to_string(),
        t!(
            "settings.bgm_volume",
            value = ((s.bgm_volume * 100.0) as i32).to_string()
        )
        .to_string(),
        t!(
            "settings.auto_update",
            status = if s.check_for_updates {
                &on_str
            } else {
                &off_str
            }
        )
        .to_string(),
        t!("common.back").to_string(),
    ];

    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = format!(
        "{}\n{}\n\n",
        t!("settings.title"),
        t!("settings.instruction")
    );
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

pub fn render_speech_verbosity(selection: usize, s: &Settings) -> (String, String) {
    let on_str = t!("common.on");
    let off_str = t!("common.off");
    let terse_str = t!("settings.piece_callouts_terse");
    let desc_str = t!("settings.piece_callouts_descriptive");
    let adv_str = t!("settings.scoring_details_advanced");
    let sim_str = t!("settings.scoring_details_simple");

    let options = [
        t!(
            "settings.piece_callouts",
            value = if s.piece_callouts_technical {
                &terse_str
            } else {
                &desc_str
            }
        )
        .to_string(),
        t!(
            "settings.scoring_details",
            value = if s.scoring_details_advanced {
                &adv_str
            } else {
                &sim_str
            }
        )
        .to_string(),
        t!(
            "settings.zone_alerts",
            status = if s.zone_alerts { &on_str } else { &off_str }
        )
        .to_string(),
        t!("common.back").to_string(),
    ];

    let sel = selection.min(options.len().saturating_sub(1));
    let mut text = format!("{}\n\n", t!("settings.speech_title"));
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
