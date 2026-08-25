use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wxdragon::color::Colour;
use wxdragon::font::{Font, FontFamily, FontStyle, FontWeight};
use wxdragon::prelude::*;

use crate::audio::AudioEngine;
use crate::db::Database;
use crate::logic::{GameState, ItemType, Tetromino, TetrominoType};
use crate::screens::{
    AppScreen, ConfirmAction, about_screen, confirm_dialog, how_to_play, in_game_screen,
    leaderboard, load_screen, main_menu, pause_menu, save_screen, settings_screen, tutorial_prompt,
    tutorial_screen, update_screen,
};
use crate::settings::{Difficulty, Settings, WindowSizeMode};
use crate::updater::{self, UpdateStatus};
use crate::visuals;
use rust_i18n::t;
use tolk::Tolk;

const SCREEN_READER_TOGGLE_KEY: i32 = 351; // wxWidgets WXK_F12
const SCREEN_READER_TOGGLE_CONFIRM_SECS: u64 = 5;

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

pub fn get_action_description(action: InputAction) -> String {
    match action {
        InputAction::Left => t!("key_describer.left").to_string(),
        InputAction::Right => t!("key_describer.right").to_string(),
        InputAction::Down => t!("key_describer.down").to_string(),
        InputAction::HardDrop => t!("key_describer.hard_drop").to_string(),
        InputAction::RotateLeft => t!("key_describer.rotate_left").to_string(),
        InputAction::RotateRight => t!("key_describer.rotate_right").to_string(),
        InputAction::Hold => t!("key_describer.hold").to_string(),
        InputAction::Radar => t!("key_describer.radar").to_string(),
        InputAction::Zone => t!("key_describer.zone").to_string(),
        InputAction::UseItem => t!("key_describer.use_item").to_string(),
        InputAction::PieceInfo => t!("key_describer.piece_info").to_string(),
        InputAction::PrevTrack => t!("key_describer.prev_track").to_string(),
        InputAction::Mute => t!("key_describer.mute").to_string(),
        InputAction::NextTrack => t!("key_describer.next_track").to_string(),
        InputAction::Start => t!("key_describer.start").to_string(),
        InputAction::HelpMode => t!("key_describer.help_mode").to_string(),
        InputAction::Up => t!("key_describer.up").to_string(),
        InputAction::Select => t!("key_describer.select").to_string(),
        InputAction::Back => t!("key_describer.back").to_string(),
    }
}

fn should_use_menu_music(screen: &AppScreen, game_in_progress: bool) -> bool {
    match screen {
        AppScreen::Tutorial { .. } | AppScreen::InGame => false,
        AppScreen::PauseMenu { .. }
        | AppScreen::SaveScreen { .. }
        | AppScreen::LoadScreen { .. }
            if game_in_progress =>
        {
            false
        }
        AppScreen::ConfirmDialog {
            action: ConfirmAction::AbandonGame,
        } => false,
        _ => true,
    }
}

pub struct ScreenReader {
    inner: Arc<Tolk>,
    enabled: Arc<AtomicBool>,
    visual_status: Arc<Mutex<Option<visuals::VisualStatus>>>,
}

impl ScreenReader {
    pub fn new(enabled: bool, visual_status: Arc<Mutex<Option<visuals::VisualStatus>>>) -> Self {
        Self {
            inner: Tolk::new(),
            enabled: Arc::new(AtomicBool::new(enabled)),
            visual_status,
        }
    }

    pub fn try_sapi(&self, enable: bool) {
        if self.is_enabled() {
            self.inner.try_sapi(enable);
        }
    }

    pub fn output<S: std::fmt::Display>(&self, text: S, interrupt: bool) {
        let text = text.to_string();
        self.publish_visual_status(&text);
        if self.is_enabled() {
            self.inner.output(text, interrupt);
        }
    }

    pub fn speak<S: std::fmt::Display>(&self, text: S, interrupt: bool) {
        let text = text.to_string();
        self.publish_visual_status(&text);
        if self.is_enabled() {
            self.inner.speak(text, interrupt);
        }
    }

    pub fn speak_forced<S: std::fmt::Display>(&self, text: S, interrupt: bool) {
        let text = text.to_string();
        self.publish_visual_status(&text);
        self.inner.speak(text, interrupt);
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    fn publish_visual_status(&self, text: &str) {
        if text.trim().is_empty() {
            return;
        }
        if let Ok(mut status) = self.visual_status.lock() {
            *status = Some(visuals::VisualStatus::new(text));
        }
    }
}

pub struct AppFrame {
    frame: Frame,
    panel: Panel,
    text_display: StaticText,
    game_state: Arc<Mutex<GameState>>,
    audio_engine: Rc<AudioEngine>,
    timer: Rc<RefCell<wxdragon::timer::Timer<Frame>>>,
    tolk: Arc<ScreenReader>,
    settings: Arc<Mutex<Settings>>,
    screen: Arc<Mutex<AppScreen>>,
    game_in_progress: Arc<Mutex<bool>>,
    db: Arc<Database>,
    tutorial_state: Arc<Mutex<tutorial_screen::TutorialState>>,
    visual_assets: Rc<visuals::VisualAssets>,
    visual_text: Arc<Mutex<String>>,
    visual_status: Arc<Mutex<Option<visuals::VisualStatus>>>,
}

impl AppFrame {
    pub fn apply_visual_settings(
        frame: &Frame,
        panel: &Panel,
        text_display: &StaticText,
        settings: &Settings,
    ) {
        let (bg, fg) = settings.get_theme_colors();
        panel.set_background_color(Colour::new(bg.0, bg.1, bg.2, 255));
        text_display.set_foreground_color(Colour::new(fg.0, fg.1, fg.2, 255));

        if let Some(font) = Font::new_with_details(
            settings.font_scale.point_size(),
            FontFamily::Modern.as_i32(),
            FontStyle::Normal.as_i32(),
            FontWeight::Normal.as_i32(),
            false,
            "Consolas",
        ) {
            text_display.set_font(&font);
        }

        match settings.window_size {
            WindowSizeMode::Standard => {
                if frame.is_maximized() {
                    frame.maximize(false);
                }
                frame.set_size(Size::new(800, 600));
                frame.center_on_screen();
            }
            WindowSizeMode::Large => {
                if frame.is_maximized() {
                    frame.maximize(false);
                }
                frame.set_size(Size::new(1280, 720));
                frame.center_on_screen();
            }
            WindowSizeMode::Maximized => {
                if !frame.is_maximized() {
                    frame.maximize(true);
                }
            }
        }

        panel.layout();
        panel.refresh(true, None);
    }

    pub fn new() -> Self {
        let settings_data = Settings::load();
        let settings = Arc::new(Mutex::new(settings_data.clone()));
        let visual_status = Arc::new(Mutex::new(None));
        let tolk = Arc::new(ScreenReader::new(
            settings_data.screen_reader_enabled,
            visual_status.clone(),
        ));
        tolk.try_sapi(true);

        let db = Arc::new(Database::new("audio_tetris.db").expect("Failed to initialize database"));
        let initial_screen = if !settings_data.tutorial_completed {
            AppScreen::TutorialPrompt
        } else {
            AppScreen::MainMenu { selection: 0 }
        };
        let screen = Arc::new(Mutex::new(initial_screen));
        let game_in_progress = Arc::new(Mutex::new(false));
        let tutorial_state = Arc::new(Mutex::new(tutorial_screen::TutorialState::new(1)));

        let title = format!("Audio Tetris v{}", env!("APP_VERSION"));
        let initial_size = match settings_data.window_size {
            WindowSizeMode::Standard => Size::new(800, 600),
            WindowSizeMode::Large => Size::new(1280, 720),
            WindowSizeMode::Maximized => Size::new(800, 600),
        };
        let frame = Frame::builder()
            .with_title(&title)
            .with_size(initial_size)
            .build();
        frame.set_min_size(Size::new(640, 480));
        if settings_data.window_size == WindowSizeMode::Maximized {
            frame.maximize(true);
        } else {
            frame.center_on_screen();
        }

        let panel = Panel::builder(&frame)
            .with_style(PanelStyle::BorderNone)
            .build();
        panel.set_background_style(BackgroundStyle::Paint);

        let sizer = BoxSizer::builder(Orientation::Vertical).build();

        let text_display = StaticText::builder(&panel).with_label("Loading...").build();

        let (bg, fg) = settings_data.get_theme_colors();
        panel.set_background_color(Colour::new(bg.0, bg.1, bg.2, 255));
        text_display.set_foreground_color(Colour::new(fg.0, fg.1, fg.2, 255));
        if let Some(font) = Font::new_with_details(
            settings_data.font_scale.point_size(),
            FontFamily::Modern.as_i32(),
            FontStyle::Normal.as_i32(),
            FontWeight::Normal.as_i32(),
            false,
            "Consolas",
        ) {
            text_display.set_font(&font);
        }

        sizer.add_stretch_spacer(1);
        sizer.add(
            &text_display,
            0,
            SizerFlag::AlignCenterHorizontal | SizerFlag::AlignCenterVertical,
            0,
        );
        sizer.add_stretch_spacer(1);

        panel.set_sizer(sizer, true);

        let game_state = Arc::new(Mutex::new(GameState::new(settings_data.difficulty)));
        let audio_engine = Rc::new(AudioEngine::new(&settings_data).unwrap());
        let timer = Rc::new(RefCell::new(wxdragon::timer::Timer::new(&frame)));
        let visual_assets = Rc::new(visuals::VisualAssets::load());
        let visual_text = Arc::new(Mutex::new(String::new()));

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
            tutorial_state,
            visual_assets,
            visual_text,
            visual_status,
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
                            t!("updater.spoken_update_available", version = &info.version)
                                .to_string(),
                            true,
                        );
                    }
                }
            });
        }

        app_frame.render_screen(false, true);
        app_frame
    }

    pub fn show(&self) {
        self.frame.show(true);
        self.timer.borrow_mut().start(16, false);
        self.render_screen(true, true);

        let tolk = self.tolk.clone();
        let screen = self.screen.clone();
        let tutorial_state = self.tutorial_state.clone();
        let in_prog = *self.game_in_progress.lock().unwrap();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(150));
            let scr = screen.lock().unwrap().clone();
            let spoken = match scr {
                AppScreen::TutorialPrompt => tutorial_prompt::render_tutorial_prompt().1,
                AppScreen::Tutorial { .. } => {
                    let ts = tutorial_state.lock().unwrap();
                    tutorial_screen::render_tutorial(&ts).1
                }
                AppScreen::MainMenu { selection } => {
                    main_menu::render_main_menu(selection, in_prog).1
                }
                _ => String::new(),
            };
            if !spoken.is_empty() {
                tolk.output(&spoken, true);
            }
        });
    }

    pub fn render_screen(&self, speak: bool, initial_load: bool) {
        let screen = self.screen.lock().unwrap().clone();
        let s = self.settings.lock().unwrap();
        let in_prog = *self.game_in_progress.lock().unwrap();
        self.audio_engine
            .set_menu_music_active(should_use_menu_music(&screen, in_prog));

        let (display_text, spoken_text) = match screen {
            AppScreen::TutorialPrompt => tutorial_prompt::render_tutorial_prompt(),
            AppScreen::Tutorial { .. } => {
                let ts = self.tutorial_state.lock().unwrap();
                let (d, s) = tutorial_screen::render_tutorial(&ts);
                let spoken = if initial_load { s } else { String::new() };
                (d, spoken)
            }
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
            AppScreen::VisualSettings { selection } => {
                settings_screen::render_visual_settings(selection, &s)
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
            AppScreen::Update {
                selection,
                ref status,
            } => update_screen::render_update_screen(selection, env!("APP_VERSION"), status),
            AppScreen::ConfirmDialog { ref action } => {
                confirm_dialog::render_confirm_dialog(action.clone())
            }
            AppScreen::InGame => {
                let gs = self.game_state.lock().unwrap();
                in_game_screen::render_in_game(&gs)
            }
            AppScreen::KeyDescriber { .. } => {
                (t!("key_describer.help_title").to_string(), "".to_string())
            }
        };

        if let Ok(mut text) = self.visual_text.lock() {
            *text = display_text;
        }
        self.text_display.set_label("");
        self.panel.layout();
        self.panel.refresh(true, None);

        if speak && !spoken_text.is_empty() {
            let interrupt = !matches!(screen, AppScreen::Update { .. });
            self.tolk.output(&spoken_text, interrupt);
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
        let tutorial_state = self.tutorial_state.clone();
        let frame = self.frame;
        let panel = self.panel;
        let visual_assets = self.visual_assets.clone();
        let visual_text = self.visual_text.clone();
        let visual_status = self.visual_status.clone();
        let screen_reader_toggle_pending: Rc<RefCell<Option<Instant>>> =
            Rc::new(RefCell::new(None));

        let render_in_closure = {
            let screen_state = screen_state.clone();
            let settings = settings.clone();
            let game_state = game_state.clone();
            let game_in_progress = game_in_progress.clone();
            let tolk = tolk_instance.clone();
            let db = db.clone();
            let tutorial_state = tutorial_state.clone();
            let audio_engine = audio_engine.clone();
            let visual_text = visual_text.clone();

            move |speak: bool, initial_load: bool| {
                let screen = screen_state.lock().unwrap().clone();
                let s = settings.lock().unwrap();
                let in_prog = *game_in_progress.lock().unwrap();
                audio_engine.set_menu_music_active(should_use_menu_music(&screen, in_prog));

                let (display_text, spoken_text) = match screen {
                    AppScreen::TutorialPrompt => tutorial_prompt::render_tutorial_prompt(),
                    AppScreen::Tutorial { .. } => {
                        let ts = tutorial_state.lock().unwrap();
                        let (d, s) = tutorial_screen::render_tutorial(&ts);
                        let spoken = if initial_load { s } else { String::new() };
                        (d, spoken)
                    }
                    AppScreen::MainMenu { selection } => {
                        main_menu::render_main_menu(selection, in_prog)
                    }
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
                    AppScreen::VisualSettings { selection } => {
                        settings_screen::render_visual_settings(selection, &s)
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
                    AppScreen::Update {
                        selection,
                        ref status,
                    } => {
                        update_screen::render_update_screen(selection, env!("APP_VERSION"), status)
                    }
                    AppScreen::ConfirmDialog { ref action } => {
                        confirm_dialog::render_confirm_dialog(action.clone())
                    }
                    AppScreen::InGame => {
                        let gs = game_state.lock().unwrap();
                        in_game_screen::render_in_game(&gs)
                    }
                    AppScreen::KeyDescriber { .. } => {
                        (t!("key_describer.help_title").to_string(), "".to_string())
                    }
                };

                if let Ok(mut text) = visual_text.lock() {
                    *text = display_text;
                }
                text_ctrl.set_label("");
                panel.layout();
                panel.refresh(true, None);
                if speak && !spoken_text.is_empty() {
                    let interrupt = !matches!(screen, AppScreen::Update { .. });
                    tolk.output(&spoken_text, interrupt);
                }
            }
        };

        let last_rendered_screen = Rc::new(RefCell::new(screen_state.lock().unwrap().clone()));

        panel.on_paint({
            let panel_for_paint = panel;
            let screen_state = screen_state.clone();
            let game_state = game_state.clone();
            let game_in_progress = game_in_progress.clone();
            let visual_assets = visual_assets.clone();
            let visual_text = visual_text.clone();
            let visual_status = visual_status.clone();

            move |_| {
                let dc = AutoBufferedPaintDC::new(&panel_for_paint);
                let screen = screen_state.lock().unwrap().clone();
                let gs = game_state.lock().unwrap();
                let in_prog = *game_in_progress.lock().unwrap();
                let is_dark_mode = true;
                let display_text = visual_text
                    .lock()
                    .map(|text| text.clone())
                    .unwrap_or_default();
                let status = visual_status.lock().ok().and_then(|status| status.clone());
                visuals::draw_app(
                    &dc,
                    &visual_assets,
                    &screen,
                    &gs,
                    visuals::VisualRenderState {
                        game_in_progress: in_prog,
                        is_dark_mode,
                        display_text: &display_text,
                        visual_status: status.as_ref(),
                    },
                );
            }
        });

        panel.on_size({
            let panel_for_size = panel;
            move |_| {
                panel_for_size.refresh(true, None);
            }
        });

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
            let last_rendered = last_rendered_screen.clone();

            Rc::new(RefCell::new(move |action: InputAction| {
                let current_screen = screen_state.lock().unwrap().clone();
                let was_in_game = current_screen == AppScreen::InGame;
                let mut screen_changed = false;
                let mut is_initial_load = false;

                // Global Music Controls (except in KeyDescriber mode)
                if !matches!(current_screen, AppScreen::KeyDescriber { .. }) {
                    let in_prog = *game_in_progress.lock().unwrap();
                    let allow_track_switch = !should_use_menu_music(&current_screen, in_prog);
                    if action == InputAction::NextTrack && allow_track_switch {
                        let track = audio_engine.next_track();
                        tolk.output(t!("in_game.bgm_playing", track = &track).to_string(), true);
                        return;
                    } else if action == InputAction::PrevTrack && allow_track_switch {
                        let track = audio_engine.prev_track();
                        tolk.output(t!("in_game.bgm_playing", track = &track).to_string(), true);
                        return;
                    } else if action == InputAction::Mute {
                        let mut s = settings.lock().unwrap();
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
                        let on_str = t!("common.on");
                        let off_str = t!("common.off");
                        let status = if s.bgm_enabled { &on_str } else { &off_str };
                        tolk.speak(t!("in_game.bgm_status", status = status).to_string(), true);
                        s.save();
                        if matches!(current_screen, AppScreen::Settings { .. }) {
                            render_in_closure(true, false);
                        }
                        return;
                    }
                }

                if let AppScreen::KeyDescriber { esc_count } = current_screen {
                    if action == InputAction::Back {
                        let new_count = esc_count + 1;
                        if new_count >= 2 {
                            audio_engine.play_menu_select();
                            let in_prog = *game_in_progress.lock().unwrap();
                            if in_prog {
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: 0 };
                            } else {
                                *screen_state.lock().unwrap() =
                                    AppScreen::MainMenu { selection: 0 };
                            }
                            render_in_closure(true, false);
                            tolk.output(t!("key_describer.exited_help_mode").to_string(), true);
                        } else {
                            audio_engine.play_menu_move();
                            *screen_state.lock().unwrap() = AppScreen::KeyDescriber {
                                esc_count: new_count,
                            };
                            tolk.output(t!("key_describer.press_esc_again").to_string(), true);
                        }
                    } else {
                        audio_engine.play_menu_move();
                        *screen_state.lock().unwrap() = AppScreen::KeyDescriber { esc_count: 0 };
                        let desc = get_action_description(action);
                        tolk.output(desc, true);
                    }
                    return;
                }

                // Global START button logic for Quick Settings / Pause / Resume
                if action == InputAction::Start {
                    if current_screen == AppScreen::InGame {
                        audio_engine.play_menu_select();
                        tolk.output(t!("pause_menu.paused").to_string(), true);
                        *screen_state.lock().unwrap() = AppScreen::PauseMenu { selection: 0 };
                        render_in_closure(true, false);
                    } else if let AppScreen::PauseMenu { .. } = current_screen {
                        audio_engine.play_menu_select();
                        tolk.output(t!("pause_menu.resumed").to_string(), true);
                        *screen_state.lock().unwrap() = AppScreen::InGame;
                        render_in_closure(true, false);
                    }
                    return;
                }

                match current_screen {
                    AppScreen::TutorialPrompt => match action {
                        InputAction::Select => {
                            audio_engine.play_menu_select();
                            *screen_state.lock().unwrap() = AppScreen::Tutorial { stage: 1 };
                            *tutorial_state.lock().unwrap() =
                                tutorial_screen::TutorialState::new(1);
                            is_initial_load = true;
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            {
                                let mut s = settings.lock().unwrap();
                                s.tutorial_completed = true;
                                s.save();
                            }
                            *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                            let spoken = main_menu::render_main_menu(0, false).1;
                            tolk.output(&spoken, true);
                            screen_changed = true;
                        }
                        InputAction::Up
                        | InputAction::Down
                        | InputAction::Left
                        | InputAction::Right => {
                            audio_engine.play_menu_move();
                            let (_text, spoken) = tutorial_prompt::render_tutorial_prompt();
                            tolk.output(spoken, true);
                        }
                        _ => {}
                    },
                    AppScreen::Tutorial {
                        stage: current_stage,
                    } => {
                        let mut ts = tutorial_state.lock().unwrap();
                        match action {
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                tolk.output(t!("tutorial.exited").to_string(), true);
                                *screen_state.lock().unwrap() =
                                    AppScreen::MainMenu { selection: 0 };
                                screen_changed = true;
                            }
                            InputAction::Left => {
                                if current_stage == 1 {
                                    if ts.game_state.move_left() {
                                        audio_engine.play_horizontal_move_sound(
                                            ts.game_state.current_piece.x,
                                        );
                                        if ts.game_state.current_piece.x == 0 {
                                            ts.reached_left = true;
                                            audio_engine.play_aligned_sound();
                                            tolk.output(
                                                t!("tutorial.stage_1_left_reached").to_string(),
                                                true,
                                            );
                                        } else {
                                            tolk.output(
                                                t!(
                                                    "in_game.move_left",
                                                    col = (ts.game_state.current_piece.x + 1)
                                                        .to_string()
                                                )
                                                .to_string(),
                                                true,
                                            );
                                        }
                                        if ts.reached_left && ts.reached_right {
                                            ts.stage = 2;
                                            ts.init_stage_board();
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Tutorial { stage: 2 };
                                            audio_engine.play_menu_select();
                                            is_initial_load = true;
                                        }
                                    } else {
                                        audio_engine.play_aligned_sound();
                                    }
                                    screen_changed = true;
                                } else if current_stage == 5 {
                                    if ts.game_state.move_left() {
                                        audio_engine.play_horizontal_move_sound(
                                            ts.game_state.current_piece.x,
                                        );
                                        tolk.output(
                                            t!(
                                                "in_game.move_left",
                                                col = ts
                                                    .game_state
                                                    .current_piece
                                                    .left_column()
                                                    .to_string()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::Right => {
                                if current_stage == 1 {
                                    if ts.game_state.move_right() {
                                        audio_engine.play_horizontal_move_sound(
                                            ts.game_state.current_piece.x,
                                        );
                                        if ts.game_state.current_piece.right_column() == 10 {
                                            ts.reached_right = true;
                                            audio_engine.play_aligned_sound();
                                            tolk.output(
                                                t!("tutorial.stage_1_right_reached").to_string(),
                                                true,
                                            );
                                        } else {
                                            tolk.output(
                                                t!(
                                                    "in_game.move_right",
                                                    col = (ts.game_state.current_piece.x + 1)
                                                        .to_string()
                                                )
                                                .to_string(),
                                                true,
                                            );
                                        }
                                        if ts.reached_left && ts.reached_right {
                                            ts.stage = 2;
                                            ts.init_stage_board();
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Tutorial { stage: 2 };
                                            audio_engine.play_menu_select();
                                            is_initial_load = true;
                                        }
                                    } else {
                                        audio_engine.play_aligned_sound();
                                    }
                                    screen_changed = true;
                                } else if current_stage == 5 {
                                    if ts.game_state.move_right() {
                                        audio_engine.play_horizontal_move_sound(
                                            ts.game_state.current_piece.x,
                                        );
                                        tolk.output(
                                            t!(
                                                "in_game.move_right",
                                                col = ts
                                                    .game_state
                                                    .current_piece
                                                    .left_column()
                                                    .to_string()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::Down => {
                                if current_stage == 2 {
                                    if ts.game_state.soft_drop() {
                                        audio_engine
                                            .play_soft_drop_sound(ts.game_state.current_piece.y);
                                        ts.soft_drops += 1;
                                        tolk.output(
                                            t!(
                                                "in_game.soft_drop",
                                                row =
                                                    (ts.game_state.current_piece.y + 1).to_string()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::HardDrop => {
                                if current_stage == 2 {
                                    audio_engine.play_hard_drop_sound();
                                    ts.game_state.hard_drop();
                                    ts.stage = 3;
                                    ts.init_stage_board();
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 3 };
                                    audio_engine.play_menu_select();
                                    is_initial_load = true;
                                    screen_changed = true;
                                } else if current_stage == 3 {
                                    audio_engine.play_hard_drop_sound();
                                    ts.stage = 4;
                                    ts.init_stage_board();
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 4 };
                                    audio_engine.play_menu_select();
                                    is_initial_load = true;
                                    screen_changed = true;
                                } else if current_stage == 4 {
                                    audio_engine.play_hard_drop_sound();
                                    ts.stage = 5;
                                    ts.sub_step = 0;
                                    ts.init_stage_board();
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 5 };
                                    audio_engine.play_menu_select();
                                    is_initial_load = true;
                                    screen_changed = true;
                                } else if current_stage == 5 {
                                    if ts.sub_step == 0 {
                                        audio_engine.play_hard_drop_sound();
                                        audio_engine.play_clear_sound(1);
                                        ts.sub_step = 1;
                                        ts.init_stage_board();
                                        is_initial_load = true;
                                    } else {
                                        audio_engine.play_hard_drop_sound();
                                        audio_engine.play_clear_sound(4);
                                        ts.stage = 6;
                                        ts.init_stage_board();
                                        *screen_state.lock().unwrap() =
                                            AppScreen::Tutorial { stage: 6 };
                                        audio_engine.play_menu_select();
                                        is_initial_load = true;
                                    }
                                    screen_changed = true;
                                } else if current_stage == 6 {
                                    ts.stage = 7;
                                    ts.init_stage_board();
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 7 };
                                    audio_engine.play_menu_select();
                                    is_initial_load = true;
                                    screen_changed = true;
                                } else if current_stage == 7 {
                                    audio_engine.play_hard_drop_sound();
                                    audio_engine.play_clear_sound(2);
                                    ts.stage = 8;
                                    ts.init_stage_board();
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 8 };
                                    audio_engine.play_menu_select();
                                    is_initial_load = true;
                                    screen_changed = true;
                                }
                            }
                            InputAction::PieceInfo => {
                                if current_stage == 3 {
                                    ts.inspected = true;
                                    tolk.output(
                                        t!(
                                            "tutorial.stage_3_inspect_spoken",
                                            left = ts
                                                .game_state
                                                .current_piece
                                                .left_column()
                                                .to_string(),
                                            right = ts
                                                .game_state
                                                .current_piece
                                                .right_column()
                                                .to_string(),
                                            width = ts.game_state.current_piece.width().to_string()
                                        )
                                        .to_string(),
                                        true,
                                    );
                                    screen_changed = true;
                                }
                            }
                            InputAction::RotateRight => {
                                if current_stage == 3 {
                                    if ts.game_state.rotate_cw() {
                                        ts.rotated_cw = true;
                                        audio_engine
                                            .play_rotate_cw_sound(ts.game_state.current_piece.y);
                                        tolk.output(
                                            t!(
                                                "tutorial.stage_3_cw_spoken",
                                                left = ts
                                                    .game_state
                                                    .current_piece
                                                    .left_column()
                                                    .to_string(),
                                                right = ts
                                                    .game_state
                                                    .current_piece
                                                    .right_column()
                                                    .to_string()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::RotateLeft => {
                                if current_stage == 3 {
                                    if ts.game_state.rotate_ccw() {
                                        ts.rotated_ccw = true;
                                        audio_engine
                                            .play_rotate_ccw_sound(ts.game_state.current_piece.y);
                                        tolk.output(
                                            t!(
                                                "tutorial.stage_3_ccw_spoken",
                                                left = ts
                                                    .game_state
                                                    .current_piece
                                                    .left_column()
                                                    .to_string(),
                                                right = ts
                                                    .game_state
                                                    .current_piece
                                                    .right_column()
                                                    .to_string()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::Hold => {
                                if current_stage == 4 {
                                    if !ts.held_first {
                                        ts.held_first = true;
                                        audio_engine.play_hold_sound();
                                        ts.game_state.hold_piece = Some(TetrominoType::S);
                                        ts.game_state.current_piece =
                                            Tetromino::new(TetrominoType::I);
                                        tolk.output(
                                            t!("tutorial.stage_4_hold_1").to_string(),
                                            true,
                                        );
                                        audio_engine
                                            .play_spawn_sound(ts.game_state.current_piece.t_type);
                                    } else if !ts.swapped_back {
                                        ts.swapped_back = true;
                                        audio_engine.play_hold_swap_sound();
                                        ts.game_state.hold_piece = Some(TetrominoType::I);
                                        ts.game_state.current_piece =
                                            Tetromino::new(TetrominoType::S);
                                        tolk.output(
                                            t!("tutorial.stage_4_hold_2").to_string(),
                                            true,
                                        );
                                        audio_engine
                                            .play_spawn_sound(ts.game_state.current_piece.t_type);
                                    } else {
                                        ts.tried_denied = true;
                                        audio_engine.play_hold_denied_sound();
                                        tolk.output(
                                            t!("tutorial.stage_4_hold_3").to_string(),
                                            true,
                                        );
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::Radar => {
                                if current_stage == 6 {
                                    ts.radar_scanned = true;
                                    audio_engine.play_radar_sweep(ts.game_state.get_topography());
                                    tolk.output(
                                        t!("tutorial.stage_6_scan_complete").to_string(),
                                        true,
                                    );
                                    screen_changed = true;
                                }
                            }
                            InputAction::Zone => {
                                if current_stage == 7 {
                                    ts.zone_entered = true;
                                    audio_engine.play_zone_enter();
                                    tolk.output(t!("tutorial.stage_7_active").to_string(), true);
                                    screen_changed = true;
                                }
                            }
                            InputAction::UseItem => {
                                if current_stage == 8 {
                                    if ts.item_step == 0 {
                                        audio_engine.play_item_use(ItemType::Magnet);
                                        ts.item_step = 1;
                                        audio_engine.play_item_acquire();
                                        tolk.output(
                                            t!("tutorial.stage_8_step_0_result").to_string(),
                                            true,
                                        );
                                    } else if ts.item_step == 1 {
                                        audio_engine.play_item_use(ItemType::Laser);
                                        ts.item_step = 2;
                                        audio_engine.play_item_acquire();
                                        tolk.output(
                                            t!("tutorial.stage_8_step_1_result").to_string(),
                                            true,
                                        );
                                    } else {
                                        audio_engine.play_item_use(ItemType::Nuke);
                                        ts.stage = 9;
                                        *screen_state.lock().unwrap() =
                                            AppScreen::Tutorial { stage: 9 };
                                        audio_engine.play_menu_select();
                                        is_initial_load = true;
                                    }
                                    screen_changed = true;
                                }
                            }
                            InputAction::Select => {
                                if current_stage == 6 {
                                    ts.stage = 7;
                                    ts.init_stage_board();
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 7 };
                                    audio_engine.play_menu_select();
                                    is_initial_load = true;
                                    screen_changed = true;
                                } else if current_stage == 9 {
                                    audio_engine.play_menu_select();
                                    {
                                        let mut s = settings.lock().unwrap();
                                        s.tutorial_completed = true;
                                        s.save();
                                    }
                                    let spoken = main_menu::render_main_menu(0, false).1;
                                    tolk.output(&spoken, true);
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                    screen_changed = true;
                                }
                            }
                            _ => {}
                        }
                    }
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
                                            tolk.output(t!("pause_menu.resumed").to_string(), true);
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
                                                AppScreen::ConfirmDialog {
                                                    action: ConfirmAction::StartTutorial,
                                                };
                                        }
                                        3 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::SaveScreen { selection: 0 };
                                        }
                                        4 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::LoadScreen { selection: 0 };
                                        }
                                        5 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Leaderboard { selection: 0 };
                                        }
                                        6 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::HowToPlay { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        7 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Settings { selection: 0 };
                                        }
                                        8 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::About { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        9 => {
                                            *screen_state.lock().unwrap() = AppScreen::Update {
                                                selection: 0,
                                                status: UpdateStatus::Idle,
                                            };
                                        }
                                        10 => {
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
                                            tolk.output(
                                                t!("in_game.new_game_started").to_string(),
                                                true,
                                            );
                                            let callout_tech =
                                                settings.lock().unwrap().piece_callouts_technical;
                                            let p_name = gs
                                                .current_piece
                                                .t_type
                                                .localized_name(callout_tech);
                                            tolk.output(
                                                t!("in_game.piece_spawned", piece = &p_name)
                                                    .to_string(),
                                                false,
                                            );
                                            audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                            if let Some(spawned) = gs.item_spawned {
                                                audio_engine.play_item_spawn();
                                                tolk.output(
                                                    t!(
                                                        "in_game.item_spawned",
                                                        item = &spawned.localized_name()
                                                    )
                                                    .to_string(),
                                                    false,
                                                );
                                            }
                                            *game_in_progress.lock().unwrap() = true;
                                            *screen_state.lock().unwrap() = AppScreen::InGame;
                                        }
                                        1 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Tutorial { stage: 1 };
                                            *tutorial_state.lock().unwrap() =
                                                tutorial_screen::TutorialState::new(1);
                                            is_initial_load = true;
                                        }
                                        2 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::LoadScreen { selection: 0 };
                                        }
                                        3 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Leaderboard { selection: 0 };
                                        }
                                        4 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::HowToPlay { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        5 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::Settings { selection: 0 };
                                        }
                                        6 => {
                                            *screen_state.lock().unwrap() =
                                                AppScreen::About { scroll_line: 0 };
                                            is_initial_load = true;
                                        }
                                        7 => {
                                            *screen_state.lock().unwrap() = AppScreen::Update {
                                                selection: 0,
                                                status: UpdateStatus::Idle,
                                            };
                                        }
                                        8 => {
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
                                tolk.output(t!("key_describer.intro_spoken").to_string(), true);
                                screen_changed = true;
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                if in_prog {
                                    tolk.output(t!("pause_menu.resumed").to_string(), true);
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
                                        tolk.output(t!("pause_menu.resumed").to_string(), true);
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
                                tolk.output(t!("pause_menu.resumed").to_string(), true);
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
                                    tolk.output(
                                        t!("save_load.save_fail", err = &e.to_string()).to_string(),
                                        true,
                                    );
                                } else {
                                    tolk.output(
                                        t!("save_load.save_success", slot = slot_id.to_string())
                                            .to_string(),
                                        true,
                                    );
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
                                        let callout_tech =
                                            settings.lock().unwrap().piece_callouts_technical;
                                        let piece_name =
                                            gs.current_piece.t_type.localized_name(callout_tech);
                                        let mut msg = t!(
                                            "save_load.load_success",
                                            slot = slot_id.to_string(),
                                            piece = &piece_name
                                        )
                                        .to_string();
                                        if let Some(item) = gs.current_piece.item {
                                            msg.push_str(&t!(
                                                "save_load.load_item_suffix",
                                                item = &item.localized_name()
                                            ));
                                        }
                                        if let Some(inv) = gs.inventory {
                                            msg.push_str(&t!(
                                                "save_load.load_inv_suffix",
                                                item = &inv.localized_name()
                                            ));
                                        }
                                        tolk.output(msg, true);
                                        audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                        *screen_state.lock().unwrap() = AppScreen::InGame;
                                    }
                                    Err(_) => {
                                        audio_engine.play_hold_denied_sound();
                                        tolk.output(
                                            t!("save_load.load_empty", slot = slot_id.to_string())
                                                .to_string(),
                                            true,
                                        );
                                    }
                                }
                            } else {
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 2 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 2 };
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
                                    AppScreen::MainMenu { selection: 2 };
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
                                            AppScreen::MainMenu { selection: 5 };
                                    } else {
                                        *screen_state.lock().unwrap() =
                                            AppScreen::MainMenu { selection: 3 };
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
                                        AppScreen::MainMenu { selection: 5 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 3 };
                                }
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::Settings { selection } => match action {
                        InputAction::Up => {
                            let new_sel = if selection > 0 { selection - 1 } else { 9 };
                            *screen_state.lock().unwrap() =
                                AppScreen::Settings { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Down => {
                            let new_sel = if selection < 9 { selection + 1 } else { 0 };
                            *screen_state.lock().unwrap() =
                                AppScreen::Settings { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Left => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.language = s.language.prev();
                                rust_i18n::set_locale(s.language.code());
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!("settings.language", name = s.language.display_name())
                                        .to_string(),
                                    true,
                                );
                            } else if selection == 1 {
                                s.difficulty = match s.difficulty {
                                    Difficulty::Easy => Difficulty::Difficult,
                                    Difficulty::Moderate => Difficulty::Easy,
                                    Difficulty::Difficult => Difficulty::Moderate,
                                };
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!(
                                        "settings.difficulty_spoken",
                                        value = s.difficulty.localized_str()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 4 {
                                s.voice_volume = (s.voice_volume - 0.05).max(0.0);
                                tolk.speak(
                                    t!(
                                        "settings.voice_volume_spoken",
                                        value = ((s.voice_volume * 100.0) as i32).to_string()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 5 {
                                s.sfx_volume = (s.sfx_volume - 0.05).max(0.0);
                                audio_engine.set_sfx_volume(s.sfx_volume);
                                audio_engine.play_aligned_sound();
                            } else if selection == 6 {
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
                                let on_str = t!("common.on");
                                let off_str = t!("common.off");
                                let status = if s.bgm_enabled { &on_str } else { &off_str };
                                tolk.speak(
                                    t!("settings.bgm_status_spoken", status = status).to_string(),
                                    true,
                                );
                            } else if selection == 7 {
                                s.bgm_volume = (s.bgm_volume - 0.05).max(0.0);
                                audio_engine.set_bgm_volume(s.bgm_volume);
                                if !s.bgm_enabled && s.bgm_volume > 0.0 {
                                    s.bgm_enabled = true;
                                    audio_engine.set_bgm_enabled(true);
                                }
                                tolk.speak(
                                    t!(
                                        "settings.bgm_volume_spoken",
                                        value = ((s.bgm_volume * 100.0) as i32).to_string()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 8 {
                                s.check_for_updates = !s.check_for_updates;
                                let on_str = t!("common.on");
                                let off_str = t!("common.off");
                                let status = if s.check_for_updates {
                                    &on_str
                                } else {
                                    &off_str
                                };
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!("settings.auto_update_spoken", status = status).to_string(),
                                    true,
                                );
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Right => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.language = s.language.next();
                                rust_i18n::set_locale(s.language.code());
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!("settings.language", name = s.language.display_name())
                                        .to_string(),
                                    true,
                                );
                            } else if selection == 1 {
                                s.difficulty = match s.difficulty {
                                    Difficulty::Easy => Difficulty::Moderate,
                                    Difficulty::Moderate => Difficulty::Difficult,
                                    Difficulty::Difficult => Difficulty::Easy,
                                };
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!(
                                        "settings.difficulty_spoken",
                                        value = s.difficulty.localized_str()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 4 {
                                s.voice_volume = (s.voice_volume + 0.05).min(1.0);
                                tolk.speak(
                                    t!(
                                        "settings.voice_volume_spoken",
                                        value = ((s.voice_volume * 100.0) as i32).to_string()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 5 {
                                s.sfx_volume = (s.sfx_volume + 0.05).min(1.0);
                                audio_engine.set_sfx_volume(s.sfx_volume);
                                audio_engine.play_aligned_sound();
                            } else if selection == 6 {
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
                                let on_str = t!("common.on");
                                let off_str = t!("common.off");
                                let status = if s.bgm_enabled { &on_str } else { &off_str };
                                tolk.speak(
                                    t!("settings.bgm_status_spoken", status = status).to_string(),
                                    true,
                                );
                            } else if selection == 7 {
                                s.bgm_volume = (s.bgm_volume + 0.05).min(1.0);
                                audio_engine.set_bgm_volume(s.bgm_volume);
                                if !s.bgm_enabled && s.bgm_volume > 0.0 {
                                    s.bgm_enabled = true;
                                    audio_engine.set_bgm_enabled(true);
                                }
                                tolk.speak(
                                    t!(
                                        "settings.bgm_volume_spoken",
                                        value = ((s.bgm_volume * 100.0) as i32).to_string()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 8 {
                                s.check_for_updates = !s.check_for_updates;
                                let on_str = t!("common.on");
                                let off_str = t!("common.off");
                                let status = if s.check_for_updates {
                                    &on_str
                                } else {
                                    &off_str
                                };
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!("settings.auto_update_spoken", status = status).to_string(),
                                    true,
                                );
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Select | InputAction::Back => {
                            if selection == 2 && action == InputAction::Select {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::VisualSettings { selection: 0 };
                                screen_changed = true;
                            } else if selection == 3 && action == InputAction::Select {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::SpeechVerbosity { selection: 0 };
                                screen_changed = true;
                            } else if selection == 9 || action == InputAction::Back {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 4 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 5 };
                                }
                                screen_changed = true;
                            }
                        }
                        _ => {}
                    },
                    AppScreen::VisualSettings { selection } => match action {
                        InputAction::Up => {
                            let new_sel = if selection > 0 { selection - 1 } else { 2 };
                            *screen_state.lock().unwrap() =
                                AppScreen::VisualSettings { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Down => {
                            let new_sel = if selection < 2 { selection + 1 } else { 0 };
                            *screen_state.lock().unwrap() =
                                AppScreen::VisualSettings { selection: new_sel };
                            audio_engine.play_menu_move();
                            screen_changed = true;
                        }
                        InputAction::Left => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.window_size = s.window_size.prev();
                                Self::apply_visual_settings(&frame, &panel, &text_ctrl, &s);
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!(
                                        "settings.window_size_spoken",
                                        value = s.window_size.localized_str()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 1 {
                                s.font_scale = s.font_scale.prev();
                                Self::apply_visual_settings(&frame, &panel, &text_ctrl, &s);
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!(
                                        "settings.font_scale_spoken",
                                        value = s.font_scale.localized_str()
                                    )
                                    .to_string(),
                                    true,
                                );
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Right | InputAction::Select => {
                            let mut s = settings.lock().unwrap();
                            if selection == 0 {
                                s.window_size = s.window_size.next();
                                Self::apply_visual_settings(&frame, &panel, &text_ctrl, &s);
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!(
                                        "settings.window_size_spoken",
                                        value = s.window_size.localized_str()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 1 {
                                s.font_scale = s.font_scale.next();
                                Self::apply_visual_settings(&frame, &panel, &text_ctrl, &s);
                                audio_engine.play_menu_move();
                                tolk.speak(
                                    t!(
                                        "settings.font_scale_spoken",
                                        value = s.font_scale.localized_str()
                                    )
                                    .to_string(),
                                    true,
                                );
                            } else if selection == 2 && action == InputAction::Select {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::Settings { selection: 2 };
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            *screen_state.lock().unwrap() = AppScreen::Settings { selection: 2 };
                            screen_changed = true;
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
                                let terse_str = t!("settings.piece_callouts_terse");
                                let desc_str = t!("settings.piece_callouts_descriptive");
                                let status = if s.piece_callouts_technical {
                                    &terse_str
                                } else {
                                    &desc_str
                                };
                                tolk.speak(
                                    t!("settings.piece_callouts_spoken", status = status)
                                        .to_string(),
                                    true,
                                );
                            } else if selection == 1 {
                                s.scoring_details_advanced = !s.scoring_details_advanced;
                                let adv_str = t!("settings.scoring_details_advanced");
                                let sim_str = t!("settings.scoring_details_simple");
                                let status = if s.scoring_details_advanced {
                                    &adv_str
                                } else {
                                    &sim_str
                                };
                                tolk.speak(
                                    t!("settings.scoring_details_spoken", status = status)
                                        .to_string(),
                                    true,
                                );
                            } else if selection == 2 {
                                s.zone_alerts = !s.zone_alerts;
                                let on_str = t!("common.on");
                                let off_str = t!("common.off");
                                let status = if s.zone_alerts { &on_str } else { &off_str };
                                tolk.speak(
                                    t!("settings.zone_alerts_spoken", status = status).to_string(),
                                    true,
                                );
                            } else if selection == 3 && action == InputAction::Select {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() =
                                    AppScreen::Settings { selection: 3 };
                            }
                            s.save();
                            screen_changed = true;
                        }
                        InputAction::Back => {
                            audio_engine.play_menu_select();
                            *screen_state.lock().unwrap() = AppScreen::Settings { selection: 3 };
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
                                        AppScreen::MainMenu { selection: 4 };
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
                                        AppScreen::MainMenu { selection: 8 };
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 6 };
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
                                            let start_time = std::time::Instant::now();
                                            let cur_ver = env!("APP_VERSION");
                                            let last_check = settings_bg
                                                .lock()
                                                .unwrap()
                                                .last_update_check_timestamp;
                                            let (new_status, now) = updater::check_latest_release(
                                                true, cur_ver, last_check,
                                            );

                                            let elapsed = start_time.elapsed();
                                            if elapsed < std::time::Duration::from_millis(2200) {
                                                std::thread::sleep(
                                                    std::time::Duration::from_millis(2200)
                                                        - elapsed,
                                                );
                                            }

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
                                                        t!(
                                                            "updater.spoken_update_available_status",
                                                            version = &info.version
                                                        )
                                                        .to_string(),
                                                        true,
                                                    );
                                                }
                                                UpdateStatus::UpToDate => {
                                                    tolk_bg.speak(
                                                        t!("updater.spoken_up_to_date").to_string(),
                                                        true,
                                                    );
                                                }
                                                UpdateStatus::Error(e) => {
                                                    tolk_bg.speak(
                                                        t!("updater.spoken_error", err = &e)
                                                            .to_string(),
                                                        true,
                                                    );
                                                }
                                                _ => {}
                                            }
                                        });
                                    }
                                    (UpdateStatus::Available(_), 2) | (_, 1) => {
                                        let in_prog = *game_in_progress.lock().unwrap();
                                        let back_sel = if in_prog { 9 } else { 7 };
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
                                let back_sel = if in_prog { 9 } else { 7 };
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
                                    tolk.output(t!("in_game.new_game_started").to_string(), true);
                                    let callout_tech =
                                        settings.lock().unwrap().piece_callouts_technical;
                                    let p_name =
                                        gs.current_piece.t_type.localized_name(callout_tech);
                                    tolk.output(
                                        t!("in_game.piece_spawned", piece = &p_name).to_string(),
                                        false,
                                    );
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                    if let Some(spawned) = gs.item_spawned {
                                        audio_engine.play_item_spawn();
                                        tolk.output(
                                            t!(
                                                "in_game.item_spawned",
                                                item = &spawned.localized_name()
                                            )
                                            .to_string(),
                                            false,
                                        );
                                    }
                                    *game_in_progress.lock().unwrap() = true;
                                    *screen_state.lock().unwrap() = AppScreen::InGame;
                                }
                                ConfirmAction::StartTutorial => {
                                    *game_in_progress.lock().unwrap() = false;
                                    *screen_state.lock().unwrap() =
                                        AppScreen::Tutorial { stage: 1 };
                                    *tutorial_state.lock().unwrap() =
                                        tutorial_screen::TutorialState::new(1);
                                    is_initial_load = true;
                                }
                                ConfirmAction::AbandonGame => {
                                    *game_in_progress.lock().unwrap() = false;
                                    tolk.output(
                                        t!("confirm_dialog.game_abandoned").to_string(),
                                        true,
                                    );
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                }
                                ConfirmAction::QuitApp => {
                                    frame.close(true);
                                    return;
                                }
                                ConfirmAction::UpdateApp(ref info) => {
                                    tolk.output(t!("updater.spoken_downloading").to_string(), true);
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
                                            tolk_bg.speak(
                                                t!(
                                                    "updater.spoken_update_failed",
                                                    err = &e.to_string()
                                                )
                                                .to_string(),
                                                true,
                                            );
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
                                *screen_state.lock().unwrap() = match confirm_act {
                                    ConfirmAction::AbandonGame => {
                                        AppScreen::PauseMenu { selection: 2 }
                                    }
                                    ConfirmAction::NewGame => AppScreen::MainMenu { selection: 1 },
                                    ConfirmAction::StartTutorial => {
                                        AppScreen::MainMenu { selection: 2 }
                                    }
                                    ConfirmAction::QuitApp => AppScreen::MainMenu { selection: 10 },
                                    _ => AppScreen::PauseMenu { selection: 0 },
                                };
                            } else {
                                *screen_state.lock().unwrap() = match confirm_act {
                                    ConfirmAction::QuitApp => AppScreen::MainMenu { selection: 8 },
                                    _ => AppScreen::MainMenu { selection: 0 },
                                };
                            }
                            screen_changed = true;
                        }
                        _ => {}
                    },
                    AppScreen::InGame => {
                        let mut gs = game_state.lock().unwrap();

                        match action {
                            InputAction::Back => {
                                tolk.output(t!("pause_menu.paused").to_string(), true);
                                *screen_state.lock().unwrap() =
                                    AppScreen::PauseMenu { selection: 0 };
                                screen_changed = true;
                            }
                            InputAction::Radar => {
                                let max_h = gs.max_column_height();
                                audio_engine.play_radar_sweep(gs.get_topography());
                                tolk.output(
                                    t!("in_game.radar_sweep", max_height = max_h.to_string())
                                        .to_string(),
                                    true,
                                );
                            }
                            InputAction::Left => {
                                if gs.move_left() {
                                    audio_engine
                                        .play_horizontal_move_sound(gs.current_piece.left_column());
                                    if gs.current_piece.left_column() == 1 {
                                        audio_engine.play_aligned_sound();
                                    }
                                    tolk.output(
                                        t!(
                                            "in_game.move_left",
                                            col = gs.current_piece.left_column().to_string()
                                        )
                                        .to_string(),
                                        true,
                                    );
                                } else {
                                    audio_engine.play_aligned_sound();
                                }
                            }
                            InputAction::Right => {
                                if gs.move_right() {
                                    audio_engine.play_horizontal_move_sound(
                                        gs.current_piece.right_column(),
                                    );
                                    if gs.current_piece.right_column() == 10 {
                                        audio_engine.play_aligned_sound();
                                    }
                                    tolk.output(
                                        t!(
                                            "in_game.move_right",
                                            col = gs.current_piece.left_column().to_string()
                                        )
                                        .to_string(),
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
                                        0 => t!("in_game.deg_0"),
                                        1 => t!("in_game.deg_90"),
                                        2 => t!("in_game.deg_180"),
                                        3 => t!("in_game.deg_270"),
                                        _ => std::borrow::Cow::Borrowed(""),
                                    };
                                    let left = gs.current_piece.left_column();
                                    let right = gs.current_piece.right_column();
                                    tolk.output(
                                        t!(
                                            "in_game.rotated_right",
                                            deg = &rot_deg,
                                            left = left.to_string(),
                                            right = right.to_string()
                                        )
                                        .to_string(),
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
                                        0 => t!("in_game.deg_0"),
                                        1 => t!("in_game.deg_90"),
                                        2 => t!("in_game.deg_180"),
                                        3 => t!("in_game.deg_270"),
                                        _ => std::borrow::Cow::Borrowed(""),
                                    };
                                    let left = gs.current_piece.left_column();
                                    let right = gs.current_piece.right_column();
                                    tolk.output(
                                        t!(
                                            "in_game.rotated_left",
                                            deg = &rot_deg,
                                            left = left.to_string(),
                                            right = right.to_string()
                                        )
                                        .to_string(),
                                        true,
                                    );
                                } else {
                                    audio_engine.play_aligned_sound();
                                }
                            }
                            InputAction::PieceInfo => {
                                let callout_tech =
                                    settings.lock().unwrap().piece_callouts_technical;
                                let name = gs.current_piece.t_type.localized_name(callout_tech);
                                let left = gs.current_piece.left_column();
                                let right = gs.current_piece.right_column();
                                let width = gs.current_piece.width();
                                let held_str = if let Some(held) = gs.hold_piece {
                                    held.localized_name(callout_tech)
                                } else {
                                    t!("in_game.piece_none").to_string()
                                };
                                let text = t!(
                                    "in_game.current_piece_info",
                                    piece = &name,
                                    left = left.to_string(),
                                    right = right.to_string(),
                                    width = width.to_string(),
                                    held = &held_str
                                )
                                .to_string();
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
                                        t!(
                                            "in_game.held_piece",
                                            prev = &prev.localized_name(callout_tech),
                                            new = &new_p.localized_name(callout_tech)
                                        )
                                        .to_string(),
                                        true,
                                    );
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.output(t!("in_game.already_held").to_string(), true);
                                }
                            }
                            InputAction::Select => {}
                            InputAction::Down => {
                                if gs.soft_drop() {
                                    audio_engine.play_soft_drop_sound(gs.current_piece.y);
                                    tolk.output(
                                        t!(
                                            "in_game.soft_drop",
                                            row = (gs.current_piece.y + 1).to_string()
                                        )
                                        .to_string(),
                                        true,
                                    );
                                    if !gs.can_fall() && !gs.lock_delay_active {
                                        gs.lock_delay_active = true;
                                        gs.lock_delay_timer_ms = 500;
                                        gs.moves_since_lock_delay = 0;
                                    }
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
                                    tolk.output(
                                        t!(
                                            "in_game.zone_cleared_lines",
                                            cleared = res.zone_lines_cleared_this_turn.to_string(),
                                            total = gs.zone_lines_cleared.to_string()
                                        )
                                        .to_string(),
                                        true,
                                    );
                                } else if res.cleared_lines > 0 {
                                    audio_engine.play_clear_sound(res.cleared_lines);
                                    let mut tts = t!(
                                        "in_game.hard_drop_cleared",
                                        cleared = res.cleared_lines.to_string()
                                    )
                                    .to_string();
                                    if res.is_t_spin {
                                        audio_engine.play_t_spin_sound();
                                        if scoring_advanced {
                                            tts.push_str(&format!(" {}", t!("in_game.t_spin")));
                                        }
                                    }
                                    if res.b2b_bonus {
                                        audio_engine.play_b2b_sound();
                                        if scoring_advanced {
                                            tts.push_str(&format!(" {}", t!("in_game.b2b")));
                                        }
                                    }
                                    if res.combo > 1 && scoring_advanced {
                                        tts.push_str(&format!(
                                            " {}",
                                            t!(
                                                "in_game.combo_bonus",
                                                combo = res.combo.to_string()
                                            )
                                        ));
                                    }
                                    tts.push_str(&format!(
                                        " {}",
                                        t!(
                                            "in_game.level_score_suffix",
                                            level = gs.level.to_string(),
                                            score = gs.score.to_string()
                                        )
                                    ));
                                    tolk.output(tts, true);
                                } else {
                                    if res.is_t_spin && scoring_advanced {
                                        audio_engine.play_t_spin_sound();
                                        tolk.output(
                                            t!(
                                                "in_game.t_spin_score",
                                                score = gs.score.to_string()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    tolk.output(
                                        t!("in_game.hard_drop_score", score = gs.score.to_string())
                                            .to_string(),
                                        true,
                                    );
                                }

                                if res.zone_meter_full && zone_alerts {
                                    audio_engine.play_zone_enter();
                                    tolk.output(t!("in_game.zone_meter_full").to_string(), false);
                                }

                                if gs.is_game_over {
                                    let _ = db.record_high_score(&gs);
                                    *game_in_progress.lock().unwrap() = false;
                                    tolk.output(
                                        t!("in_game.game_over", score = gs.score.to_string())
                                            .to_string(),
                                        true,
                                    );
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                } else {
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                    let callout_tech =
                                        settings.lock().unwrap().piece_callouts_technical;
                                    tolk.output(
                                        gs.current_piece.t_type.localized_name(callout_tech),
                                        false,
                                    );
                                    if let Some(acquired) = gs.item_acquired {
                                        audio_engine.play_item_acquire();
                                        tolk.output(
                                            t!(
                                                "in_game.item_acquired",
                                                item = &acquired.localized_name()
                                            )
                                            .to_string(),
                                            true,
                                        );
                                    }
                                    if let Some(spawned) = gs.item_spawned {
                                        audio_engine.play_item_spawn();
                                        tolk.output(
                                            t!(
                                                "in_game.item_spawned",
                                                item = &spawned.localized_name()
                                            )
                                            .to_string(),
                                            false,
                                        );
                                    }
                                }
                                screen_changed = true;
                            }
                            InputAction::Zone => {
                                if gs.start_zone() {
                                    audio_engine.play_zone_enter();
                                    tolk.output(t!("in_game.zone_activated").to_string(), true);
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.output(
                                        t!("in_game.zone_not_enough_charge").to_string(),
                                        true,
                                    );
                                }
                            }
                            InputAction::UseItem => {
                                if let Some(res) = gs.use_item() {
                                    audio_engine.play_item_use(res.item);
                                    if res.item == ItemType::Magnet && res.lines_cleared > 0 {
                                        audio_engine.play_clear_sound(res.lines_cleared);
                                        let mut msg = t!(
                                            "in_game.used_magnet_cleared",
                                            cleared = res.lines_cleared.to_string(),
                                            score = gs.score.to_string()
                                        )
                                        .to_string();
                                        if let Some(acquired) = gs.item_acquired {
                                            audio_engine.play_item_acquire();
                                            msg.push_str(&format!(
                                                " {}",
                                                t!(
                                                    "in_game.item_acquired",
                                                    item = &acquired.localized_name()
                                                )
                                            ));
                                        }
                                        tolk.output(msg, true);
                                    } else {
                                        let mut msg = t!(
                                            "in_game.used_item",
                                            item = &res.item.localized_name()
                                        )
                                        .to_string();
                                        if let Some(acquired) = gs.item_acquired {
                                            audio_engine.play_item_acquire();
                                            msg.push_str(&format!(
                                                " {}",
                                                t!(
                                                    "in_game.item_acquired",
                                                    item = &acquired.localized_name()
                                                )
                                            ));
                                        }
                                        tolk.output(msg, true);
                                    }
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.output(t!("in_game.no_item_to_use").to_string(), true);
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
                                tolk.output(t!("key_describer.press_esc_again").to_string(), true);
                            } else {
                                audio_engine.play_menu_select();
                                let in_prog = *game_in_progress.lock().unwrap();
                                if in_prog {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::PauseMenu { selection: 0 };
                                    tolk.output(
                                        t!("key_describer.exited_help_mode").to_string(),
                                        true,
                                    );
                                } else {
                                    *screen_state.lock().unwrap() =
                                        AppScreen::MainMenu { selection: 0 };
                                    tolk.output(
                                        t!("key_describer.exited_help_mode").to_string(),
                                        true,
                                    );
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

                if screen_changed || was_in_game {
                    *last_rendered.borrow_mut() = screen_state.lock().unwrap().clone();
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
            let panel_for_timer = panel;

            move |_| {
                let interval = 16;

                // --- GAME DROP TIMER ---
                if *screen.lock().unwrap() != AppScreen::InGame {
                    panel_for_timer.refresh(true, None);
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
                                t!(
                                    "in_game.zone_ended_cleared",
                                    cleared = lines.to_string(),
                                    score = gs.score.to_string()
                                )
                                .to_string(),
                                true,
                            );
                        } else {
                            tolk.output(t!("in_game.zone_ended").to_string(), true);
                        }
                    }
                }

                if gs.lock_delay_active {
                    let old_timer = gs.lock_delay_timer_ms;
                    gs.lock_delay_timer_ms -= interval;
                    if old_timer > 200 && gs.lock_delay_timer_ms <= 200 {
                        audio_engine.play_lock_delay_warning();
                    }
                    if gs.lock_delay_timer_ms <= 0 {
                        let res = gs.lock_piece();
                        let settings_lock = settings.lock().unwrap();
                        let scoring_advanced = settings_lock.scoring_details_advanced;
                        let zone_alerts = settings_lock.zone_alerts;
                        drop(settings_lock);

                        if res.zone_lines_cleared_this_turn > 0 {
                            audio_engine.play_clear_sound(res.zone_lines_cleared_this_turn);
                            tolk.output(
                                t!(
                                    "in_game.zone_cleared_lines",
                                    cleared = res.zone_lines_cleared_this_turn.to_string(),
                                    total = gs.zone_lines_cleared.to_string()
                                )
                                .to_string(),
                                true,
                            );
                        }

                        if res.zone_meter_full && zone_alerts {
                            audio_engine.play_zone_enter();
                            tolk.output(t!("in_game.zone_meter_full").to_string(), true);
                        }

                        if res.cleared_lines > 0 {
                            audio_engine.play_clear_sound(res.cleared_lines);
                            let mut tts = t!(
                                "in_game.cleared_lines_count",
                                count = res.cleared_lines.to_string()
                            )
                            .to_string();
                            if res.is_t_spin {
                                audio_engine.play_t_spin_sound();
                                if scoring_advanced {
                                    tts.push_str(&format!(" {}", t!("in_game.t_spin")));
                                }
                            }
                            if res.b2b_bonus {
                                audio_engine.play_b2b_sound();
                                if scoring_advanced {
                                    tts.push_str(&format!(" {}", t!("in_game.b2b")));
                                }
                            }
                            if res.combo > 1 && scoring_advanced {
                                tts.push_str(&format!(
                                    " {}",
                                    t!("in_game.combo_bonus", combo = res.combo.to_string())
                                ));
                            }
                            tts.push_str(&format!(
                                " {}",
                                t!(
                                    "in_game.level_score_suffix",
                                    level = gs.level.to_string(),
                                    score = gs.score.to_string()
                                )
                            ));
                            tolk.output(tts, true);
                        } else {
                            if res.is_t_spin && scoring_advanced {
                                audio_engine.play_t_spin_sound();
                                tolk.output(
                                    t!("in_game.t_spin_score", score = gs.score.to_string())
                                        .to_string(),
                                    true,
                                );
                            }
                            audio_engine.play_lock_sound();
                        }

                        if gs.is_game_over {
                            let _ = db.record_high_score(&gs);
                            *in_prog.lock().unwrap() = false;
                            tolk.output(
                                t!("in_game.game_over", score = gs.score.to_string()).to_string(),
                                true,
                            );
                            *screen.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                        } else {
                            audio_engine.play_spawn_sound(gs.current_piece.t_type);
                            let callout_tech = settings.lock().unwrap().piece_callouts_technical;
                            tolk.output(
                                gs.current_piece.t_type.localized_name(callout_tech),
                                false,
                            );
                            if let Some(acquired) = gs.item_acquired {
                                audio_engine.play_item_acquire();
                                tolk.output(
                                    t!("in_game.item_acquired", item = &acquired.localized_name())
                                        .to_string(),
                                    true,
                                );
                            }
                            if let Some(spawned) = gs.item_spawned {
                                audio_engine.play_item_spawn();
                                tolk.output(
                                    t!("in_game.item_spawned", item = &spawned.localized_name())
                                        .to_string(),
                                    false,
                                );
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
                            gs.moves_since_lock_delay = 0;
                        }
                    }
                }

                audio_engine.update_danger_state(gs.max_column_height());
                panel_for_timer.refresh(true, None);
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

            let now = Instant::now();
            if key_code == SCREEN_READER_TOGGLE_KEY {
                let mut pending = screen_reader_toggle_pending.borrow_mut();
                let confirm_window = Duration::from_secs(SCREEN_READER_TOGGLE_CONFIRM_SECS);
                let confirmed = pending
                    .map(|started| now.duration_since(started) <= confirm_window)
                    .unwrap_or(false);

                if confirmed {
                    *pending = None;
                    let mut s = settings.lock().unwrap();
                    let new_enabled = !s.screen_reader_enabled;
                    s.screen_reader_enabled = new_enabled;
                    s.save();

                    if new_enabled {
                        tolk_instance.set_enabled(true);
                        tolk_instance.speak_forced(t!("settings.screen_reader_enabled"), true);
                    } else {
                        tolk_instance.speak_forced(t!("settings.screen_reader_disabled"), true);
                        tolk_instance.set_enabled(false);
                    }
                } else {
                    *pending = Some(now);
                    let enabled = settings
                        .lock()
                        .map(|s| s.screen_reader_enabled)
                        .unwrap_or(true);
                    if enabled {
                        tolk_instance.speak_forced(
                            t!(
                                "settings.screen_reader_disable_prompt",
                                seconds = SCREEN_READER_TOGGLE_CONFIRM_SECS.to_string()
                            ),
                            true,
                        );
                    } else {
                        tolk_instance.speak_forced(
                            t!(
                                "settings.screen_reader_enable_prompt",
                                seconds = SCREEN_READER_TOGGLE_CONFIRM_SECS.to_string()
                            ),
                            true,
                        );
                    }
                }
                return;
            }

            if let Some(started) = *screen_reader_toggle_pending.borrow() {
                let confirm_window = Duration::from_secs(SCREEN_READER_TOGGLE_CONFIRM_SECS);
                *screen_reader_toggle_pending.borrow_mut() = None;
                if now.duration_since(started) <= confirm_window {
                    let enabled = settings
                        .lock()
                        .map(|s| s.screen_reader_enabled)
                        .unwrap_or(true);
                    if enabled {
                        tolk_instance
                            .speak_forced(t!("settings.screen_reader_still_enabled"), true);
                    } else {
                        tolk_instance
                            .speak_forced(t!("settings.screen_reader_still_disabled"), true);
                    }
                }
            }

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
