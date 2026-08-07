use lofty::prelude::*;
use lofty::probe::Probe;
use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::logic::{ItemType, TetrominoType};
use crate::settings::Settings;

// ---------------------------------------------------------------------------
// Stereo-panned sine wave via SamplesBuffer (guaranteed channel delivery)
// ---------------------------------------------------------------------------

/// Build a stereo `SamplesBuffer` containing a sine wave with equal-power panning.
/// `pan` ranges from -1.0 (full left) to +1.0 (full right). 0.0 = center.
fn make_panned_sine(freq: f32, duration_ms: u64, pan: f32, amplitude: f32) -> SamplesBuffer<f32> {
    let sample_rate: u32 = 44100;
    let num_frames = (sample_rate as f64 * duration_ms as f64 / 1000.0) as usize;

    let clamped = pan.clamp(-1.0, 1.0);
    // Equal-power panning: left_gain = cos(theta), right_gain = sin(theta)
    let theta = ((clamped + 1.0) / 2.0) * std::f32::consts::FRAC_PI_2;
    let left_gain = theta.cos();
    let right_gain = theta.sin();

    let mut samples = Vec::with_capacity(num_frames * 2);
    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;
        let raw = (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude;
        samples.push(raw * left_gain); // Left channel
        samples.push(raw * right_gain); // Right channel
    }

    SamplesBuffer::new(2, sample_rate, samples)
}

// ---------------------------------------------------------------------------
// AudioEngine
// ---------------------------------------------------------------------------

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    bgm_sink: Arc<Mutex<Sink>>,
    bgm_tracks: Arc<Vec<(PathBuf, String)>>,
    current_track: Arc<Mutex<usize>>,
    bgm_enabled: Arc<Mutex<bool>>,
    sfx_volume: Arc<Mutex<f32>>,
    bgm_volume_setting: Arc<Mutex<f32>>,
    /// Tracks the last time a danger ping was played (as epoch millis).
    last_danger_ping_ms: Arc<Mutex<u64>>,
    /// Whether danger mode is currently active.
    danger_active: Arc<Mutex<bool>>,
}

impl AudioEngine {
    pub fn new(settings: &Settings) -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let bgm_sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle)?));

        let mut tracks = Vec::new();
        if let Ok(entries) = std::fs::read_dir("assets/music") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(ext) = path.extension().and_then(|s| s.to_str())
                {
                    let ext = ext.to_lowercase();
                    if ext == "wav" || ext == "mp3" || ext == "ogg" || ext == "flac" {
                        let mut title = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        if let Ok(tagged_file) = Probe::open(&path).and_then(|p| p.read())
                            && let Some(tag) = tagged_file
                                .primary_tag()
                                .or_else(|| tagged_file.first_tag())
                            && let Some(t) = tag.title()
                        {
                            title = t.into_owned();
                        }
                        tracks.push((path, title));
                    }
                }
            }
        }
        tracks.sort_by(|a, b| a.0.cmp(&b.0));
        let bgm_tracks = Arc::new(tracks);
        let current_track = Arc::new(Mutex::new(0));
        let bgm_enabled = Arc::new(Mutex::new(settings.bgm_enabled));
        let sfx_volume = Arc::new(Mutex::new(settings.sfx_volume));
        let bgm_volume_setting = Arc::new(Mutex::new(settings.bgm_volume));

        let engine = Self {
            _stream,
            stream_handle,
            bgm_sink,
            bgm_tracks,
            current_track,
            bgm_enabled,
            sfx_volume,
            bgm_volume_setting,
            last_danger_ping_ms: Arc::new(Mutex::new(0)),
            danger_active: Arc::new(Mutex::new(false)),
        };

        engine.start_bgm_thread();
        Ok(engine)
    }

    pub fn set_sfx_volume(&self, vol: f32) {
        *self.sfx_volume.lock().unwrap() = vol;
    }

    pub fn set_bgm_volume(&self, vol: f32) {
        *self.bgm_volume_setting.lock().unwrap() = vol;
        let enabled = *self.bgm_enabled.lock().unwrap();
        let s = self.bgm_sink.lock().unwrap();
        if !enabled {
            s.set_volume(0.0);
        } else {
            s.set_volume(vol);
        }
    }

    pub fn start_bgm_thread(&self) {
        let sink = self.bgm_sink.clone();
        let bgm_tracks = self.bgm_tracks.clone();
        let track_idx = self.current_track.clone();
        let enabled = self.bgm_enabled.clone();
        let bgm_vol = self.bgm_volume_setting.clone();

        thread::spawn(move || {
            loop {
                let is_empty = { sink.lock().unwrap().empty() };
                if is_empty && !bgm_tracks.is_empty() {
                    let idx = *track_idx.lock().unwrap();
                    let path = &bgm_tracks[idx % bgm_tracks.len()].0;
                    if let Ok(file) = File::open(path) {
                        let reader = BufReader::new(file);
                        if let Ok(decoder) = Decoder::new(reader) {
                            let s = sink.lock().unwrap();
                            s.append(decoder);
                            if !*enabled.lock().unwrap() {
                                s.set_volume(0.0);
                            } else {
                                s.set_volume(*bgm_vol.lock().unwrap());
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn next_track(&self) -> String {
        if self.bgm_tracks.is_empty() {
            return "No tracks found".to_string();
        }
        let mut idx = self.current_track.lock().unwrap();
        *idx = (*idx + 1) % self.bgm_tracks.len();
        self.bgm_sink.lock().unwrap().clear();

        let track = &self.bgm_tracks[*idx];
        track.1.clone()
    }

    pub fn prev_track(&self) -> String {
        if self.bgm_tracks.is_empty() {
            return "No tracks found".to_string();
        }
        let mut idx = self.current_track.lock().unwrap();
        if *idx == 0 {
            *idx = self.bgm_tracks.len() - 1;
        } else {
            *idx -= 1;
        }
        self.bgm_sink.lock().unwrap().clear();

        let track = &self.bgm_tracks[*idx];
        track.1.clone()
    }

    pub fn set_bgm_enabled(&self, enabled: bool) {
        let mut e = self.bgm_enabled.lock().unwrap();
        *e = enabled;
        let s = self.bgm_sink.lock().unwrap();
        if !enabled {
            s.set_volume(0.0);
        } else {
            s.set_volume(*self.bgm_volume_setting.lock().unwrap());
        }
    }

    // -----------------------------------------------------------------------
    // Helper: play a center-panned mono sine on a fire-and-forget thread
    // -----------------------------------------------------------------------
    fn play_sine(&self, freq: f32, duration_ms: u64, vol_multiplier: f32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * vol_multiplier;
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(freq)
                .take_duration(Duration::from_millis(duration_ms))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    /// Play a stereo-panned sine wave.
    /// `pan`: -1.0 = full left, 0.0 = center, +1.0 = full right.
    fn play_panned_sine(&self, freq: f32, duration_ms: u64, pan: f32, vol_multiplier: f32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * vol_multiplier;
        thread::spawn(move || {
            let source = make_panned_sine(freq, duration_ms, pan, vol);
            let sink = Sink::try_new(&handle).unwrap();
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    // =======================================================================
    // MOVEMENT & BOARD NAVIGATION
    // =======================================================================

    /// Soft drop (moving down row by row).
    /// Low-frequency descending click — pitch drops as piece approaches the bottom.
    pub fn play_soft_drop_sound(&self, y: i32) {
        // Base 300 Hz, drops ~10 Hz per row. At row 19 (bottom) it's ~110 Hz.
        let freq = (300.0 - (y as f32 * 10.0)).max(110.0);
        self.play_sine(freq, 40, 1.0);
    }

    /// Horizontal movement (left/right).
    /// Light quick tick, stereo-panned by column position.
    /// Column 0 = hard left (-1.0), column 9 = hard right (+1.0).
    pub fn play_horizontal_move_sound(&self, x: i32) {
        let pan = (x as f32 / 9.0) * 2.0 - 1.0; // maps 0..9 to -1.0..+1.0
        self.play_panned_sine(600.0, 25, pan, 1.0);
    }

    /// Hard drop — heavy impact sound.
    /// Low fundamental + harmonic for a thick, decisive thud.
    pub fn play_hard_drop_sound(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 1.5;
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            // Layer two frequencies for a thick impact
            let fundamental = rodio::source::SineWave::new(80.0)
                .take_duration(Duration::from_millis(200))
                .amplify(vol);
            let harmonic = rodio::source::SineWave::new(160.0)
                .take_duration(Duration::from_millis(150))
                .amplify(vol * 0.6);
            // Mix by appending to sink (they play sequentially, so use a second sink)
            sink.append(fundamental);
            // Use a second sink for the harmonic overlay
            let sink2 = Sink::try_new(&handle).unwrap();
            sink2.append(harmonic);
            sink.sleep_until_end();
            sink2.sleep_until_end();
        });
    }

    /// Lock sound for auto-drop lock (softer than hard drop).
    pub fn play_lock_sound(&self) {
        self.play_sine(130.0, 120, 1.2);
    }

    // =======================================================================
    // PIECE ROTATION
    // =======================================================================

    /// Clockwise rotation — ascending two-note blip: C4 → G4.
    /// Slight vertical offset so the player can sense height.
    pub fn play_rotate_cw_sound(&self, y: i32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        let offset = y as f32 * 3.0; // subtle height context
        thread::spawn(move || {
            // Note 1: C4 (261.63 Hz)
            let sink = Sink::try_new(&handle).unwrap();
            let note1 = rodio::source::SineWave::new(261.63 - offset)
                .take_duration(Duration::from_millis(60))
                .amplify(vol);
            sink.append(note1);
            sink.sleep_until_end();
            // Note 2: G4 (392.00 Hz)
            let sink2 = Sink::try_new(&handle).unwrap();
            let note2 = rodio::source::SineWave::new(392.00 - offset)
                .take_duration(Duration::from_millis(60))
                .amplify(vol);
            sink2.append(note2);
            sink2.sleep_until_end();
        });
    }

    /// Counter-clockwise rotation — descending two-note blip: G4 → C4.
    pub fn play_rotate_ccw_sound(&self, y: i32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        let offset = y as f32 * 3.0;
        thread::spawn(move || {
            // Note 1: G4 (392.00 Hz)
            let sink = Sink::try_new(&handle).unwrap();
            let note1 = rodio::source::SineWave::new(392.00 - offset)
                .take_duration(Duration::from_millis(60))
                .amplify(vol);
            sink.append(note1);
            sink.sleep_until_end();
            // Note 2: C4 (261.63 Hz)
            let sink2 = Sink::try_new(&handle).unwrap();
            let note2 = rodio::source::SineWave::new(261.63 - offset)
                .take_duration(Duration::from_millis(60))
                .amplify(vol);
            sink2.append(note2);
            sink2.sleep_until_end();
        });
    }

    // =======================================================================
    // HOLD QUEUE & PIECE SWAPPING
    // =======================================================================

    /// First time holding a piece — soft inward "pop" (frequency sweep up).
    pub fn play_hold_sound(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        thread::spawn(move || {
            // Quick ascending sweep: 200 Hz → 400 Hz over 80ms
            let steps = 8;
            for i in 0..steps {
                let freq = 200.0 + (200.0 * (i as f32 / steps as f32));
                let sink = Sink::try_new(&handle).unwrap();
                let source = rodio::source::SineWave::new(freq)
                    .take_duration(Duration::from_millis(10))
                    .amplify(vol);
                sink.append(source);
                sink.sleep_until_end();
            }
        });
    }

    /// Swapping out a held piece — two-phase swoosh-clack.
    pub fn play_hold_swap_sound(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        thread::spawn(move || {
            // Phase 1: rising sweep (swoosh) — 300 Hz → 500 Hz over 60ms
            let steps = 6;
            for i in 0..steps {
                let freq = 300.0 + (200.0 * (i as f32 / steps as f32));
                let sink = Sink::try_new(&handle).unwrap();
                let source = rodio::source::SineWave::new(freq)
                    .take_duration(Duration::from_millis(10))
                    .amplify(vol);
                sink.append(source);
                sink.sleep_until_end();
            }
            // Phase 2: sharp clack — 250 Hz snap
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(250.0)
                .take_duration(Duration::from_millis(40))
                .amplify(vol * 1.3);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    /// Hold locked out — low error buzz.
    pub fn play_hold_denied_sound(&self) {
        // Low-register buzz at 90 Hz, 120ms
        self.play_sine(90.0, 120, 1.0);
    }

    // =======================================================================
    // ROW CLEARS & GAME STATE
    // =======================================================================

    /// Line clear sounds — tiered by number of lines.
    pub fn play_clear_sound(&self, lines: u32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 2.0;
        thread::spawn(move || {
            match lines {
                1 => {
                    // Single: major triad chord (C5 + E5 + G5 simultaneously)
                    let sink_c = Sink::try_new(&handle).unwrap();
                    let sink_e = Sink::try_new(&handle).unwrap();
                    let sink_g = Sink::try_new(&handle).unwrap();
                    sink_c.append(
                        rodio::source::SineWave::new(523.25)
                            .take_duration(Duration::from_millis(250))
                            .amplify(vol),
                    );
                    sink_e.append(
                        rodio::source::SineWave::new(659.25)
                            .take_duration(Duration::from_millis(250))
                            .amplify(vol * 0.8),
                    );
                    sink_g.append(
                        rodio::source::SineWave::new(783.99)
                            .take_duration(Duration::from_millis(250))
                            .amplify(vol * 0.7),
                    );
                    sink_c.sleep_until_end();
                }
                2 => {
                    // Double: 3-note ascending arpeggio C5 → E5 → G5
                    let notes = [523.25, 659.25, 783.99];
                    for freq in notes {
                        let sink = Sink::try_new(&handle).unwrap();
                        sink.append(
                            rodio::source::SineWave::new(freq)
                                .take_duration(Duration::from_millis(60))
                                .amplify(vol),
                        );
                        sink.sleep_until_end();
                    }
                }
                3 => {
                    // Triple: 4-note ascending arpeggio C5 → E5 → G5 → C6
                    let notes = [523.25, 659.25, 783.99, 1046.50];
                    for freq in notes {
                        let sink = Sink::try_new(&handle).unwrap();
                        sink.append(
                            rodio::source::SineWave::new(freq)
                                .take_duration(Duration::from_millis(60))
                                .amplify(vol * 1.2),
                        );
                        sink.sleep_until_end();
                    }
                }
                4 => {
                    // Tetris! Rapid chiptune fanfare: C5 → D5 → E5 → G5 → A5 → C6
                    let notes = [523.25, 587.33, 659.25, 783.99, 880.00, 1046.50];
                    for freq in notes {
                        let sink = Sink::try_new(&handle).unwrap();
                        sink.append(
                            rodio::source::SineWave::new(freq)
                                .take_duration(Duration::from_millis(40))
                                .amplify(vol * 1.5),
                        );
                        sink.sleep_until_end();
                    }
                }
                _ => {}
            }
        });
    }

    // =======================================================================
    // DANGER WARNING
    // =======================================================================

    /// Call this each game tick with the current max column height (0..20).
    /// When height >= 15 (75% of 20-row board), a sonar ping starts pulsing
    /// with increasing tempo as the stack approaches the top.
    pub fn update_danger_state(&self, max_height: u32) {
        let threshold = 15; // 75% of 20 rows
        if max_height < threshold {
            *self.danger_active.lock().unwrap() = false;
            return;
        }

        *self.danger_active.lock().unwrap() = true;

        // Calculate ping interval: 2000ms at threshold, linearly decreasing to 300ms at height 20
        let progress = ((max_height - threshold) as f32) / (20 - threshold) as f32; // 0.0 to 1.0
        let interval_ms = 2000.0 - (1700.0 * progress); // 2000ms → 300ms

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut last = self.last_danger_ping_ms.lock().unwrap();
        if now - *last >= interval_ms as u64 {
            *last = now;
            // Fire the sonar ping
            self.play_sine(70.0, 100, 0.8);
        }
    }

    // =======================================================================
    // ALIGNMENT & SPAWN (kept/refined)
    // =======================================================================

    /// Perfect fit chime — bright high transient, stays above speech band at 1200 Hz
    /// but is short enough (<50ms) to not mask.
    pub fn play_aligned_sound(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 0.75;
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(1200.0)
                .take_duration(Duration::from_millis(50))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    /// Piece spawn — each piece type has a distinct base frequency for identification.
    pub fn play_spawn_sound(&self, t_type: TetrominoType) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();

            let base_freq = match t_type {
                TetrominoType::I => 440.0,
                TetrominoType::J => 493.88,
                TetrominoType::L => 523.25,
                TetrominoType::O => 587.33,
                TetrominoType::S => 659.25,
                TetrominoType::T => 698.46,
                TetrominoType::Z => 783.99,
            };

            let source1 = rodio::source::SineWave::new(base_freq)
                .take_duration(Duration::from_millis(150))
                .amplify(vol);

            sink.append(source1);
            sink.sleep_until_end();
        });
    }

    /// Radar sweep — plays a tone per column, pitch reflecting stack height.
    /// Each column is stereo-panned left-to-right across the stereo field.
    pub fn play_radar_sweep(&self, heights: Vec<u32>) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        thread::spawn(move || {
            for (i, height) in heights.iter().enumerate() {
                let freq = if *height == 0 {
                    110.0 // Low hum for empty column
                } else {
                    220.0 + (*height as f32 * 30.0) // Higher pitch for taller stacks
                };

                // Pan each column left-to-right across the stereo field
                let pan = (i as f32 / 9.0) * 2.0 - 1.0;
                let source = make_panned_sine(freq, 80, pan, vol);

                let sink = Sink::try_new(&handle).unwrap();
                sink.append(source);
                sink.sleep_until_end();
            }
        });
    }

    // =======================================================================
    // MENU SOUNDS (unchanged)
    // =======================================================================

    pub fn play_lock_delay_warning(&self) {
        // A subtle, higher-pitched ticking sound
        self.play_panned_sine(600.0, 30, 0.0, 0.3);
    }

    pub fn play_t_spin_sound(&self) {
        self.play_panned_sine(800.0, 150, -0.5, 0.6);
        self.play_panned_sine(1200.0, 200, 0.5, 0.6);
    }

    pub fn play_b2b_sound(&self) {
        self.play_panned_sine(1000.0, 100, 0.0, 0.7);
        self.play_panned_sine(1500.0, 200, 0.0, 0.7);
    }

    pub fn play_menu_move(&self) {
        self.play_sine(300.0, 30, 0.3);
    }

    pub fn play_menu_select(&self) {
        self.play_sine(600.0, 50, 0.5);
    }

    pub fn play_zone_enter(&self) {
        self.play_panned_sine(440.0, 1000, 0.0, 0.3);
        self.play_panned_sine(554.37, 1000, -0.5, 0.3);
        self.play_panned_sine(659.25, 1000, 0.5, 0.3);
    }

    pub fn play_item_spawn(&self) {
        self.play_sine(800.0, 100, 0.4);
    }

    pub fn play_item_acquire(&self) {
        self.play_sine(1200.0, 150, 0.5);
        self.play_sine(1600.0, 200, 0.5);
    }

    pub fn play_item_use(&self, item: ItemType) {
        match item {
            ItemType::Magnet => {
                self.play_panned_sine(200.0, 500, 0.0, 0.6);
            }
            ItemType::Nuke => {
                self.play_panned_sine(100.0, 800, 0.0, 0.8); // Deep rumble
            }
            ItemType::Laser => {
                self.play_panned_sine(2000.0, 300, 0.0, 0.6); // High pitched zap
            }
        }
    }

    pub fn toggle_mute(&self) -> bool {
        let mut is_muted = self.bgm_enabled.lock().unwrap();
        *is_muted = !*is_muted;
        if *is_muted {
            self.bgm_sink.lock().unwrap().play();
        } else {
            self.bgm_sink.lock().unwrap().pause();
        }
        *is_muted
    }
}
