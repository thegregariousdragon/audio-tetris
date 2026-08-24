use crate::updater::UpdateStatus;
use rust_i18n::t;

pub fn render_update_screen(
    selection: usize,
    current_version: &str,
    status: &UpdateStatus,
) -> (String, String) {
    let mut text = t!("updater.title", version = current_version).to_string();
    let mut options = Vec::new();
    let check_updates_str = t!("updater.check_for_updates").to_string();
    let install_update_str = t!("updater.install_update").to_string();
    let back_str = t!("common.back").to_string();

    match status {
        UpdateStatus::Idle => {
            text.push_str(&t!("updater.ready"));
            options.push(check_updates_str);
            options.push(back_str);
        }
        UpdateStatus::Checking => {
            text.push_str(&t!("updater.checking"));
            options.push(check_updates_str);
            options.push(back_str);
        }
        UpdateStatus::Throttled => {
            text.push_str(&t!("updater.throttled"));
            options.push(check_updates_str);
            options.push(back_str);
        }
        UpdateStatus::Available(info) => {
            text.push_str(&t!(
                "updater.available",
                version = &info.version,
                notes = &info.release_notes
            ));
            options.push(install_update_str);
            options.push(check_updates_str);
            options.push(back_str);
        }
        UpdateStatus::UpToDate => {
            text.push_str(&t!("updater.up_to_date"));
            options.push(check_updates_str);
            options.push(back_str);
        }
        UpdateStatus::Downloading => {
            text.push_str(&t!("updater.downloading"));
            options.push(back_str);
        }
        UpdateStatus::Error(err) => {
            text.push_str(&t!("updater.error", err = err));
            options.push(check_updates_str);
            options.push(back_str);
        }
    }

    let sel = selection.min(options.len().saturating_sub(1));
    for (i, opt) in options.iter().enumerate() {
        if i == sel {
            text.push_str(&format!("->   {}\n", opt));
        } else {
            text.push_str(&format!("   {}\n", opt));
        }
    }

    let spoken = match (status, sel) {
        (UpdateStatus::Checking, _) => t!("updater.spoken_checking").to_string(),
        (UpdateStatus::Available(info), 0) => t!(
            "updater.spoken_install_option",
            version = &info.version,
            total = options.len().to_string()
        )
        .to_string(),
        _ => t!(
            "common.item_counter",
            item = &options[sel],
            current = (sel + 1).to_string(),
            total = options.len().to_string()
        )
        .to_string(),
    };

    (text, spoken)
}
