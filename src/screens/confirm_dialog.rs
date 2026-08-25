use crate::screens::ConfirmAction;
use rust_i18n::t;

pub fn render_confirm_dialog(action: ConfirmAction) -> (String, String) {
    match action {
        ConfirmAction::NewGame => (
            t!("confirm_dialog.new_game_text").to_string(),
            t!("confirm_dialog.new_game_spoken").to_string(),
        ),
        ConfirmAction::StartTutorial => (
            t!("confirm_dialog.start_tutorial_text").to_string(),
            t!("confirm_dialog.start_tutorial_spoken").to_string(),
        ),
        ConfirmAction::AbandonGame => (
            t!("confirm_dialog.abandon_game_text").to_string(),
            t!("confirm_dialog.abandon_game_spoken").to_string(),
        ),
        ConfirmAction::QuitApp => (
            t!("confirm_dialog.quit_app_text").to_string(),
            t!("confirm_dialog.quit_app_spoken").to_string(),
        ),
        ConfirmAction::UpdateApp(ref info) => (
            t!("confirm_dialog.update_app_text", version = &info.version).to_string(),
            t!("confirm_dialog.update_app_spoken", version = &info.version).to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_confirm_dialog_start_tutorial() {
        let (text, spoken) = render_confirm_dialog(ConfirmAction::StartTutorial);
        assert!(!text.trim().is_empty());
        assert!(!spoken.trim().is_empty());
    }
}
