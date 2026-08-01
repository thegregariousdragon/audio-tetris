use wxdragon::prelude::*;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use crate::logic::GameState;
use crate::audio::AudioEngine;
use crate::settings::{Settings, Difficulty};
use tolk::Tolk;
use gilrs::{Gilrs, Event, EventType, Button};

#[derive(Clone, Copy, PartialEq)]
pub enum AppScreen {
    MainMenu { selection: usize },
    Settings { selection: usize },
    HowToPlay { scroll_line: usize },
    About { scroll_line: usize },
    InGame,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputAction {
    Up, Down, Left, Right, Select, Back, HardDrop, Radar, RotateLeft, RotateRight, Start, NextTrack, PrevTrack, Mute, Hold
}

fn get_how_to_play_lines(controller_enabled: bool) -> Vec<String> {
    let mut lines = vec![
        "How to Play".to_string(),
        "".to_string(),
    ];
    if controller_enabled {
        lines.extend(vec![
            "Gamepad Controls:".to_string(),
            "Move: D-Pad Left and Right".to_string(),
            "Rotate Left: Left Bumper".to_string(),
            "Rotate Right: Right Bumper".to_string(),
            "Soft Drop: D-Pad Down".to_string(),
            "Hard Drop: Right Trigger".to_string(),
            "Hold Piece: D-Pad Up".to_string(),
            "Radar Sweep: Left Trigger".to_string(),
            "Quick Settings or Pause: Start Button".to_string(),
            "Change Music Track: Press Left Stick or Right Stick".to_string(),
            "".to_string(),
        ]);
    }
    lines.extend(vec![
        "Keyboard Playstyles:".to_string(),
        "Audio Tetris features mirrored clusters so you can play your way!".to_string(),
        "- Left-Handed One-Hand: Move with ASD, Rotate with ZXC, Music with QWE.".to_string(),
        "- Right-Handed One-Hand: Move with L/Semicolon/Apostrophe, Rotate with Comma/Period/Slash, Music with P/Brackets.".to_string(),
        "- Traditional Two-Handed: Move with your right hand (L/Semicolon/Apostrophe) and Rotate with your left hand (ZXC).".to_string(),
        "".to_string(),
        "Keyboard Controls:".to_string(),
        "Move: A and D keys, or L and Apostrophe keys".to_string(),
        "Rotate Left: Z key (or Comma key)".to_string(),
        "Rotate Right: X key (or Period key)".to_string(),
        "Soft Drop: S key (or Semicolon key)".to_string(),
        "Hard Drop: Spacebar".to_string(),
        "Hold Piece: C key (or Slash key)".to_string(),
        "Radar Sweep: R key".to_string(),
        "Quick Settings: Tab key".to_string(),
        "Pause or Back: Escape key".to_string(),
        "Music Controls: Q or P (Prev), W or Left Bracket (Mute), E or Right Bracket (Next)".to_string(),
        "".to_string(),
        "Audio Cues:".to_string(),
        "- High pitch = Top of board, Low pitch = Bottom".to_string(),
        "- Chime = Perfect alignment with a gap".to_string(),
        "- Thud = Piece locked".to_string(),
        "- Explosion = Line cleared".to_string(),
        "".to_string(),
        "Press Enter or A to read all. Press Escape to go back.".to_string(),
    ]);
    lines
}

fn get_about_lines() -> Vec<String> {
    vec![
        "About Audio Tetris".to_string(),
        format!("Version {}", env!("APP_VERSION")),
        "Copyright (c) 2026 Gregory Lopez. All rights reserved.".to_string(),
        "Built with Rust, wxDragon, GilRs, and Tolk.".to_string(),
        "".to_string(),
        "Press Enter or A to read all. Press Escape to go back.".to_string(),
    ]
}

pub struct AppFrame {
    frame: Frame,
    panel: Panel,
    text_display: StaticText,
    game_state: Arc<Mutex<GameState>>,
    audio_engine: Arc<AudioEngine>,
    timer: Rc<RefCell<wxdragon::timer::Timer<Frame>>>,
    gilrs_timer: Rc<RefCell<wxdragon::timer::Timer<Frame>>>,
    gilrs: Arc<Mutex<Gilrs>>,
    tolk: Arc<Tolk>,
    settings: Arc<Mutex<Settings>>,
    screen: Arc<Mutex<AppScreen>>,
    game_in_progress: Arc<Mutex<bool>>,
}

impl AppFrame {
    pub fn new() -> Self {
        let tolk = Tolk::new();
        tolk.try_sapi(true);
        
        let settings_data = Settings::load();
        let settings = Arc::new(Mutex::new(settings_data.clone()));
        let screen = Arc::new(Mutex::new(AppScreen::MainMenu { selection: 0 }));
        let game_in_progress = Arc::new(Mutex::new(false));
        
        let title = format!("Audio Tetris v{}", env!("APP_VERSION"));
        let frame = Frame::builder()
            .with_title(&title)
            .with_size(Size::new(600, 400))
            .build();
            
        let panel = Panel::builder(&frame)
            .with_style(PanelStyle::BorderNone)
            .build();
            
        let sizer = BoxSizer::builder(Orientation::Vertical).build();
        
        let text_display = StaticText::builder(&panel)
            .with_label("Loading...")
            .build();

        sizer.add(&text_display, 1, SizerFlag::All | SizerFlag::Expand, 20);
        
        panel.set_sizer(sizer, true);

        let diff = settings_data.difficulty;
        let game_state = Arc::new(Mutex::new(GameState::new(diff)));
        let audio_engine = Arc::new(AudioEngine::new(&settings_data).unwrap());
        let timer = Rc::new(RefCell::new(wxdragon::timer::Timer::new(&frame)));
        let gilrs_timer = Rc::new(RefCell::new(wxdragon::timer::Timer::new(&frame)));
        let gilrs = Arc::new(Mutex::new(Gilrs::new().unwrap()));

        Self {
            frame,
            panel,
            text_display,
            game_state,
            audio_engine,
            timer,
            gilrs_timer,
            gilrs,
            tolk,
            settings,
            screen,
            game_in_progress,
        }
    }

    pub fn render_screen(&self, speak: bool, initial_load: bool) {
        let screen = *self.screen.lock().unwrap();
        let s = self.settings.lock().unwrap();
        let in_prog = *self.game_in_progress.lock().unwrap();
        
        let (display_text, spoken_text) = match screen {
            AppScreen::MainMenu { selection } => {
                let opt0 = if in_prog { "Resume Game" } else { "New Game" };
                let options = [opt0, "How to Play", "Settings", "About", "Quit"];
                let mut text = String::from("Main Menu\n\n");
                for (i, opt) in options.iter().enumerate() {
                    if i == selection {
                        text.push_str(&format!("-> {}\n", opt));
                    } else {
                        text.push_str(&format!("   {}\n", opt));
                    }
                }
                (text, options[selection].to_string())
            }
            AppScreen::Settings { selection } => {
                let options = [
                    format!("Difficulty: {}", s.difficulty.as_str()),
                    format!("Sound Effects Volume: {}%", (s.sfx_volume * 100.0) as i32),
                    format!("Background Music Volume: {}%", (s.bgm_volume * 100.0) as i32),
                    format!("Gamepad Support: {}", if s.controller_enabled { "ON" } else { "OFF" }),
                    "Back".to_string(),
                ];
                let mut text = String::from("Settings\nUse Left and Right arrows to adjust values.\n\n");
                for (i, opt) in options.iter().enumerate() {
                    if i == selection {
                        text.push_str(&format!("-> {}\n", opt));
                    } else {
                        text.push_str(&format!("   {}\n", opt));
                    }
                }
                (text, options[selection].clone())
            }
            AppScreen::HowToPlay { scroll_line } => {
                let lines = get_how_to_play_lines(s.controller_enabled);
                let mut text = String::new();
                for (i, line) in lines.iter().enumerate() {
                    if i == scroll_line {
                        text.push_str(&format!("-> {}\n", line));
                    } else {
                        text.push_str(&format!("   {}\n", line));
                    }
                }
                
                let line_text = if scroll_line < lines.len() { lines[scroll_line].clone() } else { "".to_string() };
                let spoken = if initial_load {
                    format!("How to play. Use arrows to read line by line. Press Enter or A to read all. Press Escape to go back. {}", line_text)
                } else {
                    line_text
                };
                
                (text, spoken)
            }
            AppScreen::About { scroll_line } => {
                let lines = get_about_lines();
                let mut text = String::new();
                for (i, line) in lines.iter().enumerate() {
                    if i == scroll_line {
                        text.push_str(&format!("-> {}\n", line));
                    } else {
                        text.push_str(&format!("   {}\n", line));
                    }
                }
                
                let line_text = if scroll_line < lines.len() { lines[scroll_line].clone() } else { "".to_string() };
                let spoken = if initial_load {
                    format!("About Audio Tetris. Use arrows to read line by line. Press Enter or A to read all. Press Escape to go back. {}", line_text)
                } else {
                    line_text
                };
                
                (text, spoken)
            }
            AppScreen::InGame => {
                let gs = self.game_state.lock().unwrap();
                let text = format!("In Game\nLevel: {}\nScore: {}\nLines: {}\n\nPress Escape or Start to pause game.", gs.level, gs.score, gs.total_lines);
                (text, "Game started. Use arrow keys or D-Pad to play.".to_string())
            }
        };

        self.text_display.set_label(&display_text);
        
        if speak && !spoken_text.is_empty() {
            self.tolk.speak(spoken_text, true);
        }
    }

    pub fn setup_events(&mut self) {
        let game_state = self.game_state.clone();
        let audio_engine = self.audio_engine.clone();
        let timer_clone = self.timer.clone();
        let tolk_instance = self.tolk.clone();
        let screen_state = self.screen.clone();
        let game_in_progress = self.game_in_progress.clone();
        let settings = self.settings.clone();
        let text_display = self.text_display;
        let frame = self.frame;

        // 1. GAME TICK TIMER
        self.timer.borrow().on_tick({
            let game_state = game_state.clone();
            let audio_engine = audio_engine.clone();
            let timer = timer_clone.clone();
            let tolk = tolk_instance.clone();
            let screen = screen_state.clone();
            let in_prog = game_in_progress.clone();
            move |_| {
                if *screen.lock().unwrap() != AppScreen::InGame { return; }
                let mut gs = game_state.lock().unwrap();
                if gs.is_game_over { return; }
                
                if gs.move_piece(0, 1) {
                    audio_engine.play_move_sound(gs.current_piece.x, gs.current_piece.y);
                } else {
                    let lines = gs.lock_piece();
                    if lines > 0 {
                        audio_engine.play_clear_sound(lines);
                        tolk.output(format!("Cleared {} lines! Level: {}. Score: {}", lines, gs.level, gs.score), true);
                        timer.borrow().start(gs.current_speed_ms(), false);
                    } else {
                        audio_engine.play_lock_sound();
                    }
                    if gs.is_game_over {
                        timer.borrow().stop();
                        *in_prog.lock().unwrap() = false;
                        tolk.output(format!("Game Over! Final Score: {}", gs.score), true);
                        *screen.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                    } else {
                        audio_engine.play_spawn_sound(gs.current_piece.t_type);
                    }
                }
            }
        });

        // 2. SHARED INPUT LOGIC (used by both Keyboard and Gamepad)
        let shared_logic = {
            let game_state = game_state.clone();
            let audio_engine = audio_engine.clone();
            let timer = timer_clone.clone();
            let tolk = tolk_instance.clone();
            let settings = settings.clone();
            let screen_state = screen_state.clone();
            let text_display = text_display;
            let frame = frame;
            let game_in_progress = game_in_progress.clone();
            
            Rc::new(RefCell::new(move |action: InputAction| {
                let render_in_closure = |speak: bool, initial_load: bool| {
                    let current_screen = *screen_state.lock().unwrap();
                    let s = settings.lock().unwrap();
                    let in_prog = *game_in_progress.lock().unwrap();
                    
                    let (display_text, spoken_text) = match current_screen {
                        AppScreen::MainMenu { selection } => {
                            let opt0 = if in_prog { "Resume Game" } else { "New Game" };
                            let options = [opt0, "How to Play", "Settings", "About", "Quit"];
                            let mut text = String::from("Main Menu\n\n");
                            for (i, opt) in options.iter().enumerate() {
                                if i == selection { text.push_str(&format!("-> {}\n", opt)); } 
                                else { text.push_str(&format!("   {}\n", opt)); }
                            }
                            (text, options[selection].to_string())
                        }
                        AppScreen::Settings { selection } => {
                            let options = [
                                format!("Difficulty: {}", s.difficulty.as_str()),
                                format!("Sound Effects Volume: {}%", (s.sfx_volume * 100.0) as i32),
                                format!("Background Music Volume: {}%", (s.bgm_volume * 100.0) as i32),
                                format!("Gamepad Support: {}", if s.controller_enabled { "ON" } else { "OFF" }),
                                "Back".to_string(),
                            ];
                            let mut text = String::from("Settings\nUse Left and Right arrows to adjust values.\n\n");
                            for (i, opt) in options.iter().enumerate() {
                                if i == selection { text.push_str(&format!("-> {}\n", opt)); } 
                                else { text.push_str(&format!("   {}\n", opt)); }
                            }
                            (text, options[selection].clone())
                        }
                        AppScreen::HowToPlay { scroll_line } => {
                            let lines = get_how_to_play_lines(s.controller_enabled);
                            let mut text = String::new();
                            for (i, line) in lines.iter().enumerate() {
                                if i == scroll_line { text.push_str(&format!("-> {}\n", line)); } 
                                else { text.push_str(&format!("   {}\n", line)); }
                            }
                            let line_text = if scroll_line < lines.len() { lines[scroll_line].clone() } else { "".to_string() };
                            let spoken = if initial_load {
                                format!("How to play. Use arrows to read line by line. Press Enter or A to read all. Press Escape to go back. {}", line_text)
                            } else { line_text };
                            (text, spoken)
                        }
                        AppScreen::About { scroll_line } => {
                            let lines = get_about_lines();
                            let mut text = String::new();
                            for (i, line) in lines.iter().enumerate() {
                                if i == scroll_line { text.push_str(&format!("-> {}\n", line)); } 
                                else { text.push_str(&format!("   {}\n", line)); }
                            }
                            let line_text = if scroll_line < lines.len() { lines[scroll_line].clone() } else { "".to_string() };
                            let spoken = if initial_load {
                                format!("About Audio Tetris. Use arrows to read line by line. Press Enter or A to read all. Press Escape to go back. {}", line_text)
                            } else { line_text };
                            (text, spoken)
                        }
                        AppScreen::InGame => {
                            let gs = game_state.lock().unwrap();
                            let text = format!("In Game\nLevel: {}\nScore: {}\nLines: {}\n\nPress Escape to pause game.", gs.level, gs.score, gs.total_lines);
                            (text, "".to_string())
                        }
                    };
                    text_display.set_label(&display_text);
                    if speak && !spoken_text.is_empty() {
                        tolk.speak(spoken_text, true);
                    }
                };

                let current_screen = *screen_state.lock().unwrap();
                let mut screen_changed = false;
                let mut is_initial_load = false;

                // Handle global music track skipping
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

                // Handle global START button logic for Quick Settings / Pause / Resume
                if action == InputAction::Start {
                    if current_screen == AppScreen::InGame {
                        timer.borrow().stop();
                        audio_engine.play_menu_select();
                        tolk.output("Game Paused. Settings.", true);
                        *screen_state.lock().unwrap() = AppScreen::Settings { selection: 0 };
                        screen_changed = true;
                    } else if let AppScreen::Settings { .. } = current_screen {
                        if *game_in_progress.lock().unwrap() {
                            audio_engine.play_menu_select();
                            tolk.output("Game Resumed", true);
                            let gs = game_state.lock().unwrap();
                            timer.borrow().start(gs.current_speed_ms(), false);
                            *screen_state.lock().unwrap() = AppScreen::InGame;
                        } else {
                            audio_engine.play_menu_select();
                            *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                        }
                        screen_changed = true;
                    } else {
                        audio_engine.play_menu_select();
                        *screen_state.lock().unwrap() = AppScreen::Settings { selection: 0 };
                        screen_changed = true;
                    }
                    if screen_changed { render_in_closure(true, true); }
                    return;
                }

                match current_screen {
                    AppScreen::MainMenu { selection } => {
                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 { selection - 1 } else { 4 };
                                *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < 4 { selection + 1 } else { 0 };
                                *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Select => {
                                audio_engine.play_menu_select();
                                if selection == 0 {
                                    let mut gs = game_state.lock().unwrap();
                                    let mut in_prog = game_in_progress.lock().unwrap();
                                    if !*in_prog {
                                        let diff = settings.lock().unwrap().difficulty;
                                        *gs = GameState::new(diff);
                                        tolk.output("New Game Started!", true);
                                        audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                        *in_prog = true;
                                    } else {
                                        tolk.output("Game Resumed", true);
                                    }
                                    timer.borrow().start(gs.current_speed_ms(), false);
                                    *screen_state.lock().unwrap() = AppScreen::InGame;
                                } else if selection == 1 {
                                    *screen_state.lock().unwrap() = AppScreen::HowToPlay { scroll_line: 0 };
                                    is_initial_load = true;
                                } else if selection == 2 {
                                    *screen_state.lock().unwrap() = AppScreen::Settings { selection: 0 };
                                } else if selection == 3 {
                                    *screen_state.lock().unwrap() = AppScreen::About { scroll_line: 0 };
                                    is_initial_load = true;
                                } else if selection == 4 {
                                    frame.close(true);
                                    return;
                                }
                                screen_changed = true;
                            }
                            InputAction::Back => {
                                frame.close(true);
                            }
                            _ => {}
                        }
                    }
                    AppScreen::Settings { selection } => {
                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 { selection - 1 } else { 4 };
                                *screen_state.lock().unwrap() = AppScreen::Settings { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < 4 { selection + 1 } else { 0 };
                                *screen_state.lock().unwrap() = AppScreen::Settings { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Left => {
                                let mut s = settings.lock().unwrap();
                                if selection == 0 {
                                    s.difficulty = match s.difficulty {
                                        Difficulty::Difficult => Difficulty::Moderate,
                                        Difficulty::Moderate => Difficulty::Easy,
                                        Difficulty::Easy => Difficulty::Easy,
                                    };
                                } else if selection == 1 {
                                    s.sfx_volume = (s.sfx_volume - 0.05).max(0.0);
                                    audio_engine.set_sfx_volume(s.sfx_volume);
                                    audio_engine.play_aligned_sound();
                                } else if selection == 2 {
                                    s.bgm_volume = (s.bgm_volume - 0.05).max(0.0);
                                    audio_engine.set_bgm_volume(s.bgm_volume);
                                } else if selection == 3 {
                                    s.controller_enabled = !s.controller_enabled;
                                    let status = if s.controller_enabled { "Enabled" } else { "Disabled" };
                                    tolk.speak(format!("Gamepad Support {}", status), true);
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
                                        Difficulty::Difficult => Difficulty::Difficult,
                                    };
                                } else if selection == 1 {
                                    s.sfx_volume = (s.sfx_volume + 0.05).min(1.0);
                                    audio_engine.set_sfx_volume(s.sfx_volume);
                                    audio_engine.play_aligned_sound();
                                } else if selection == 2 {
                                    s.bgm_volume = (s.bgm_volume + 0.05).min(1.0);
                                    audio_engine.set_bgm_volume(s.bgm_volume);
                                } else if selection == 3 {
                                    s.controller_enabled = !s.controller_enabled;
                                    let status = if s.controller_enabled { "Enabled" } else { "Disabled" };
                                    tolk.speak(format!("Gamepad Support {}", status), true);
                                }
                                s.save();
                                screen_changed = true;
                            }
                            InputAction::Select | InputAction::Back => {
                                if selection == 3 && action == InputAction::Select {
                                    let mut s = settings.lock().unwrap();
                                    s.controller_enabled = !s.controller_enabled;
                                    s.save();
                                    let status = if s.controller_enabled { "Enabled" } else { "Disabled" };
                                    tolk.speak(format!("Gamepad Support {}", status), true);
                                    screen_changed = true;
                                } else if selection == 4 || action == InputAction::Back {
                                    audio_engine.play_menu_select();
                                    *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 2 };
                                    screen_changed = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    AppScreen::HowToPlay { scroll_line } => {
                        let lines_count = get_how_to_play_lines(settings.lock().unwrap().controller_enabled).len();
                        match action {
                            InputAction::Up => {
                                if scroll_line > 0 {
                                    *screen_state.lock().unwrap() = AppScreen::HowToPlay { scroll_line: scroll_line - 1 };
                                    screen_changed = true;
                                }
                            }
                            InputAction::Down => {
                                if scroll_line < lines_count - 1 {
                                    *screen_state.lock().unwrap() = AppScreen::HowToPlay { scroll_line: scroll_line + 1 };
                                    screen_changed = true;
                                }
                            }
                            InputAction::Select => {
                                let lines = get_how_to_play_lines(settings.lock().unwrap().controller_enabled);
                                tolk.output(lines.join(" "), true);
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 1 };
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::About { scroll_line } => {
                        let lines_count = get_about_lines().len();
                        match action {
                            InputAction::Up => {
                                if scroll_line > 0 {
                                    *screen_state.lock().unwrap() = AppScreen::About { scroll_line: scroll_line - 1 };
                                    screen_changed = true;
                                }
                            }
                            InputAction::Down => {
                                if scroll_line < lines_count - 1 {
                                    *screen_state.lock().unwrap() = AppScreen::About { scroll_line: scroll_line + 1 };
                                    screen_changed = true;
                                }
                            }
                            InputAction::Select => {
                                let lines = get_about_lines();
                                tolk.output(lines.join(" "), true);
                            }
                            InputAction::Back => {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 3 };
                                screen_changed = true;
                            }
                            _ => {}
                        }
                    }
                    AppScreen::InGame => {
                        let mut gs = game_state.lock().unwrap();
                        
                        match action {
                            InputAction::Back => {
                                timer.borrow().stop();
                                tolk.output("Game Paused", true);
                                *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                                screen_changed = true;
                            }
                            InputAction::Radar => {
                                audio_engine.play_radar_sweep(gs.get_topography());
                            }
                            InputAction::Left => {
                                if gs.move_piece(-1, 0) {
                                    audio_engine.play_move_sound(gs.current_piece.x, gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak(format!("Left, col {}", gs.current_piece.x), true);
                                }
                            }
                            InputAction::Right => {
                                if gs.move_piece(1, 0) {
                                    audio_engine.play_move_sound(gs.current_piece.x, gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak(format!("Right, col {}", gs.current_piece.x), true);
                                }
                            }
                            InputAction::RotateRight => {
                                if gs.rotate_piece() {
                                    audio_engine.play_rotate_sound(gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak("Rotated Right", true);
                                }
                            }
                            InputAction::RotateLeft => {
                                if gs.rotate_piece_ccw() {
                                    audio_engine.play_rotate_sound(gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak("Rotated Left", true);
                                }
                            }
                            InputAction::Hold => {
                                if let Some((held, new_p)) = gs.hold() {
                                    audio_engine.play_menu_select();
                                    tolk.speak(format!("Held {}. New piece: {}", held, new_p), true);
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                } else {
                                    audio_engine.play_lock_sound();
                                    tolk.speak("Already held this turn", true);
                                }
                            }
                            InputAction::Select => {
                                // A Button is freed up in-game, does nothing.
                            }
                            InputAction::Down => {
                                if gs.move_piece(0, 1) {
                                    audio_engine.play_move_sound(gs.current_piece.x, gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak(format!("Down, row {}", gs.current_piece.y), true);
                                }
                            }
                            InputAction::HardDrop => {
                                while gs.move_piece(0, 1) {}
                                let lines = gs.lock_piece();
                                if lines > 0 {
                                    audio_engine.play_clear_sound(lines);
                                    tolk.output(format!("Hard drop. Cleared {} lines! Level: {}. Score: {}", lines, gs.level, gs.score), true);
                                    timer.borrow().start(gs.current_speed_ms(), false);
                                } else {
                                    audio_engine.play_lock_sound();
                                }
                                if gs.is_game_over {
                                    timer.borrow().stop();
                                    *game_in_progress.lock().unwrap() = false;
                                    tolk.output(format!("Game Over! Final Score: {}", gs.score), true);
                                    *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                                    screen_changed = true;
                                } else {
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                if screen_changed {
                    render_in_closure(true, is_initial_load);
                }
            }))
        };

        // 3. KEYBOARD INPUT EVENT BINDING
        self.panel.on_key_down({
            let shared_logic = shared_logic.clone();
            move |evt| {
                let keycode = match evt {
                    wxdragon::event::window_events::WindowEventData::Keyboard(ref kbd_event) => {
                        kbd_event.get_key_code().unwrap_or(0)
                    }
                    _ => 0
                };
                
                let current_screen_val = *screen_state.lock().unwrap();
                let action = match keycode {
                    315 => if current_screen_val == AppScreen::InGame { None } else { Some(InputAction::Up) }, // UP Arrow
                    317 => if current_screen_val == AppScreen::InGame { None } else { Some(InputAction::Down) }, // DOWN Arrow
                    314 => if current_screen_val == AppScreen::InGame { None } else { Some(InputAction::Left) }, // LEFT Arrow
                    316 => if current_screen_val == AppScreen::InGame { None } else { Some(InputAction::Right) }, // RIGHT Arrow
                    83 | 115 | 59 => Some(InputAction::Down), // S, ;
                    65 | 97 | 76 | 108 => Some(InputAction::Left), // A, L
                    68 | 100 | 39 => Some(InputAction::Right), // D, '
                    13 | 370 => Some(InputAction::Select), // ENTER
                    27 => Some(InputAction::Back), // ESCAPE
                    9 => Some(InputAction::Start), // TAB (Quick Settings)
                    32 => Some(InputAction::HardDrop), // SPACE
                    67 | 99 | 47 => Some(InputAction::Hold), // C, /
                    90 | 122 | 44 => Some(InputAction::RotateLeft), // Z, ,
                    88 | 120 | 46 => Some(InputAction::RotateRight), // X, .
                    82 | 114 => Some(InputAction::Radar), // R
                    81 | 113 | 80 | 112 => Some(InputAction::PrevTrack), // Q, P
                    87 | 119 | 91 => Some(InputAction::Mute), // W, [
                    69 | 101 | 93 => Some(InputAction::NextTrack), // E, ]
                    _ => None
                };

                if let Some(act) = action {
                    shared_logic.borrow_mut()(act);
                }
            }
        });

        // 4. GAMEPAD POLLING TIMER BINDING
        self.gilrs_timer.borrow().on_tick({
            let shared_logic = shared_logic.clone();
            let gilrs = self.gilrs.clone();
            let settings = self.settings.clone();
            let screen_state = self.screen.clone();
            
            move |_| {
                if !settings.lock().unwrap().controller_enabled {
                    return;
                }
                
                let mut g = gilrs.lock().unwrap();
                while let Some(Event { event, .. }) = g.next_event() {
                    let action = match event {
                        EventType::ButtonPressed(Button::DPadUp, _) => {
                            if *screen_state.lock().unwrap() == AppScreen::InGame {
                                Some(InputAction::Hold)
                            } else {
                                Some(InputAction::Up)
                            }
                        },
                        EventType::ButtonPressed(Button::DPadDown, _) => Some(InputAction::Down),
                        EventType::ButtonPressed(Button::DPadLeft, _) => Some(InputAction::Left),
                        EventType::ButtonPressed(Button::DPadRight, _) => Some(InputAction::Right),
                        EventType::ButtonPressed(Button::South, _) => Some(InputAction::Select), // A
                        EventType::ButtonPressed(Button::East, _) => Some(InputAction::Back), // B
                        EventType::ButtonPressed(Button::Start, _) => Some(InputAction::Start), // Menu/Start
                        EventType::ButtonPressed(Button::RightTrigger2, _) => Some(InputAction::HardDrop), // RT
                        EventType::ButtonPressed(Button::LeftTrigger2, _) => Some(InputAction::Radar), // LT
                        EventType::ButtonPressed(Button::RightTrigger, _) => Some(InputAction::RotateRight), // RB
                        EventType::ButtonPressed(Button::LeftTrigger, _) => Some(InputAction::RotateLeft), // LB
                        EventType::ButtonPressed(Button::LeftThumb, _) => Some(InputAction::PrevTrack), // L3
                        EventType::ButtonPressed(Button::RightThumb, _) => Some(InputAction::NextTrack), // R3
                        _ => None
                    };
                    
                    if let Some(act) = action {
                        shared_logic.borrow_mut()(act);
                    }
                }
            }
        });

        self.frame.on_close(|evt| {
            evt.skip(true); // Allow default close action
        });
    }

    pub fn show(&mut self) {
        self.frame.show(true);
        self.render_screen(true, true); // Speak the initial main menu
        self.gilrs_timer.borrow().start(16, false); // Poll gamepad at ~60Hz
    }
}
