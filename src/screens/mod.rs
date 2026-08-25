pub mod about_screen;
pub mod confirm_dialog;
pub mod how_to_play;
pub mod in_game_screen;
pub mod leaderboard;
pub mod load_screen;
pub mod main_menu;
pub mod pause_menu;
pub mod save_screen;
pub mod settings_screen;
pub mod tutorial_prompt;
pub mod tutorial_screen;
pub mod update_screen;

use crate::updater::{UpdateInfo, UpdateStatus};

#[derive(Clone, PartialEq, Debug)]
pub enum ConfirmAction {
    NewGame,
    StartTutorial,
    AbandonGame,
    QuitApp,
    UpdateApp(UpdateInfo),
}

#[derive(Clone, PartialEq, Debug)]
pub enum AppScreen {
    TutorialPrompt,
    Tutorial {
        stage: usize,
    },
    MainMenu {
        selection: usize,
    },
    PauseMenu {
        selection: usize,
    },
    SaveScreen {
        selection: usize,
    },
    LoadScreen {
        selection: usize,
    },
    Leaderboard {
        selection: usize,
    },
    Settings {
        selection: usize,
    },
    VisualSettings {
        selection: usize,
    },
    SpeechVerbosity {
        selection: usize,
    },
    HowToPlay {
        scroll_line: usize,
    },
    About {
        scroll_line: usize,
    },
    Update {
        selection: usize,
        status: UpdateStatus,
    },
    ConfirmDialog {
        action: ConfirmAction,
    },
    InGame,
    KeyDescriber {
        esc_count: usize,
    },
}
