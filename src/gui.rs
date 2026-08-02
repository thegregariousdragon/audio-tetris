use wxdragon::prelude::*;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use crate::logic::GameState;
use crate::audio::AudioEngine;
use crate::settings::{Settings, Difficulty};
use tolk::Tolk;

#[derive(Clone, Copy, PartialEq)]
pub enum AppScreen {
    MainMenu { selection: usize },
    Settings { selection: usize },
    SpeechVerbosity { selection: usize },
    HowToPlay { scroll_line: usize },
    About { scroll_line: usize },
    InGame,
    KeyDescriber { esc_count: usize },
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputAction {
    Up, Down, Left, Right, Select, Back, HardDrop, Radar, RotateLeft, RotateRight, Start, NextTrack, PrevTrack, Mute, Hold,
    Zone, UseItem, HelpMode
}

fn get_how_to_play_lines() -> Vec<String> {
    vec![
        "How to Play Audio Tetris".to_string(),
        "".to_string(),
        "Game Summary:".to_string(),
        "Audio Tetris is a fully screen-reader compatible take on the classic block-stacking puzzle game. Your goal is to navigate falling shapes to the bottom of the board, fitting them together to form solid horizontal lines. Clearing lines earns points and keeps the board from filling up!".to_string(),
        "".to_string(),
        "Game Features:".to_string(),
        "- Radar System: Activate the radar to hear the topography of the board and plan your next move.".to_string(),
        "- Zone Mode: Build up your Zone Meter by clearing lines. Activate the Zone to pause the falling pieces and rack up massive bonus points by clearing multiple lines at once!".to_string(),
        "- Power-ups: Acquire special items during gameplay to help you out of tough spots. Keep an ear out for item spawn and acquisition sounds.".to_string(),
        "- Advanced Scoring: Pull off T-Spins, Back-to-Back clears, and Combos for high scores and satisfying sound effects.".to_string(),
        "".to_string(),
        "Keyboard Controls:".to_string(),
        "Press the H key while on the Main Menu to enter Keyboard Help Mode. In this mode, you can press any key to hear exactly what it does in the game. It's the best way to learn the controls at your own pace!".to_string(),
        "".to_string(),
        "Settings Menu:".to_string(),
        "1. Difficulty: Controls the starting speed and fall rate of the game.".to_string(),
        "2. Speech Verbosity: Opens a sub-menu to customize screen reader callouts for pieces, scoring, and zones.".to_string(),
        "3. Voice Cues Volume: Adjusts the volume of the screen reader's announcements.".to_string(),
        "4. Sound Effects Volume: Adjusts the volume of game sounds like dropping, rotating, and clearing lines.".to_string(),
        "5. Background Music: Toggles the background music on or off.".to_string(),
        "6. Background Music Volume: Adjusts the volume of the background music.".to_string(),
    ]
}

fn get_about_lines() -> Vec<String> {
    vec![
        "About Audio Tetris".to_string(),
        "Version 0.1.0".to_string(),
        "Copyright © 2026 Gregory Lopez and Google Antigravity".to_string(),
        "Released under the MIT License.".to_string(),
        "".to_string(),
        "This project aims to provide a fully accessible, natively compiled Tetris experience for visually impaired gamers, featuring high-performance audio, zero-latency inputs, and direct screen reader speech integration via Tolk.".to_string(),
        "".to_string(),
        "Open Source Components & Licenses:".to_string(),
        "- Rust Language: Developed by the Rust Foundation (MIT / Apache 2.0 License).".to_string(),
        "- wxDragon: Native GUI bindings authored by Allen Dang (MIT License).".to_string(),
        "- Tolk: Screen reader abstraction library authored by Leonard de Ruijter (LGPL License).".to_string(),
        "- Rodio: Audio playback library authored by Tomaka and the RustAudio team (MIT / Apache 2.0 License).".to_string(),
        "- Serde: Data serialization framework authored by David Tolnay (MIT / Apache 2.0 License).".to_string(),
        "".to_string(),
        "Press Backspace or Escape to return to the Main Menu.".to_string(),
    ]
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
        let audio_engine = Rc::new(AudioEngine::new(&settings_data).unwrap());
        let timer = Rc::new(RefCell::new(wxdragon::timer::Timer::new(&frame)));

        Self {
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
        }
    }

    pub fn render_screen(&self, speak: bool, initial_load: bool) {
        let screen = *self.screen.lock().unwrap();
        let s = self.settings.lock().unwrap();
        let in_prog = *self.game_in_progress.lock().unwrap();
        
        let (display_text, spoken_text) = match screen {
            AppScreen::MainMenu { selection } => {
                let opt0 = if in_prog { "Resume Game" } else { "New Game" };
                let options: [&str; 5] = [opt0, "How to Play", "Settings", "About", "Quit"];
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
                    "Speech Verbosity".to_string(),
                    format!("Voice Cues Volume: {}% (controlled by your screen reader)", (s.voice_volume * 100.0) as i32),
                    format!("Sound Effects Volume: {}%", (s.sfx_volume * 100.0) as i32),
                    format!("Background Music: {}", if s.bgm_enabled { "ON" } else { "OFF" }),
                    format!("Background Music Volume: {}%", (s.bgm_volume * 100.0) as i32),
                    "Back".to_string(),
                ];
                let mut text = String::from("Settings\nUse Left and Right arrows to adjust values.\n\n");
                for (i, opt) in options.iter().enumerate() {
                    if i == selection {
                        text.push_str(&format!("->{} {}\n", " ", opt));
                    } else {
                        text.push_str(&format!("   {}\n", opt));
                    }
                }
                (text, options[selection].clone())
            }
            AppScreen::SpeechVerbosity { selection } => {
                let options = [
                    format!("Piece Callouts: {}", if s.piece_callouts_technical { "Terse" } else { "Descriptive" }),
                    format!("Scoring Details: {}", if s.scoring_details_advanced { "Advanced" } else { "Simple" }),
                    format!("Zone Alerts: {}", if s.zone_alerts { "ON" } else { "OFF" }),
                    "Back".to_string(),
                ];
                let mut text = String::from("Speech Verbosity\nUse Left and Right arrows to adjust values.\n\n");
                for (i, opt) in options.iter().enumerate() {
                    if i == selection {
                        text.push_str(&format!("->{} {}\n", " ", opt));
                    } else {
                        text.push_str(&format!("   {}\n", opt));
                    }
                }
                (text, if initial_load { format!("Speech Verbosity Menu. {}", options[selection].clone()) } else { options[selection].clone() })
            }
            AppScreen::HowToPlay { scroll_line } => {
                let lines = get_how_to_play_lines();
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
                    format!("How to play. Use arrows to read line by line. Press Enter to read all. Press Escape to go back. {}", line_text)
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
                    format!("About Audio Tetris. Use arrows to read line by line. Press Enter to read all. Press Escape to go back. {}", line_text)
                } else {
                    line_text
                };
                
                (text, spoken)
            }
            AppScreen::InGame => {
                let gs = self.game_state.lock().unwrap();
                let text = format!("In Game\nLevel: {}\nScore: {}\nLines: {}\n\nPress Escape or Start to pause game.", gs.level, gs.score, gs.total_lines);
                (text, "Game started. Use arrow keys to play.".to_string())
            }
            AppScreen::KeyDescriber { .. } => {
                ("Keyboard Help Mode\nPress any key to hear its function.\nPress Escape twice to exit.".to_string(), "".to_string())
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
        let tolk_instance = self.tolk.clone();
        let screen_state = self.screen.clone();
        let game_in_progress = self.game_in_progress.clone();
        let settings = self.settings.clone();
        let text_display = self.text_display;
        let frame = self.frame;

        // 2. SHARED INPUT LOGIC
        let shared_logic = {
            let game_state = game_state.clone();
            let audio_engine = audio_engine.clone();
                        let tolk = tolk_instance.clone();
            let settings = settings.clone();
            let screen_state = screen_state.clone();
            let game_in_progress = game_in_progress.clone();
            
            Rc::new(RefCell::new(move |action: InputAction| {
                let render_in_closure = |speak: bool, initial_load: bool| {
                    let current_screen = *screen_state.lock().unwrap();
                    let s = settings.lock().unwrap();
                    let in_prog = *game_in_progress.lock().unwrap();
                    
                    let (display_text, spoken_text) = match current_screen {
                        AppScreen::MainMenu { selection } => {
                            let opt0 = if in_prog { "Resume Game" } else { "New Game" };
                            let options: [&str; 5] = [opt0, "How to Play", "Settings", "About", "Quit"];
                            let mut text = String::from("Main Menu\n\n");
                            for (i, opt) in options.iter().enumerate() {
                                if i == selection { text.push_str(&format!("->{} {}\n", " ", opt)); } 
                                else { text.push_str(&format!("   {}\n", opt)); }
                            }
                            (text, format!("{} {} of {}", options[selection], selection + 1, options.len()))
                        }
                        AppScreen::Settings { selection } => {
                            let options = [
                                format!("Difficulty: {}", s.difficulty.as_str()),
                                "Speech Verbosity".to_string(),
                                format!("Voice Cues Volume: {}% (controlled by your screen reader)", (s.voice_volume * 100.0) as i32),
                                format!("Sound Effects Volume: {}%", (s.sfx_volume * 100.0) as i32),
                                format!("Background Music: {}", if s.bgm_enabled { "ON" } else { "OFF" }),
                                format!("Background Music Volume: {}%", (s.bgm_volume * 100.0) as i32),
                                "Back".to_string(),
                            ];
                            let mut text = String::from("Settings\nUse Left and Right arrows to adjust values.\n\n");
                            for (i, opt) in options.iter().enumerate() {
                                if i == selection { text.push_str(&format!("->{} {}\n", " ", opt)); } 
                                else { text.push_str(&format!("   {}\n", opt)); }
                            }
                            (text, format!("{} {} of {}", options[selection], selection + 1, options.len()))
                        }
                        AppScreen::SpeechVerbosity { selection } => {
                            let options = [
                                format!("Piece Callouts: {}", if s.piece_callouts_technical { "Terse" } else { "Descriptive" }),
                                format!("Scoring Details: {}", if s.scoring_details_advanced { "Advanced" } else { "Simple" }),
                                format!("Zone Alerts: {}", if s.zone_alerts { "ON" } else { "OFF" }),
                                "Back".to_string(),
                            ];
                            let mut text = String::from("Speech Verbosity\nUse Left and Right arrows to adjust values.\n\n");
                            for (i, opt) in options.iter().enumerate() {
                                if i == selection { text.push_str(&format!("->{} {}\n", " ", opt)); } 
                                else { text.push_str(&format!("   {}\n", opt)); }
                            }
                            let spoken = format!("{} {} of {}", options[selection], selection + 1, options.len());
                            (text, if initial_load { format!("Speech Verbosity Menu. {}", spoken) } else { spoken })
                        }
                        AppScreen::HowToPlay { scroll_line } => {
                            let lines = get_how_to_play_lines();
                            let mut text = String::new();
                            for (i, line) in lines.iter().enumerate() {
                                if i == scroll_line { text.push_str(&format!("-> {}\n", line)); } 
                                else { text.push_str(&format!("   {}\n", line)); }
                            }
                            let line_text = if scroll_line < lines.len() { lines[scroll_line].clone() } else { "".to_string() };
                            let spoken = if initial_load {
                                format!("How to play. Use arrows to read line by line. Press Enter to read all. Press Escape to go back. {}", line_text)
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
                                format!("About Audio Tetris. Use arrows to read line by line. Press Enter to read all. Press Escape to go back. {}", line_text)
                            } else { line_text };
                            (text, spoken)
                        }
                        AppScreen::InGame => {
                            let gs = game_state.lock().unwrap();
                            let text = format!("In Game\nLevel: {}\nScore: {}\nLines: {}\n\nPress Escape to pause game.", gs.level, gs.score, gs.total_lines);
                            (text, "".to_string())
                        }
                        AppScreen::KeyDescriber { .. } => ("".to_string(), "".to_string()),
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
                        audio_engine.play_menu_select();
                        tolk.output("Game Paused. Settings.", true);
                        *screen_state.lock().unwrap() = AppScreen::Settings { selection: 0 };
                        screen_changed = true;
                    } else if let AppScreen::Settings { .. } = current_screen {
                        if *game_in_progress.lock().unwrap() {
                            audio_engine.play_menu_select();
                            tolk.output("Game Resumed", true);
                            let _gs = game_state.lock().unwrap();
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
                            InputAction::HelpMode => {
                                audio_engine.play_menu_select();
                                *screen_state.lock().unwrap() = AppScreen::KeyDescriber { esc_count: 0 };
                                tolk.output("Keyboard Help Mode. Press any key to hear its function. Press Escape twice to exit.", true);
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
                                let new_sel = if selection > 0 { selection - 1 } else { 6 };
                                *screen_state.lock().unwrap() = AppScreen::Settings { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < 6 { selection + 1 } else { 0 };
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
                                    audio_engine.play_menu_select();
                                    *screen_state.lock().unwrap() = AppScreen::SpeechVerbosity { selection: 0 };
                                } else if selection == 2 {
                                    s.voice_volume = (s.voice_volume - 0.05).max(0.0);
                                    tolk.speak(format!("Voice Cues Volume {}%", (s.voice_volume * 100.0) as i32), true);
                                } else if selection == 3 {
                                    s.sfx_volume = (s.sfx_volume - 0.05).max(0.0);
                                    audio_engine.set_sfx_volume(s.sfx_volume);
                                    audio_engine.play_aligned_sound();
                                } else if selection == 4 {
                                    s.bgm_enabled = !s.bgm_enabled;
                                    if s.bgm_enabled {
                                        s.bgm_volume = if s.saved_bgm_volume > 0.0 { s.saved_bgm_volume } else { 0.2 };
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
                                        tolk.speak(format!("Background Music On. Volume {}%", (s.bgm_volume * 100.0) as i32), true);
                                    }
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
                                    audio_engine.play_menu_select();
                                    *screen_state.lock().unwrap() = AppScreen::SpeechVerbosity { selection: 0 };
                                } else if selection == 2 {
                                    s.voice_volume = (s.voice_volume + 0.05).min(1.0);
                                    tolk.speak(format!("Voice Cues Volume {}%", (s.voice_volume * 100.0) as i32), true);
                                } else if selection == 3 {
                                    s.sfx_volume = (s.sfx_volume + 0.05).min(1.0);
                                    audio_engine.set_sfx_volume(s.sfx_volume);
                                    audio_engine.play_aligned_sound();
                                } else if selection == 4 {
                                    s.bgm_enabled = !s.bgm_enabled;
                                    if s.bgm_enabled {
                                        s.bgm_volume = if s.saved_bgm_volume > 0.0 { s.saved_bgm_volume } else { 0.2 };
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
                                        tolk.speak(format!("Background Music On. Volume {}%", (s.bgm_volume * 100.0) as i32), true);
                                    }
                                }
                                s.save();
                                screen_changed = true;
                            }
                            InputAction::Select | InputAction::Back => {
                                if selection == 1 && action == InputAction::Select {
                                    audio_engine.play_menu_select();
                                    *screen_state.lock().unwrap() = AppScreen::SpeechVerbosity { selection: 0 };
                                    screen_changed = true;
                                } else if selection == 4 && action == InputAction::Select {
                                    let mut s = settings.lock().unwrap();
                                    s.bgm_enabled = !s.bgm_enabled;
                                    if s.bgm_enabled {
                                        s.bgm_volume = if s.saved_bgm_volume > 0.0 { s.saved_bgm_volume } else { 0.2 };
                                        audio_engine.set_bgm_volume(s.bgm_volume);
                                    } else {
                                        s.saved_bgm_volume = s.bgm_volume;
                                        s.bgm_volume = 0.0;
                                        audio_engine.set_bgm_volume(0.0);
                                    }
                                    audio_engine.set_bgm_enabled(s.bgm_enabled);
                                    s.save();
                                    let status = if s.bgm_enabled { "On" } else { "Off" };
                                    tolk.speak(format!("Background Music {}", status), true);
                                    screen_changed = true;
                                } else if selection == 6 || action == InputAction::Back {
                                    audio_engine.play_menu_select();
                                    *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 2 };
                                    screen_changed = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    AppScreen::SpeechVerbosity { selection } => {
                        match action {
                            InputAction::Up => {
                                let new_sel = if selection > 0 { selection - 1 } else { 3 };
                                *screen_state.lock().unwrap() = AppScreen::SpeechVerbosity { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Down => {
                                let new_sel = if selection < 3 { selection + 1 } else { 0 };
                                *screen_state.lock().unwrap() = AppScreen::SpeechVerbosity { selection: new_sel };
                                audio_engine.play_menu_move();
                                screen_changed = true;
                            }
                            InputAction::Left | InputAction::Right | InputAction::Select => {
                                let mut s = settings.lock().unwrap();
                                if selection == 0 {
                                    s.piece_callouts_technical = !s.piece_callouts_technical;
                                    let status = if s.piece_callouts_technical { "Terse" } else { "Descriptive" };
                                    tolk.speak(format!("Piece Callouts: {}", status), true);
                                } else if selection == 1 {
                                    s.scoring_details_advanced = !s.scoring_details_advanced;
                                    let status = if s.scoring_details_advanced { "Advanced" } else { "Simple" };
                                    tolk.speak(format!("Scoring Details: {}", status), true);
                                } else if selection == 2 {
                                    s.zone_alerts = !s.zone_alerts;
                                    let status = if s.zone_alerts { "On" } else { "Off" };
                                    tolk.speak(format!("Zone Alerts: {}", status), true);
                                } else if selection == 3 && action == InputAction::Select {
                                    audio_engine.play_menu_select();
                                    *screen_state.lock().unwrap() = AppScreen::Settings { selection: 1 };
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
                        }
                    }
                    AppScreen::HowToPlay { scroll_line } => {
                        let lines_count = get_how_to_play_lines().len();
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
                                let lines = get_how_to_play_lines();
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
                                tolk.output("Game Paused", true);
                                *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                                screen_changed = true;
                            }
                            InputAction::Radar => {
                                audio_engine.play_radar_sweep(gs.get_topography());
                            }
                            InputAction::Left => {
                                if gs.move_piece(-1, 0) {
                                    audio_engine.play_horizontal_move_sound(gs.current_piece.x);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak(format!("Left, column {}", gs.current_piece.x), true);
                                }
                            }
                            InputAction::Right => {
                                if gs.move_piece(1, 0) {
                                    audio_engine.play_horizontal_move_sound(gs.current_piece.x);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak(format!("Right, column {}", gs.current_piece.x), true);
                                }
                            }
                            InputAction::RotateRight => {
                                if gs.rotate_piece() {
                                    audio_engine.play_rotate_cw_sound(gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak("Rotated Right", true);
                                }
                            }
                            InputAction::RotateLeft => {
                                if gs.rotate_piece_ccw() {
                                    audio_engine.play_rotate_ccw_sound(gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak("Rotated Left", true);
                                }
                            }
                            InputAction::Hold => {
                                if let Some((is_swap, held, new_p)) = gs.hold() {
                                    if is_swap {
                                        audio_engine.play_hold_swap_sound();
                                    } else {
                                        audio_engine.play_hold_sound();
                                    }
                                    tolk.speak(format!("Held {:?}. New piece: {:?}", held, new_p), true);
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                } else {
                                    audio_engine.play_hold_denied_sound();
                                    tolk.speak("Already held this turn", true);
                                }
                            }
                            InputAction::Select => {
                                // A Button is freed up in-game, does nothing.
                            }
                            InputAction::Down => {
                                if gs.move_piece(0, 1) {
                                    audio_engine.play_soft_drop_sound(gs.current_piece.y);
                                    if gs.is_perfect_fit() { audio_engine.play_aligned_sound(); }
                                    tolk.speak(format!("Down, row {}", gs.current_piece.y), true);
                                }
                            }
                            InputAction::HardDrop => {
                                while gs.move_piece(0, 1) {}
                                audio_engine.play_hard_drop_sound();
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
                                    let mut tts = format!("Hard drop. Cleared {} lines!", res.cleared_lines);
                                    if res.is_t_spin { 
                                        audio_engine.play_t_spin_sound();
                                        if scoring_advanced { tts.push_str(" T-Spin!"); }
                                    }
                                    if res.b2b_bonus { 
                                        audio_engine.play_b2b_sound();
                                        if scoring_advanced { tts.push_str(" Back to back!"); }
                                    }
                                    if res.combo > 1 && scoring_advanced { tts.push_str(&format!(" {} Combo!", res.combo)); }
                                    tts.push_str(&format!(" Level: {}. Score: {}", gs.level, gs.score));
                                    tolk.output(tts, true);
                                } else {
                                    if res.is_t_spin && scoring_advanced {
                                        audio_engine.play_t_spin_sound();
                                        tolk.output(format!("T-Spin! Score: {}", gs.score), true);
                                    }
                                    tolk.output(format!("Hard drop. Score: {}", gs.score), true);
                                }
                                
                                if gs.is_game_over {
                                    *game_in_progress.lock().unwrap() = false;
                                    tolk.output(format!("Game Over! Final Score: {}", gs.score), true);
                                    *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                                } else {
                                    audio_engine.play_spawn_sound(gs.current_piece.t_type);
                                    if let Some(acquired) = gs.item_acquired {
                                        audio_engine.play_item_acquire();
                                        tolk.output(format!("Acquired {}!", acquired.as_str()), true);
                                    }
                                    if let Some(spawned) = gs.item_spawned {
                                        audio_engine.play_item_spawn();
                                        tolk.output(format!("{} spawned!", spawned.as_str()), false);
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
                    AppScreen::KeyDescriber { .. } => {}
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

            move |_| {
                let interval = 16;
                
                // --- GAME DROP TIMER ---
                if *screen.lock().unwrap() != AppScreen::InGame { return; }
                let mut gs = game_state.lock().unwrap();
                if gs.is_game_over { return; }
                
                if gs.is_in_zone {
                    gs.zone_timer_ms -= interval;
                    if gs.zone_timer_ms <= 0 {
                        let lines = gs.end_zone();
                        if lines > 0 {
                            audio_engine.play_clear_sound(lines);
                            tolk.output(format!("Zone ended! Cleared {} lines. Score: {}", lines, gs.score), true);
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
                                        if scoring_advanced { tts.push_str(" T-Spin!"); }
                            }
                            if res.b2b_bonus { 
                                        audio_engine.play_b2b_sound();
                                        if scoring_advanced { tts.push_str(" Back to back!"); }
                            }
                            if res.combo > 1 && scoring_advanced { tts.push_str(&format!(" {} Combo!", res.combo)); }
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
                            *in_prog.lock().unwrap() = false;
                            tolk.output(format!("Game Over! Final Score: {}", gs.score), true);
                            *screen.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                        } else {
                            audio_engine.play_spawn_sound(gs.current_piece.t_type);
                            if let Some(acquired) = gs.item_acquired {
                                audio_engine.play_item_acquire();
                                tolk.output(format!("Acquired {}!", acquired.as_str()), true);
                            }
                            if let Some(spawned) = gs.item_spawned {
                                audio_engine.play_item_spawn();
                                tolk.output(format!("{} spawned!", spawned.as_str()), false);
                            }
                        }
                    }
                    return; // exit early for lock delay ticks
                }

                // Normal fall logic
                gs.fall_timer_ms += interval;
                if gs.fall_timer_ms >= gs.current_speed_ms() {
                    gs.fall_timer_ms = 0;
                    
                    if !gs.is_in_zone {
                        if gs.can_move_down() {
                            gs.move_piece(0, 1);
                            audio_engine.play_soft_drop_sound(gs.current_piece.y);
                        } else {
                            gs.lock_delay_active = true;
                            gs.lock_delay_timer_ms = 500;
                            gs.moves_since_lock_delay = 0;
                            audio_engine.play_lock_delay_warning();
                        }
                    } else {
                        if !gs.can_move_down() {
                            gs.lock_delay_active = true;
                            gs.lock_delay_timer_ms = 500;
                            gs.moves_since_lock_delay = 0;
                            audio_engine.play_lock_delay_warning();
                        }
                    }
                    // Check danger state based on max column height
                    let max_h = gs.get_topography().iter().copied().max().unwrap_or(0);
                    audio_engine.update_danger_state(max_h);
                }
            }
        });
        // 3. KEYBOARD INPUT EVENT BINDING
        self.panel.on_key_down({
            let shared_logic = shared_logic.clone();
            let screen_state = screen_state.clone();
            let tolk = tolk_instance.clone();
            let audio_engine = audio_engine.clone();
            move |evt| {
                let keycode = match evt {
                    wxdragon::event::window_events::WindowEventData::Keyboard(ref kbd_event) => {
                        kbd_event.get_key_code().unwrap_or(0)
                    }
                    _ => 0
                };

                // --- KEYBOARD HELP MODE INTERCEPT ---
                let current_screen_val = *screen_state.lock().unwrap();
                if let AppScreen::KeyDescriber { esc_count } = current_screen_val {
                    if keycode == 27 { // Escape
                        let new_count = esc_count + 1;
                        if new_count >= 2 {
                            audio_engine.play_menu_select();
                            tolk.output("Exiting Keyboard Help Mode.", true);
                            *screen_state.lock().unwrap() = AppScreen::MainMenu { selection: 0 };
                        } else {
                            *screen_state.lock().unwrap() = AppScreen::KeyDescriber { esc_count: new_count };
                            tolk.output("Press Escape again to exit Keyboard Help Mode.", true);
                        }
                    } else {
                        // Reset esc count on any other key
                        if esc_count > 0 {
                            *screen_state.lock().unwrap() = AppScreen::KeyDescriber { esc_count: 0 };
                        }
                        let desc = match keycode {
                            315 => "Up Arrow. Hard Drop in game, or Menu Up.",
                            317 => "Down Arrow. Soft Drop in game, or Menu Down.",
                            314 => "Left Arrow. Move piece left.",
                            316 => "Right Arrow. Move piece right.",
                            87 | 119 => "W. Hard Drop in game, or Menu Up.",
                            83 | 115 => "S. Soft Drop in game, or Menu Down.",
                            65 | 97 => "A. Move piece left.",
                            68 | 100 => "D. Move piece right.",
                            13 | 370 => "Enter. Apply option or Start game.",
                            27 => "Escape. Back or Pause.", // won't reach here but for completeness
                            9 => "Tab. Quick Settings or Pause.",
                            32 => "Spacebar. Hard Drop.",
                            67 | 99 => "C. Hold piece.",
                            47 => "Slash. Hold piece.",
                            90 | 122 => "Z. Rotate left.",
                            44 => "Comma. Rotate left.",
                            88 | 120 => "X. Rotate right.",
                            46 => "Period. Rotate right.",
                            69 | 101 => "E. Radar sweep.",
                            78 | 110 => "N. Radar sweep.",
                            81 | 113 => "Q. Activate Zone.",
                            77 | 109 => "M. Activate Zone.",
                            306 | 340 | 344 | 160 | 161 => "Shift. Use Power-up Item.",
                            45 => "Minus. Previous music track.",
                            61 | 43 => "Plus or Equals. Next music track.",
                            72 | 104 => "H. Enter Keyboard Help Mode.",
                            _ => {
                                tolk.output(format!("Unmapped key. Code: {}", keycode), true);
                                return;
                            }
                        };
                        tolk.output(desc, true);
                    }
                    return; // Don't process further when in help mode
                }

                // --- NORMAL KEY PROCESSING ---
                let action = match keycode {
                    315 => if current_screen_val == AppScreen::InGame { Some(InputAction::HardDrop) } else { Some(InputAction::Up) }, // UP Arrow
                    317 => Some(InputAction::Down), // DOWN Arrow
                    314 => Some(InputAction::Left), // LEFT Arrow
                    316 => Some(InputAction::Right), // RIGHT Arrow
                    87 | 119 => if current_screen_val == AppScreen::InGame { Some(InputAction::HardDrop) } else { Some(InputAction::Up) }, // W
                    83 | 115 => Some(InputAction::Down), // S
                    65 | 97 => Some(InputAction::Left), // A
                    68 | 100 => Some(InputAction::Right), // D
                    13 | 370 => Some(InputAction::Select), // ENTER
                    27 => Some(InputAction::Back), // ESCAPE
                    9 => Some(InputAction::Start), // TAB (Quick Settings)
                    32 => Some(InputAction::HardDrop), // SPACE
                    67 | 99 | 47 => Some(InputAction::Hold), // C, /
                    90 | 122 | 44 => Some(InputAction::RotateLeft), // Z, ,
                    88 | 120 | 46 => Some(InputAction::RotateRight), // X, .
                    69 | 101 | 78 | 110 => Some(InputAction::Radar), // E, N
                    81 | 113 | 77 | 109 => Some(InputAction::Zone), // Q, M
                    306 | 340 | 344 | 160 | 161 => Some(InputAction::UseItem), // Left/Right Shift
                    45 => Some(InputAction::PrevTrack), // Minus
                    61 | 43 => Some(InputAction::NextTrack), // Equals/Plus
                    72 | 104 => Some(InputAction::HelpMode), // H
                    _ => None
                };

                if let Some(act) = action {
                    shared_logic.borrow_mut()(act);
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
        }
}
