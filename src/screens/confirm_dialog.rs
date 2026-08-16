use crate::screens::ConfirmAction;

pub fn render_confirm_dialog(action: ConfirmAction) -> (String, String) {
    match action {
        ConfirmAction::NewGame => (
            "Abandon active game and start a new game?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel".to_string(),
            "Abandon active game and start a new game? Press Enter to confirm, Escape to cancel.".to_string(),
        ),
        ConfirmAction::StartTutorial => (
            "Abandon active game and start Tutorial?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel".to_string(),
            "Abandon active game and start Tutorial? Press Enter to confirm, Escape to cancel.".to_string(),
        ),
        ConfirmAction::AbandonGame => (
            "Abandon active game and return to Main Menu?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel".to_string(),
            "Abandon active game and return to Main Menu? Press Enter to confirm, Escape to cancel.".to_string(),
        ),
        ConfirmAction::QuitApp => (
            "Quit Audio Tetris?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel".to_string(),
            "Quit Audio Tetris? Press Enter to confirm, Escape to cancel.".to_string(),
        ),
        ConfirmAction::UpdateApp(ref info) => (
            format!(
                "Install Update {} and restart Audio Tetris?\nYour saves and settings will be preserved.\n\n-> Press Enter to Confirm\n   Press Escape to Cancel",
                info.version
            ),
            format!(
                "Install update {} and restart Audio Tetris? Your saves and settings will be preserved. Press Enter to confirm, Escape to cancel.",
                info.version
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_confirm_dialog_start_tutorial() {
        let (text, spoken) = render_confirm_dialog(ConfirmAction::StartTutorial);
        assert!(text.contains("start Tutorial"));
        assert!(spoken.contains("Abandon active game and start Tutorial?"));
    }
}
