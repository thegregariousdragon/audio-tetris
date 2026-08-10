use crate::screens::ConfirmAction;

pub fn render_confirm_dialog(action: ConfirmAction) -> (String, String) {
    let (prompt, spoken) = match action {
        ConfirmAction::NewGame => (
            "Abandon active game and start a new game?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel",
            "Abandon active game and start a new game? Press Enter to confirm, Escape to cancel."
                .to_string(),
        ),
        ConfirmAction::AbandonGame => (
            "Abandon active game and return to Main Menu?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel",
            "Abandon active game and return to Main Menu? Press Enter to confirm, Escape to cancel."
                .to_string(),
        ),
        ConfirmAction::QuitApp => (
            "Quit Audio Tetris?\n\n-> Press Enter to Confirm\n   Press Escape to Cancel",
            "Quit Audio Tetris? Press Enter to confirm, Escape to cancel.".to_string(),
        ),
    };

    (prompt.to_string(), spoken)
}
