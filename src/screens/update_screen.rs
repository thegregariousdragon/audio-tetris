use crate::updater::UpdateStatus;

pub fn render_update_screen(
    selection: usize,
    current_version: &str,
    status: &UpdateStatus,
) -> (String, String) {
    let mut text = format!(
        "Software Update (Current Version: v{})\n\n",
        current_version
    );
    let mut options = Vec::new();

    match status {
        UpdateStatus::Idle => {
            text.push_str("Status: Ready to check for updates.\n\n");
            options.push("Check for Updates");
            options.push("Back");
        }
        UpdateStatus::Checking => {
            text.push_str("Status: Checking for updates...\n\n");
            options.push("Check for Updates");
            options.push("Back");
        }
        UpdateStatus::Throttled => {
            text.push_str("Status: Update check skipped (automatically checked within the last 12 hours).\n\n");
            options.push("Check for Updates");
            options.push("Back");
        }
        UpdateStatus::Available(info) => {
            text.push_str(&format!(
                "Status: Update Available! Version {}\n\nWhat's New:\n{}\n\n",
                info.version, info.release_notes
            ));
            options.push("Install Update");
            options.push("Check for Updates");
            options.push("Back");
        }
        UpdateStatus::UpToDate => {
            text.push_str("Status: You are running the latest version of Audio Tetris.\n\n");
            options.push("Check for Updates");
            options.push("Back");
        }
        UpdateStatus::Downloading => {
            text.push_str("Status: Downloading and preparing update...\n\n");
            options.push("Back");
        }
        UpdateStatus::Error(err) => {
            text.push_str(&format!(
                "Status: Error checking for updates.\nDetails: {}\n\n",
                err
            ));
            options.push("Check for Updates");
            options.push("Back");
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
        (UpdateStatus::Available(info), 0) => {
            format!(
                "Install Update, Version {}. Option 1 of {}",
                info.version,
                options.len()
            )
        }
        _ => format!("{} {} of {}", options[sel], sel + 1, options.len()),
    };

    (text, spoken)
}
