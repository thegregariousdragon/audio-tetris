use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wxdragon::prelude::*;

use crate::audio::AudioEngine;
use crate::db::Database;
use crate::logic::GameState;
use crate::screens::{
    AppScreen, ConfirmAction, about_screen, confirm_dialog, how_to_play, in_game_screen,
    leaderboard, load_screen, main_menu, pause_menu, save_screen, settings_screen, update_screen,
};
use crate::settings::{Difficulty, Settings};
use crate::updater::{self, UpdateStatus};
use tolk::Tolk;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    HardDrop,
    Radar,
    RotateLeft,
    RotateRight,
    Start,
    NextTrack,
    PrevTrack,
    Mute,
    Hold,
    Zone,
    UseItem,
    HelpMode,
    PieceInfo,
}

pub fn get_action_description(action: InputAction) -> &'static str {
    match action {
        InputAction::Left => "Left Arrow or A: Move piece left.",
        InputAction::Right => "Right Arrow or D: Move piece right.",
        InputAction::Down => {
            "Down Arrow or S: Soft drop piece in-game, or move menu selection down."
        }
        InputAction::HardDrop => "Space Bar: Hard drop piece instantly.",
        InputAction::RotateLeft => "Z or Comma: Rotate piece counter-clockwise.",
        InputAction::RotateRight => "X or Period: Rotate piece clockwise.",
        InputAction::Hold => "C or Slash: Hold piece.",
        InputAction::Radar => "E or L Key: Radar sweep for stack heights.",
        InputAction::Zone => "Q or K Key: Activate Zone mode.",
        InputAction::UseItem => "Shift Key: Use power-up item.",
        InputAction::PieceInfo => "V or Semicolon Key: Inspect piece shape and column span.",
        InputAction::PrevTrack => "I Key: Previous background music track.",
        InputAction::Mute => "O Key: Toggle background music mute.",
        InputAction::NextTrack => "P Key: Next background music track.",
        InputAction::Start => "Start Button: Gamepad Menu / Pause.",
        InputAction::HelpMode => "H Key: Keyboard Help Mode.",
        InputAction::Up => "Up Arrow or W: Move menu selection up.",
        InputAction::Select => "Enter Key: Select menu option.",
        InputAction::Back => "Escape Key: Go back or pause game.",
    }
}

pub struct AppFrame {
    frame: Frame,
    panel: Panel,
    text_display: StaticText,
    game_state: Arc<Mutex<GameState>>,
    audio_engine: Rc<AudioEngine>,
    timer: Rc<RefCell<wxdragon::timer::Timer<Frame>>>,
    tolk: Arc<Tolk>,
    settings: Arc<Mutex<Settings>>,
    screen: Arc<Mutex<AppScreen>>,
    game_in_progress: Arc<Mutex<bool>>,
    db: Arc<Database>,
}

impl AppFrame {
    pub fn new() -> Self {
        let tolk = Tolk::new();
        tolk.try_sapi(true);

        let settings_data = Settings::load();
        let settings = Arc::new(Mutex::new(settings_data.clone()));
        let screen = Arc::new(Mutex::new(AppScreen::MainMenu { selection: 0 }));
        let game_in_progress = Arc::new(Mutex::new(false));

        let db = Arc::new(Database::new("audio_tetris.db").expect("Failed to initialize database"));

        let title = format!("Audio Tetris v{}", env!("APP_VERSION"));
        let frame = Frame::builder()
            .with_title(&title)
            .with_size(Size::new(600, 400))
            .build();

        let panel = Panel::builder(&frame)
            .with_style(PanelStyle::BorderNone)
            .build();

        let sizer = BoxSizer::builder(Orientation::Vertical).build();

        let text_display = StaticText::builder(&panel).with_label("Loading...").build();

        sizer.add(&text_display, 1, SizerFlag::All | SizerFlag::Expand, 20);

        panel.set_sizer(sizer, true);

        let game_state = Arc::new(Mutex::new(GameState::new(settings_data.difficulty)));
        let audio_engine = Rc::new(AudioEngine::new(&settings_data).unwrap());
        let timer = Rc::new(RefCell::new(wxdragon::timer::Timer::new(&frame)));

        let app_frame = Self {
            frame,
            panel,
            text_display,
            game_state,
            audio_engine,
            timer,
            tolk,
            settings,
            screen,
            game_in_progress,
            db,
        };

        if settings_data.check_for_updates {
            let settings_bg = app_frame.settings.clone();
            let screen_bg = app_frame.screen.clone();
            let tolk_bg = app_frame.tolk.clone();
            let last_check = settings_data.last_update_check_timestamp;

            std::thread::spawn(move || {
                let cur_ver = env!("APP_VERSION");
                let (status, now) = updater::check_latest_release(false, cur_ver, last_check);
                {
                    let mut s = settings_bg.lock().unwrap();
                    s.last_update_check_timestamp = now;
                    s.save();
                }
                if let UpdateStatus::Available(info) = status {
                    let current_scr = screen_bg.lock().unwrap().clone();
                    if current_scr != AppScreen::InGame {
                        tolk_bg.speak(
                            format!(
                                "Update available! Version {} is now available. Select Update in Main Menu to view release notes.",
                                info.version
                            ),
                            true,
                        );
                    }
                }
            });
        }

        app_frame.render_screen(true, true);
        app_frame
    }

    pub fn show(&self) {
        self.frame.show(true);
        self.timer.borrow_mut().start(16, false);
    }

    pub fn render_screen(&self, speak: bool, initial_load: bool) {
        let screen = self.screen.lock().unwrap().clone();
        let s = self.settings.lock().unwrap();
        let in_prog = *self.game_in_progress.lock().unwrap();

        let (display_text, spoken_text) = match screen {
            AppScreen::MainMenu { selection } => main_menu::render_main_menu(selection, in_prog),
            AppScreen::PauseMenu { selection } => pause_menu::render_pause_menu(selection),
            AppScreen::SaveScreen { selection } => {
                let slots = self.db.get_all_save_slots();
                save_screen::render_save_screen(selection, &slots)
            }
            AppScreen::LoadScreen { selection } => {
                let slots = self.db.get_all_save_slots();
                load_screen::render_load_screen(selection, &slots)
            }
            AppScreen::Leaderboard { selection } => {
                let scores = self.db.get_high_scores(10);
                let stats = self.db.get_player_stats();
                leaderboard::render_leaderboard(selection, &scores, &stats)
            }
            AppScreen::Settings { selection } => settings_screen::render_settings(selection, &s),
            AppScreen::SpeechVerbosity { selection } => {
                settings_screen::render_speech_verbosity(selection, &s)
            }
            AppScreen::HowToPlay { scroll_line } => {
                how_to_play::render_how_to_play(scroll_line, initial_load)
            }
            AppScreen::About { scroll_line } => about_screen::render_about(scroll_line, initial_load),
            AppScreen::Update { selection, ref status } => {
                update_screen::render_update_screen(selection, env!("APP_VERSION"), status)
            }
            AppScreen::ConfirmDialog { action } => confirm_dialog::render_confirm_dialog(action.clone()),
            AppScreen::InGame => {
                let gs = self.game_state.lock().unwrap();
                in_game_screen::render_in_game(&gs)
            }
            AppScreen::KeyDescriber { .. } => (
                "Keyboard Help Mode\nPress any key to hear its function.\nPress Escape twice to exit."
                    .to_string(),
                "".to_string(),
            ),
        };

        self.text_display.set_label(&display_text);

        if speak && !spoken_text.is_empty() {
            self.tolk.output(&spoken_text, true);
        }
    }

    pub fn setup_events(&mut self) {
        let text_ctrl = self.text_display;
        let game_state = self.game_state.clone();
        let audio_engine = self.audio_engine.clone();
        let tolk_instance = self.tolk.clone();
        let settings = self.settings.clone();
        let screen_state = self.screen.clone();
        let game_in_progress = self.game_in_progress.clone();
        let db = self.db.clone();
        let frame = self.frame;

        let render_in_closure = {
            let screen_state = screen_state.clone();
            let settings = settings.clone();
            let game_state = game_state.clone();
            let game_in_progress = game_in_progress.clone();
            let tolk = tolk_instance.clone();
            let db = db.clone();

            move |speak: bool, initial_load: bool| {
                let screen = screen_state.lock().unwrap().clone();
                let s = settings.lock().unwrap();
                let in_prog = *game_in_progress.lock().unwrap();

                let (display_text, spoken_text) = match screen {
                    AppScreen::MainMenu { selection } => main_menu::render_main_menu(selection, in_prog),
                    AppScreen::PauseMenu { selection } => pause_menu::render_pause_menu(selection),
                    AppScreen::SaveScreen { selection } => {
                        let slots = db.get_all_save_slots();
                        save_screen::render_save_screen(selection, &slots)
                    }
                    AppScreen::LoadScreen { selection } => {
                        let slots = db.get_all_save_slots();
                        load_screen::render_load_screen(selection, &slots)
                    }
                    AppScreen::Leaderboard { selection } => {
                        let scores = db.get_high_scores(10);
                        let stats = db.get_player_stats();
                        leaderboard::render_leaderboard(selection, &scores, &stats)
                    }
                    AppScreen::Settings { selection } => {
                        settings_screen::render_settings(selection, &s)
                    }
                    AppScreen::SpeechVerbosity { selection } => {
                        settings_screen::render_speech_verbosity(selection, &s)
                    }
                    AppScreen::HowToPlay { scroll_line } => {
                        how_to_play::render_how_to_play(scroll_line, initial_load)
                    }
                    AppScreen::About { scroll_line } => {
                        about_screen::render_about(scroll_line, initial_load)
                    }
                    AppScreen::Update { selection, ref status } => {
                        update_screen::render_update_screen(selection, env!("APP_VERSION"), status)
                    }
                    AppScreen::ConfirmDialog { action } => {
                        confirm_dialog::render_confirm_dialog(action)
                    }
                    AppScreen::InGame => {
                        let gs = game_state.lock().unwrap();
                        in_game_screen::render_in_game(&gs)
                    }
                    AppScreen::KeyDescriber { .. } => (
                        "Keyboard Help Mode\nPress any key to hear its function.\nPress Escape twice to exit.".to_string(),
                        "".to_string(),
                    ),
                };

                text_ctrl.set_label(&display_text);
                if speak && !spoken_text.is_empty() {
                    tolk.output(&spoken_text, true);
                }
            }
        };

        // 1. KEY DOWN HANDLER
        let on_action = {
            let game_state = game_state.clone();
            let audio_engine = audio_engine.clone();
            let tolk = tolk_instance.clone();
            let settings = settings.clone();
            let screen_state = screen_state.clone();
            let game_in_progress = game_in_progress.clone();
            let render_in_closure = render_in_closure.clone();
            let db = db.clone();

            Rc::new(RefCell::new(move |action: InputAction| {
                let current_screen = screen_state.lock().unwrap().clone();
                let mut screen_changed = false;
                let mut is_initial_load = false;

                // Global Music Controls (except in KeyDescriber mode)
                if !matches!(current_screen, AppScreen::KeyDescriber { .. }) {
                    if action == InputAction::NextTrack {
                        let track = audio_engine.next_track();
                        tolk.output(format!("Playing {}", track), true);
                        return;
                    } else if action == InputAction::PrevTrack {
                        let track = audio_engine.prev_track();
                        tolk.output(format!("Playing {}", track), true);
                        return;
                    } else if action == InputAction::Mute {
                        let is_muted = audio_engine.toggle_mute();
                        if is_muted {
                            tolk.output("Music Muted", true);
                        } else {
                            tolk.output("Music Unmuted", true);
                        }
                        return;
                    }
                }

                // Global Help Mode (H key)
                if action == InputAction::HelpMode {
                    audio_engine.play_menu_select();
                    *screen_state.lock().unwrap() = AppScreen::KeyDescriber { esc_count: 0 };
                    tolk.output(
                        "Keyboard Help Mode. Press any key to hear its function. Press Escape twice to exit.",
                        true,
                    );
                    render_in_closure(true, false);
                    return;
                }

                // Global START button logic for Quick Settings / Pause / Resume
                if action == InputAction::Start {
                    if current_screen == AppScreen::InGame {
                        audio_engine.play_menu_select();
                        tolk.output("Game Paused", true);
                        *screen_state.lock().unwrap() = AppScreen::PauseMenu { selection: 0 };
                        screen_changed = true;
                    } else if let AppScreen::PauseMenu { .. } = current_screen {
                        audio_engine.play_menu_select();
                        tolk.output("Game Resumed", true);
                        *screen_state.lock().unwrap() = AppScreen::InGame;
                        screen_changed = true;
                    }
                    if screen_changed {
                        render_in_closure(true, true);
                    }
                    return;
                }

                match current_screen {
                    AppScreen::MainMenu { selection } => {
                        let in_prog = *game_in_progress.lock().unwrap();
                        let options_count = main_menu::get_main_menu_options(in_prog).len();

                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 {
                                    selection - 1
                                } else {
                                    options_count - 1
                                };
                                *screen_state.lock().unwrap() =
                                    AppScreen::MainMenu { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < options_count - 1 {
                                    selection + 1
                                } else {
                                    0
                                };
                                *screen_state.lock().unwrap() =
                                    AppScreen::MainMenu { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                if in_prog {
                                    match selection {
                                        0 => {
                                            tolk.output("Game Resumed", true);
                                            *screen_state.lock().unwrap() = AppScreen::InGame;
                                        }
                                        1 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::ConfirmDialog {
                                                    action: ConfirmAction::NewGame,
                                                };
                                        }
                                        2 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::SaveScreen { selection: 0 };
                                        }
                                        3 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::LoadScreen { selection: 0 };
                                        }
                                        4 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Leaderboard { selection: 0 };
                                        }
                                        5 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::HowToPlay { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        6 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Settings { selection: 0 };
                                        }
                                        7 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::About { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        8 => {
                                            *screen_state.lock().unwrap() = AppScreen::Update {
                                                selection: 0,
                                                status: UpdateStatus::Idle,
                                            };
                                        }
                                        9 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::ConfirmDialog {
                                                    action: ConfirmAction::QuitApp,
                                                };
                                        }
                                        _ => {}
                                    }
                                } else {
                                    match selection {
                                        0 => {
                                            let diff = settings.lock().unwrap().difficulty;
                                            let mut gs = game_state.lock().unwrap();
                                            *gs = GameState::new(diff);
                                            tolk.output("New Game Started!", true);
                                            let callout_tech =
                                                settings.lock().unwrap().piece_callouts_technical;
                                            tolk.output(
                                                format!(
                                                    "{} spawned",
                                                    gs.current_piece.t_type.as_str(callout_tech)
                                                ),
                                                false,
                                            );
                                            audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                            *game_in_progress.lock().unwrap() = true;
                                            *screen_state.lock().unwrap() = AppScreen::InGame;
                                        }
                                        1 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::LoadScreen { selection: 0 };
                                        }
                                        2 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Leaderboard { selection: 0 };
                                        }
                                        3 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::HowToPlay { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        4 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Settings { selection: 0 };
                                        }
                                        5 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::About { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        6 => {
                                            *screen_state.lock().unwrap() = AppScreen::Update {
                                                selection: 0,
                                                status: UpdateStatus::Idle,
                                            };
                                        }
                                        7 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::ConfirmDialog {
                                                    action: ConfirmAction::QuitApp,
                                                };
                                        }
                                        _ => {}
                                    }
                                }
                                screen_changed = true;
                            }
                            InputAction::HelpMode => {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::KeyDescriber { esc_count: 0 };
                                tolk.output("Keyboard Help Mode. Press any key to hear its function. Press Escape twice to exit.", true);
                                screen_changed = true;
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                if in_prog {
                                    tolk.output("Game Resumed", true);
                                    *screen_state.lock().unwrap() = AppScreen::InGame;
                                    screen_changed = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    AppScreen::PauseMenu { selection } => {
                        let options_count = pause_menu::get_pause_menu_options().len();
                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 {
                                    selection - 1
                                } else {
                                    options_count - 1
                                };
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < options_count - 1 {
                                    selection + 1
                                } else {
                                    0
                                };
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                match selection {
                                    0 => {
                                        tolk.output("Game Resumed", true);
                                        *screen_state.lock().unwrap() = AppScreen::InGame;
                                    }
                                    1 => {
                                        *screen_state.lock().unwrap() =
                                            AppScreen::SaveScreen { selection: 0 };
                                    }
                                    2 => {
                                        *screen_state.lock().unwrap() = AppScreen::ConfirmDialog {
                                            action: ConfirmAction::AbandonGame,
                                        };
                                    }
                                    3 => {
                                        *screen_state.lock().unwrap() =
                                            AppScreen::HowToPlay { scroll_line: 0 };
                                        is_initial_load = true;
                                    }
                                    4 => {
                                        *screen_state.lock().unwrap() =
                                            AppScreen::Settings { selection: 0 };
                                    }
                                    _ => {}
                                }
                                screen_changed = true;
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                tolk.output("Game Resumed", true);
                                *screen_state.lock().unwrap() = AppScreen::InGame;
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::SaveScreen { selection } => match action {
                        InputAction::Up => {
                            let new_sel = if selection > 0 { selection - 1 } else { 5 };
                            *screen_state.lock().unwrap() =
                                AppScreen::SaveScreen { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Down => {
                            let new_sel = if selection < 5 { selection + 1 } else { 0 };
                            *screen_state.lock().unwrap() =
                                AppScreen::SaveScreen { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Select => {
                            audio_engine.play_menu_select();
                            if selection < 5 {
                                let slot_id = selection + 1;
                                let gs = game_state.lock().unwrap();
                                if let Err(e) = db.save_slot(slot_id, &gs) {
                                    tolk.output(format!("Failed to save game: {}", e), true);
                                } else {
                                    tolk.output(format!("Game Saved to Slot {}", slot_id), true);
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 1 };
                                }
                            } else {
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: 1 };
                            }
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            *screen_state.lock().unwrap() = AppScreen::PauseMenu { selection: 1 };
                            screen_changed = true;
                        }
                        _ => {}
                    },
                    AppScreen::LoadScreen { selection } => match action {
                        InputAction::Up => {
                            let new_sel = if selection > 0 { selection - 1 } else { 5 };
                            *screen_state.lock().unwrap() =
                                AppScreen::LoadScreen { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Down => {
                            let new_sel = if selection < 5 { selection + 1 } else { 0 };
                            *screen_state.lock().unwrap() =
                                AppScreen::LoadScreen { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Select => {
                            audio_engine.play_menu_select();
                            if selection < 5 {
                                let slot_id = selection + 1;
                                match db.load_slot(slot_id) {
                                    Ok(loaded_gs) => {
                                        let mut gs = game_state.lock().unwrap();
                                        *gs = loaded_gs;
                                        *game_in_progress.lock().unwrap() = true;
                                        tolk.output(
                                            format!("Game Loaded from Slot {}", slot_id),
                                            true,
                                        );
                                        audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                        *screen_state.lock().unwrap() = AppScreen::InGame;
                                    }
                                    Err(_) => {
                                        audio_engine.play_hold_denied_sound();
                                        tolk.output(format!("Slot {} is Empty", slot_id), true);
                                    }
                                }
                            } else {
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 2 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 1 };
                                }
                            }
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            let in_prog = *game_in_progress.lock().unwrap();
                            if in_prog {
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: 2 };
                            } else {
                                *screen_state.lock().unwrap() =
                                    AppScreen::MainMenu { selection: 1 };
                            }
                            screen_changed = true;
                        }
                        _ => {}
                    },
                    AppScreen::Leaderboard { selection } => {
                        let scores = db.get_high_scores(10);
                        let stats = db.get_player_stats();
                        let items_count = leaderboard::get_leaderboard_items_count(&scores);
                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 {
                                    selection - 1
                                } else {
                                    items_count - 1
                                };
                                *screen_state.lock().unwrap() =
                                    AppScreen::Leaderboard { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < items_count - 1 {
                                    selection + 1
                                } else {
                                    0
                                };
                                *screen_state.lock().unwrap() =
                                    AppScreen::Leaderboard { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                if selection == items_count - 1 {
                                    let in_prog = *game_in_progress.lock().unwrap();
                                    if in_prog {
                                        *screen_state.lock().unwrap() =
                                            AppScreen::MainMenu { selection: 4 };
                                    } else {
                                        *screen_state.lock().unwrap() =
                                            AppScreen::MainMenu { selection: 2 };
                                    }
                                } else {
                                    let (_disp, spoken) =
                                        leaderboard::render_leaderboard(selection, &scores, &stats);
                                    tolk.output(spoken, true);
                                }
                                screen_changed = true;
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 4 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 2 };
                                }
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::Settings { selection } => match action {
                        InputAction::Up => {
                            let new_sel = if selection > 0 { selection - 1 } else { 7 };
                            *screen_state.lock().unwrap() =
                                AppScreen::Settings { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Down => {
                            let new_sel = if selection < 7 { selection + 1 } else { 0 };
                            *screen_state.lock().unwrap() =
                                AppScreen::Settings { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Left => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.difficulty = match s.difficulty {
                                    Difficulty::Easy => Difficulty::Difficult,
                                    Difficulty::Moderate => Difficulty::Easy,
                                    Difficulty::Difficult => Difficulty::Moderate,
                                };
                                audio_engine.play_menu_move();
                                tolk.speak(format!("Difficulty {}", s.difficulty.as_str()), true);
                            } else if selection == 2 {
                                s.voice_volume = (s.voice_volume - 0.05).max(0.0);
                                tolk.speak(
                                    format!("Voice Volume {}%", (s.voice_volume * 100.0) as i32),
                                    true,
                                );
                            } else if selection == 3 {
                                s.sfx_volume = (s.sfx_volume - 0.05).max(0.0);
                                audio_engine.set_sfx_volume(s.sfx_volume);
                                audio_engine.play_aligned_sound();
                            } else if selection == 4 {
                                s.bgm_enabled = !s.bgm_enabled;
                                if s.bgm_enabled {
                                    s.bgm_volume = if s.saved_bgm_volume > 0.0 {
                                        s.saved_bgm_volume
                                    } else {
                                        0.2
                                    };
                                    audio_engine.set_bgm_volume(s.bgm_volume);
                                } else {
                                    s.saved_bgm_volume = s.bgm_volume;
                                    s.bgm_volume = 0.0;
                                    audio_engine.set_bgm_volume(0.0);
                                }
                                audio_engine.set_bgm_enabled(s.bgm_enabled);
                                let status = if s.bgm_enabled { "On" } else { "Off" };
                                tolk.speak(format!("Background Music {}", status), true);
                            } else if selection == 5 {
                                s.bgm_volume = (s.bgm_volume - 0.05).max(0.0);
                                audio_engine.set_bgm_volume(s.bgm_volume);
                                if !s.bgm_enabled && s.bgm_volume > 0.0 {
                                    s.bgm_enabled = true;
                                    audio_engine.set_bgm_enabled(true);
                                }
                                tolk.speak(
                                    format!(
                                        "Background Music Volume {}%",
                                        (s.bgm_volume * 100.0) as i32
                                    ),
                                    true,
                                );
                            } else if selection == 6 {
                                s.check_for_updates = !s.check_for_updates;
                                let status = if s.check_for_updates { "On" } else { "Off" };
                                audio_engine.play_menu_move();
                                tolk.speak(format!("Auto Update Notifications {}", status), true);
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Right => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.difficulty = match s.difficulty {
                                    Difficulty::Easy => Difficulty::Moderate,
                                    Difficulty::Moderate => Difficulty::Difficult,
                                    Difficulty::Difficult => Difficulty::Easy,
                                };
                                audio_engine.play_menu_move();
                                tolk.speak(format!("Difficulty {}", s.difficulty.as_str()), true);
                            } else if selection == 2 {
                                s.voice_volume = (s.voice_volume + 0.05).min(1.0);
                                tolk.speak(
                                    format!("Voice Volume {}%", (s.voice_volume * 100.0) as i32),
                                    true,
                                );
                            } else if selection == 3 {
                                s.sfx_volume = (s.sfx_volume + 0.05).min(1.0);
                                audio_engine.set_sfx_volume(s.sfx_volume);
                                audio_engine.play_aligned_sound();
                            } else if selection == 4 {
                                s.bgm_enabled = !s.bgm_enabled;
                                if s.bgm_enabled {
                                    s.bgm_volume = if s.saved_bgm_volume > 0.0 {
                                        s.saved_bgm_volume
                                    } else {
                                        0.2
                                    };
                                    audio_engine.set_bgm_volume(s.bgm_volume);
                                } else {
                                    s.saved_bgm_volume = s.bgm_volume;
                                    s.bgm_volume = 0.0;
                                    audio_engine.set_bgm_volume(0.0);
                                }
                                audio_engine.set_bgm_enabled(s.bgm_enabled);
                                let status = if s.bgm_enabled { "On" } else { "Off" };
                                tolk.speak(format!("Background Music {}", status), true);
                            } else if selection == 5 {
                                s.bgm_volume = (s.bgm_volume + 0.05).min(1.0);
                                audio_engine.set_bgm_volume(s.bgm_volume);
                                if !s.bgm_enabled && s.bgm_volume > 0.0 {
                                    s.bgm_enabled = true;
                                    audio_engine.set_bgm_enabled(true);
                                }
                                tolk.speak(
                                    format!(
                                        "Background Music Volume {}%",
                                        (s.bgm_volume * 100.0) as i32
                                    ),
                                    true,
                                );
                            } else if selection == 6 {
                                s.check_for_updates = !s.check_for_updates;
                                let status = if s.check_for_updates { "On" } else { "Off" };
                                audio_engine.play_menu_move();
                                tolk.speak(format!("Auto Update Notifications {}", status), true);
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Select | InputAction::Back => {
                            if selection == 1 && action == InputAction::Select {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::SpeechVerbosity { selection: 0 };
                                screen_changed = true;
                            } else if selection == 7 || action == InputAction::Back {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 4 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 4 };
                                }
                                screen_changed = true;
                            }
                        }
                        _ => {}
                    },
                    AppScreen::SpeechVerbosity { selection } => match action {
                        InputAction::Up => {
                            let new_sel = if selection > 0 { selection - 1 } else { 3 };
                            *screen_state.lock().unwrap() =
                                AppScreen::SpeechVerbosity { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Down => {
                            let new_sel = if selection < 3 { selection + 1 } else { 0 };
                            *screen_state.lock().unwrap() =
                                AppScreen::SpeechVerbosity { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Left | InputAction::Right | InputAction::Select => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.piece_callouts_technical = !s.piece_callouts_technical;
                                let status = if s.piece_callouts_technical {
                                    "Terse"
                                } else {
                                    "Descriptive"
                                };
                                tolk.speak(format!("Piece Callouts: {}", status), true);
                            } else if selection == 1 {
                                s.scoring_details_advanced = !s.scoring_details_advanced;
                                let status = if s.scoring_details_advanced {
                                    "Advanced"
                                } else {
                                    "Simple"
                                };
                                tolk.speak(format!("Scoring Details: {}", status), true);
                            } else if selection == 2 {
                                s.zone_alerts = !s.zone_alerts;
                                let status = if s.zone_alerts { "On" } else { "Off" };
                                tolk.speak(format!("Zone Alerts: {}", status), true);
                            } else if selection == 3 && action == InputAction::Select {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::Settings { selection: 1 };
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            *screen_state.lock().unwrap() = AppScreen::Settings { selection: 1 };
                            screen_changed = true;
                        }
                        _ => {}
                    },
                    AppScreen::HowToPlay { scroll_line } => {
                        let lines_count = how_to_play::get_how_to_play_lines().len();
                        match action {
                            InputAction::Up => {
                                if scroll_line > 0 {
                                    *screen_state.lock().unwrap() = AppScreen::HowToPlay {
                                        scroll_line: scroll_line - 1,
                                    };
                                    audio_engine.play_menu_move();
                                    screen_changed = true;
                                }
                            }
                            InputAction::Down => {
                                if scroll_line < lines_count - 1 {
                                    *screen_state.lock().unwrap() = AppScreen::HowToPlay {
                                        scroll_line: scroll_line + 1,
                                    };
                                    audio_engine.play_menu_move();
                                    screen_changed = true;
                                }
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                let full_text = how_to_play::get_how_to_play_lines().join(" ");
                                tolk.speak(full_text, true);
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 3 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 3 };
                                }
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::About { scroll_line } => {
                        let lines_count = about_screen::get_about_lines().len();
                        match action {
                            InputAction::Up => {
                                if scroll_line > 0 {
                                    *screen_state.lock().unwrap() = AppScreen::About {
                                        scroll_line: scroll_line - 1,
                                    };
                                    audio_engine.play_menu_move();
                                    screen_changed = true;
                                }
                            }
                            InputAction::Down => {
                                if scroll_line < lines_count - 1 {
                                    *screen_state.lock().unwrap() = AppScreen::About {
                                        scroll_line: scroll_line + 1,
                                    };
                                    audio_engine.play_menu_move();
                                    screen_changed = true;
                                }
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                let full_text = about_screen::get_about_lines().join(" ");
                                tolk.speak(full_text, true);
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 7 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 5 };
                                }
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::Update {
                        selection,
                        ref status,
                    } => {
                        let options_count = match status {
                            UpdateStatus::Available(_) => 3,
                            _ => 2,
                        };

                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 {
                                    selection - 1
                                } else {
                                    options_count - 1
                                };
                                *screen_state.lock().unwrap() = AppScreen::Update {
                                    selection: new_sel,
                                    status: status.clone(),
                                };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < options_count - 1 {
                                    selection + 1
                                } else {
                                    0
                                };
                                *screen_state.lock().unwrap() = AppScreen::Update {
                                    selection: new_sel,
                                    status: status.clone(),
                                };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                match (status, selection) {
                                    (UpdateStatus::Available(info), 0) => {
                                        *screen_state.lock().unwrap() = AppScreen::ConfirmDialog {
                                            action: ConfirmAction::UpdateApp(info.clone()),
                                        };
                                        screen_changed = true;
                                    }
                                    (UpdateStatus::Available(_), 1) | (_, 0) => {
                                        *screen_state.lock().unwrap() = AppScreen::Update {
                                            selection: 0,
                                            status: UpdateStatus::Checking,
                                        };
                                        screen_changed = true;

                                        let screen_state_bg = screen_state.clone();
                                        let settings_bg = settings.clone();
                                        let tolk_bg = tolk.clone();

                                        std::thread::spawn(move || {
                                            let cur_ver = env!("APP_VERSION");
                                            let last_check = settings_bg
                                                .lock()
                                                .unwrap()
                                                .last_update_check_timestamp;
                                            let (new_status, now) = updater::check_latest_release(
                                                true, cur_ver, last_check,
                                            );

                                            {
                                                let mut s = settings_bg.lock().unwrap();
                                                s.last_update_check_timestamp = now;
                                                s.save();
                                            }

                                            let mut scr = screen_state_bg.lock().unwrap();
                                            if let AppScreen::Update { ref mut status, .. } = *scr {
                                                *status = new_status.clone();
                                            }
                                            match new_status {
                                                UpdateStatus::Available(info) => {
                                                    tolk_bg.speak(
                                                        format!(
                                                            "Update Available: Version {}",
                                                            info.version
                                                        ),
                                                        true,
                                                    );
                                                }
                                                UpdateStatus::UpToDate => {
                                                    tolk_bg.speak(
                                                        "You are using the latest version of Audio Tetris.",
                                                        true,
                                                    );
                                                }
                                                UpdateStatus::Error(e) => {
                                                    tolk_bg.speak(
                                                        format!(
                                                            "Error checking for updates: {}",
                                                            e
                                                        ),
                                                        true,
                                                    );
                                                }
                                                _ => {}
                                            }
                                        });
                                    }
                                    (UpdateStatus::Available(_), 2) | (_, 1) => {
                                        let in_prog = *game_in_progress.lock().unwrap();
                                        let back_sel = if in_prog { 8 } else { 6 };
                                        *screen_state.lock().unwrap() = AppScreen::MainMenu {
                                            selection: back_sel,
                                        };
                                        screen_changed = true;
                                    }
                                    _ => {}
                                }
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                let back_sel = if in_prog { 8 } else { 6 };
                                *screen_state.lock().unwrap() = AppScreen::MainMenu {
                                    selection: back_sel,
                                };
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::ConfirmDialog {
                        action: confirm_act,
                    } => match action {
                        InputAction::Select => {
                            audio_engine.play_menu_select();
                            match confirm_act {
                                ConfirmAction::NewGame => {
                                    let diff = settings.lock().unwrap().difficulty;
                                    let mut gs = game_state.lock().unwrap();
                                    *gs = GameState::new(diff);
                                    tolk.output("New Game Started!", true);
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                    *game_in_progress.lock().unwrap() = true;
                                    *screen_state.lock().unwrap() = AppScreen::InGame;
                                }
                                ConfirmAction::AbandonGame => {
                                    *game_in_progress.lock().unwrap() = false;
                                    tolk.output("Game Abandoned", true);
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                }
                                ConfirmAction::QuitApp => {
                                    frame.close(true);
                                    return;
                                }
                                ConfirmAction::UpdateApp(ref info) => {
                                    tolk.output("Downloading update...", true);
                                    let download_url = info.download_url.clone();
                                    *screen_state.lock().unwrap() = AppScreen::Update {
                                        selection: 0,
                                        status: UpdateStatus::Downloading,
                                    };
                                    render_in_closure(true, false);

                                    let tolk_bg = tolk.clone();
                                    std::thread::spawn(move || {
                                        if let Err(e) =
                                            updater::perform_in_place_update(&download_url)
                                        {
                                            tolk_bg.speak(format!("Update failed: {}", e), true);
                                        }
                                    });
                                }
                            }
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            let in_prog = *game_in_progress.lock().unwrap();
                            if in_prog {
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: 2 };
                            } else {
                                *screen_state.lock().unwrap() =
                                    AppScreen::MainMenu { selection: 0 };
                            }
                            screen_changed = true;
                        }
                        _ => {}
                    },
                    AppScreen::InGame => {
                        let mut gs = game_state.lock().unwrap();

                        match action {
                            InputAction::Back => {
                                tolk.output("Game Paused", true);
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: 0 };
                                screen_changed = true;
                            }
                            InputAction::Radar => {
                                let max_h = gs.max_column_height();
                                audio_engine.play_radar_sweep(gs.get_topography());
                                tolk.output(
                                    format!("Radar sweep: highest stack height {}", max_h),
                                    true,
                                );
                            }
                            InputAction::Left => {
                                if gs.move_left() {
                                    audio_engine.play_horizontal_move_sound(gs.current_piece.x);
                                    if gs.current_piece.x == 0 || gs.current_piece.x == 9 {
                                        audio_engine.play_aligned_sound();
                                    }
                                    tolk.output(
                                        format!("Left, column {}", gs.current_piece.x + 1),
                                        true,
                                    );
                                } else {
                                    audio_engine.play_aligned_sound();
                                }
                            }
                            InputAction::Right => {
                                if gs.move_right() {
                                    audio_engine.play_horizontal_move_sound(gs.current_piece.x);
                                    if gs.current_piece.x == 0 || gs.current_piece.x == 9 {
                                        audio_engine.play_aligned_sound();
                                    }
                                    tolk.output(
                                        format!("Right, column {}", gs.current_piece.x + 1),
                                        true,
                                    );
                                } else {
                                    audio_engine.play_aligned_sound();
                                }
                            }
                            InputAction::RotateRight => {
                                if gs.rotate_cw() {
                                    audio_engine.play_rotate_cw_sound(gs.current_piece.y);
                                    let rot_deg = match gs.current_piece.rotation {
                                        0 => "0 degrees",
                                        1 => "90 degrees",
                                        2 => "180 degrees",
                                        3 => "270 degrees",
                                        _ => "",
                                    };
                                    let left = gs.current_piece.left_column() + 1;
                                    let right = gs.current_piece.right_column() + 1;
                                    tolk.output(
                                        format!(
                                            "Rotated Right, {}. Columns {} through {}",
                                            rot_deg, left, right
                                        ),
                                        true,
                                    );
                                } else {
                                    audio_engine.play_aligned_sound();
                                }
                            }
                            InputAction::RotateLeft => {
                                if gs.rotate_ccw() {
                                    audio_engine.play_rotate_ccw_sound(gs.current_piece.y);
                                    let rot_deg = match gs.current_piece.rotation {
                                        0 => "0 degrees",
                                        1 => "90 degrees",
                                        2 => "180 degrees",
                                        3 => "270 degrees",
                                        _ => "",
                                    };
                                    let left = gs.current_piece.left_column() + 1;
                                    let right = gs.current_piece.right_column() + 1;
                                    tolk.output(
                                        format!(
                                            "Rotated Left, {}. Columns {} through {}",
                                            rot_deg, left, right
                                        ),
                                        true,
                                    );
                                } else {
                                    audio_engine.play_aligned_sound();
                                }
                            }
                            InputAction::PieceInfo => {
                                let callout_tech =
                                    settings.lock().unwrap().piece_callouts_technical;
                                let name = gs.current_piece.t_type.as_str(callout_tech);
                                let left = gs.current_piece.left_column();
                                let right = gs.current_piece.right_column();
                                let width = gs.current_piece.width();
                                let mut text = format!(
                                    "Current piece: {}. Columns {} through {}. Width: {}.",
                                    name, left, right, width
                                );
                                if let Some(held) = gs.hold_piece {
                                    text.push_str(&format!(
                                        " Held piece: {}.",
                                        held.as_str(callout_tech)
                                    ));
                                } else {
                                    text.push_str(" Held piece: None.");
                                }
                                tolk.output(text, true);
                            }
                            InputAction::Hold => {
                                let callout_tech =
                                    settings.lock().unwrap().piece_callouts_technical;
                                if let Some((swapped, prev, new_p)) = gs.hold() {
                                    if swapped {
                                        audio_engine.play_hold_swap_sound();
                                    } else {
                                        audio_engine.play_hold_sound();
                                    }
                                    tolk.output(
                                        format!(
                                            "Held {}. New piece: {}",
                                            prev.as_str(callout_tech),
                                            new_p.as_str(callout_tech)
                                        ),
                                        true,
                                    );
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.output("Already held piece this turn", true);
                                }
                            }
                            InputAction::Select => {}
                            InputAction::Down => {
                                if gs.soft_drop() {
                                    audio_engine.play_soft_drop_sound(gs.current_piece.y);
                                    tolk.output(
                                        format!("Soft drop, row {}", gs.current_piece.y + 1),
                                        true,
                                    );
                                }
                            }
                            InputAction::HardDrop => {
                                audio_engine.play_hard_drop_sound();
                                let res = gs.hard_drop();
                                let settings_lock = settings.lock().unwrap();
                                let scoring_advanced = settings_lock.scoring_details_advanced;
                                let zone_alerts = settings_lock.zone_alerts;
                                drop(settings_lock);

                                if res.zone_lines_cleared_this_turn > 0 {
                                    audio_engine.play_clear_sound(res.zone_lines_cleared_this_turn);
                                }

                                if res.zone_meter_full && zone_alerts {
                                    tolk.output("Zone Meter Full!", true);
                                }

                                if res.cleared_lines > 0 {
                                    audio_engine.play_clear_sound(res.cleared_lines);
                                    let mut tts =
                                        format!("Hard drop. Cleared {} lines!", res.cleared_lines);
                                    if res.is_t_spin {
                                        audio_engine.play_t_spin_sound();
                                        if scoring_advanced {
                                            tts.push_str(" T-Spin!");
                                        }
                                    }
                                    if res.b2b_bonus {
                                        audio_engine.play_b2b_sound();
                                        if scoring_advanced {
                                            tts.push_str(" Back to back!");
                                        }
                                    }
                                    if res.combo > 1 && scoring_advanced {
                                        tts.push_str(&format!(" {} Combo!", res.combo));
                                    }
                                    tts.push_str(&format!(
                                        " Level: {}. Score: {}",
                                        gs.level, gs.score
                                    ));
                                    tolk.output(tts, true);
                                } else {
                                    if res.is_t_spin && scoring_advanced {
                                        audio_engine.play_t_spin_sound();
                                        tolk.output(format!("T-Spin! Score: {}", gs.score), true);
                                    }
                                    tolk.output(format!("Hard drop. Score: {}", gs.score), true);
                                }

                                if gs.is_game_over {
                                    let _ = db.record_high_score(&gs);
                                    *game_in_progress.lock().unwrap() = false;
                                    tolk.output(
                                        format!("Game Over! Final Score: {}", gs.score),
                                        true,
                                    );
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                } else {
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                    let callout_tech =
                                        settings.lock().unwrap().piece_callouts_technical;
                                    tolk.output(
                                        gs.current_piece.t_type.as_str(callout_tech),
                                        false,
                                    );
                                    if let Some(acquired) = gs.item_acquired {
                                        audio_engine.play_item_acquire();
                                        tolk.output(
                                            format!("Acquired {}!", acquired.as_str()),
                                            true,
                                        );
                                    }
                                    if let Some(spawned) = gs.item_spawned {
                                        audio_engine.play_item_spawn();
                                        tolk.output(
                                            format!("{} spawned!", spawned.as_str()),
                                            false,
                                        );
                                    }
                                }
                                screen_changed = true;
                            }
                            InputAction::Zone => {
                                if gs.start_zone() {
                                    audio_engine.play_zone_enter();
                                    tolk.output("Zone Activated!", true);
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.output("Not enough charge for Zone", true);
                                }
                            }
                            InputAction::UseItem => {
                                if let Some(item) = gs.use_item() {
                                    audio_engine.play_item_use(item);
                                    tolk.output(format!("Used {}", item.as_str()), true);
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.output("No item to use", true);
                                }
                            }
                            _ => {}
                        }
                    }
                    AppScreen::KeyDescriber { esc_count } => {
                        if action == InputAction::Back {
                            if esc_count == 0 {
                                *screen_state.lock().unwrap() =
                                    AppScreen::KeyDescriber { esc_count: 1 };
                                audio_engine.play_menu_move();
                                tolk.output("Press Escape again to exit Keyboard Help Mode.", true);
                            } else {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 0 };
                                    tolk.output("Exited Keyboard Help Mode.", true);
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                    tolk.output("Exited Keyboard Help Mode.", true);
                                }
                            }
                        } else {
                            *screen_state.lock().unwrap() =
                                AppScreen::KeyDescriber { esc_count: 0 };
                            audio_engine.play_menu_move();
                            let desc = get_action_description(action);
                            tolk.output(desc, true);
                        }
                        screen_changed = true;
                    }
                }

                if screen_changed {
                    render_in_closure(true, is_initial_load);
                }
            }))
        };

        // 2. GAME TICK TIMER
        self.timer.borrow().on_tick({
            let game_state = game_state.clone();
            let audio_engine = audio_engine.clone();
            let tolk = tolk_instance.clone();
            let screen = screen_state.clone();
            let in_prog = game_in_progress.clone();
            let settings = settings.clone();
            let db = db.clone();

            move |_| {
                let interval = 16;

                // --- GAME DROP TIMER ---
                if *screen.lock().unwrap() != AppScreen::InGame {
                    return;
                }
                let mut gs = game_state.lock().unwrap();
                if gs.is_game_over {
                    return;
                }

                if gs.is_in_zone {
                    gs.zone_timer_ms -= interval;
                    if gs.zone_timer_ms <= 0 {
                        let lines = gs.end_zone();
                        if lines > 0 {
                            audio_engine.play_clear_sound(lines);
                            tolk.output(
                                format!("Zone ended! Cleared {} lines. Score: {}", lines, gs.score),
                                true,
                            );
                        } else {
                            tolk.output("Zone ended.", true);
                        }
                    }
                }

                if gs.lock_delay_active {
                    gs.lock_delay_timer_ms -= interval;
                    if gs.lock_delay_timer_ms <= 0 {
                        let res = gs.lock_piece();
                        let settings_lock = settings.lock().unwrap();
                        let scoring_advanced = settings_lock.scoring_details_advanced;
                        let zone_alerts = settings_lock.zone_alerts;
                        drop(settings_lock);

                        if res.zone_lines_cleared_this_turn > 0 {
                            audio_engine.play_clear_sound(res.zone_lines_cleared_this_turn);
                        }

                        if res.zone_meter_full && zone_alerts {
                            tolk.output("Zone Meter Full!", true);
                        }

                        if res.cleared_lines > 0 {
                            audio_engine.play_clear_sound(res.cleared_lines);
                            let mut tts = format!("Cleared {} lines!", res.cleared_lines);
                            if res.is_t_spin {
                                audio_engine.play_t_spin_sound();
                                if scoring_advanced {
                                    tts.push_str(" T-Spin!");
                                }
                            }
                            if res.b2b_bonus {
                                audio_engine.play_b2b_sound();
                                if scoring_advanced {
                                    tts.push_str(" Back to back!");
                                }
                            }
                            if res.combo > 1 && scoring_advanced {
                                tts.push_str(&format!(" {} Combo!", res.combo));
                            }
                            tts.push_str(&format!(" Level: {}. Score: {}", gs.level, gs.score));
                            tolk.output(tts, true);
                        } else {
                            if res.is_t_spin && scoring_advanced {
                                audio_engine.play_t_spin_sound();
                                tolk.output(format!("T-Spin! Score: {}", gs.score), true);
                            }
                            audio_engine.play_lock_sound();
                        }

                        if gs.is_game_over {
                            let _ = db.record_high_score(&gs);
                            *in_prog.lock().unwrap() = false;
                            tolk.output(format!("Game Over! Final Score: {}", gs.score), true);
                            *screen.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                        } else {
                            audio_engine.play_spawn_sound(gs.current_piece.t_type);
                            let callout_tech = settings.lock().unwrap().piece_callouts_technical;
                            tolk.output(gs.current_piece.t_type.as_str(callout_tech), false);
                            if let Some(acquired) = gs.item_acquired {
                                audio_engine.play_item_acquire();
                                tolk.output(format!("Acquired {}!", acquired.as_str()), true);
                            }
                            if let Some(spawned) = gs.item_spawned {
                                audio_engine.play_item_spawn();
                                tolk.output(format!("{} spawned!", spawned.as_str()), false);
                            }
                        }
                        render_in_closure(true, false);
                    }
                } else {
                    gs.fall_timer_ms += interval;
                    let speed = gs.current_speed_ms();
                    if gs.fall_timer_ms >= speed {
                        gs.fall_timer_ms = 0;
                        if !gs.move_down() {
                            gs.lock_delay_active = true;
                            gs.lock_delay_timer_ms = 500;
                        }
                    }
                }

                audio_engine.update_danger_state(gs.max_column_height());
            }
        });

        // 3. KEYBOARD EVENT BINDINGS
        let panel_for_events = self.panel;
        panel_for_events.on_key_down(move |evt| {
            let key_code = match evt {
                wxdragon::event::window_events::WindowEventData::Keyboard(ref kbd_event) => {
                    kbd_event.get_key_code().unwrap_or(0)
                }
                _ => 0,
            };
            let current_screen_val = screen_state.lock().unwrap().clone();
            let action = match key_code {
                315 | 87 | 119 => {
                    if current_screen_val == AppScreen::InGame {
                        None
                    } else {
                        Some(InputAction::Up)
                    }
                }
                317 | 83 | 115 => Some(InputAction::Down), // DOWN Arrow / S
                314 | 65 | 97 => Some(InputAction::Left),  // LEFT Arrow / A
                316 | 68 | 100 => Some(InputAction::Right), // RIGHT Arrow / D
                13 | 370 => Some(InputAction::Select),     // Return / Enter
                27 => Some(InputAction::Back),             // Escape
                32 => Some(InputAction::HardDrop),         // Space ONLY for Hard Drop
                90 | 122 | 44 => Some(InputAction::RotateLeft), // Z, Comma
                88 | 120 | 46 => Some(InputAction::RotateRight), // X, Period
                67 | 99 | 47 => Some(InputAction::Hold),   // C, Slash
                69 | 101 | 76 | 108 => Some(InputAction::Radar), // E, L
                81 | 113 | 75 | 107 => Some(InputAction::Zone), // Q, K
                86 | 118 | 59 | 186 => Some(InputAction::PieceInfo), // V, Semicolon
                306 | 344 | 160 | 161 => Some(InputAction::UseItem), // Shift keys ONLY
                73 | 105 => Some(InputAction::PrevTrack),  // I
                79 | 111 => Some(InputAction::Mute),       // O
                80 | 112 => Some(InputAction::NextTrack),  // P
                72 | 104 => Some(InputAction::HelpMode),   // H
                _ => None,
            };

            if let Some(act) = action {
                on_action.borrow_mut()(act);
            }
        });
    }
}
