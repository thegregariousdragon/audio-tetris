use rodio::{OutputStream, OutputStreamHandle, Sink, Source, Decoder};
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufReader;

use crate::logic::TetrominoType;
use crate::settings::Settings;

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    bgm_sink: Arc<Mutex<Sink>>,
    current_track: Arc<Mutex<usize>>,
    is_muted: Arc<Mutex<bool>>,
    sfx_volume: Arc<Mutex<f32>>,
    bgm_volume_setting: Arc<Mutex<f32>>,
}

impl AudioEngine {
    pub fn new(settings: &Settings) -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let bgm_sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle)?));
        let current_track = Arc::new(Mutex::new(0));
        let is_muted = Arc::new(Mutex::new(false));
        let sfx_volume = Arc::new(Mutex::new(settings.sfx_volume));
        let bgm_volume_setting = Arc::new(Mutex::new(settings.bgm_volume));
        
        let engine = Self {
            _stream,
            stream_handle,
            bgm_sink,
            current_track,
            is_muted,
            sfx_volume,
            bgm_volume_setting,
        };
        
        engine.start_bgm_thread();
        Ok(engine)
    }

    pub fn set_sfx_volume(&self, vol: f32) {
        *self.sfx_volume.lock().unwrap() = vol;
    }

    pub fn set_bgm_volume(&self, vol: f32) {
        *self.bgm_volume_setting.lock().unwrap() = vol;
        let muted = *self.is_muted.lock().unwrap();
        let s = self.bgm_sink.lock().unwrap();
        if muted {
            s.set_volume(0.0);
        } else {
            s.set_volume(vol);
        }
    }

    pub fn start_bgm_thread(&self) {
        let sink = self.bgm_sink.clone();
        let track_idx = self.current_track.clone();
        let muted = self.is_muted.clone();
        let bgm_vol = self.bgm_volume_setting.clone();
        
        thread::spawn(move || {
            let tracks = ["assets/music/edm.wav",
                "assets/music/rock.wav",
                "assets/music/pop.wav"];
            
            loop {
                let is_empty = { sink.lock().unwrap().empty() };
                if is_empty {
                    let idx = *track_idx.lock().unwrap();
                    let path = tracks[idx % tracks.len()];
                    if let Ok(file) = File::open(path) {
                        let reader = BufReader::new(file);
                        if let Ok(decoder) = Decoder::new(reader) {
                            let s = sink.lock().unwrap();
                            s.append(decoder);
                            if *muted.lock().unwrap() {
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
        let mut idx = self.current_track.lock().unwrap();
        *idx += 1;
        self.bgm_sink.lock().unwrap().clear();
        
        let tracks = ["EDM", "Rock", "Pop"];
        tracks[*idx % tracks.len()].to_string()
    }

    pub fn prev_track(&self) -> String {
        let mut idx = self.current_track.lock().unwrap();
        if *idx == 0 {
            *idx = 2; // Assuming 3 tracks total
        } else {
            *idx -= 1;
        }
        self.bgm_sink.lock().unwrap().clear();
        
        let tracks = ["EDM", "Rock", "Pop"];
        tracks[*idx % tracks.len()].to_string()
    }

    pub fn toggle_mute(&self) -> bool {
        let mut muted = self.is_muted.lock().unwrap();
        *muted = !*muted;
        let s = self.bgm_sink.lock().unwrap();
        if *muted {
            s.set_volume(0.0);
        } else {
            s.set_volume(*self.bgm_volume_setting.lock().unwrap());
        }
        *muted
    }

    /// Pitch mapping: lower y (top) = higher pitch. Higher y (bottom) = lower pitch.
    pub fn play_move_sound(&self, _x: i32, y: i32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        let freq = 880.0 - (y as f32 * 30.0).clamp(0.0, 600.0);

        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(freq)
                .take_duration(Duration::from_millis(50))
                .amplify(vol); // Base 0.2 normally, but we map 0.0-1.0 setting directly
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    pub fn play_rotate_sound(&self, y: i32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        let freq = 1100.0 - (y as f32 * 30.0).clamp(0.0, 800.0);
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(freq)
                .take_duration(Duration::from_millis(50))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    pub fn play_aligned_sound(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 0.75; // Distinct high chime (slightly quieter)
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(1200.0) 
                .take_duration(Duration::from_millis(50))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    pub fn play_lock_sound(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 1.5; // Louder lock sound
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(220.0)
                .take_duration(Duration::from_millis(150))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    pub fn play_clear_sound(&self, lines: u32) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 2.0; // Clear is loudest
        thread::spawn(move || {
            let sink = Sink::try_new(&handle).unwrap();
            let freq = match lines {
                4 => 1100.0,
                3 => 880.0,
                2 => 770.0,
                _ => 660.0,
            };
            let source = rodio::source::SineWave::new(freq)
                .take_duration(Duration::from_millis(300))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

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

    pub fn play_radar_sweep(&self, heights: Vec<u32>) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap();
        thread::spawn(move || {
            for height in heights {
                let sink = Sink::try_new(&handle).unwrap();
                let freq = if height == 0 {
                    110.0 // Low hum for empty column
                } else {
                    220.0 + (height as f32 * 30.0) // Higher pitch for taller stacks
                };
                
                let source = rodio::source::SineWave::new(freq)
                    .take_duration(Duration::from_millis(80))
                    .amplify(vol);
                
                sink.append(source);
                sink.sleep_until_end();
            }
        });
    }

    pub fn play_menu_move(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 0.3;
        thread::spawn(move || {
            let sink = rodio::Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(300.0)
                .take_duration(std::time::Duration::from_millis(30))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    pub fn play_menu_select(&self) {
        let handle = self.stream_handle.clone();
        let vol = *self.sfx_volume.lock().unwrap() * 0.5;
        thread::spawn(move || {
            let sink = rodio::Sink::try_new(&handle).unwrap();
            let source = rodio::source::SineWave::new(600.0)
                .take_duration(std::time::Duration::from_millis(50))
                .amplify(vol);
            sink.append(source);
            sink.sleep_until_end();
        });
    }
}
