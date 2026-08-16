use crate::logic::{GameState, ItemType, Tetromino, TetrominoType};
use crate::settings::Difficulty;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TutorialStage {
    LateralMovement = 1,
    VerticalPitch = 2,
    RotationInspection = 3,
    HoldSlot = 4,
    LineClears = 5,
    RadarSweep = 6,
    ZoneMode = 7,
    PowerUpItems = 8,
    Graduation = 9,
}

impl TutorialStage {
    pub fn from_usize(val: usize) -> Self {
        match val {
            1 => TutorialStage::LateralMovement,
            2 => TutorialStage::VerticalPitch,
            3 => TutorialStage::RotationInspection,
            4 => TutorialStage::HoldSlot,
            5 => TutorialStage::LineClears,
            6 => TutorialStage::RadarSweep,
            7 => TutorialStage::ZoneMode,
            8 => TutorialStage::PowerUpItems,
            _ => TutorialStage::Graduation,
        }
    }

    #[allow(dead_code)]
    pub fn as_usize(&self) -> usize {
        *self as usize
    }

    pub fn title(&self) -> &'static str {
        match self {
            TutorialStage::LateralMovement => "Lesson 1 of 8: Lateral Movement & Stereo Panning",
            TutorialStage::VerticalPitch => "Lesson 2 of 8: Elevation, Soft Drop, & Landing Impact",
            TutorialStage::RotationInspection => "Lesson 3 of 8: Rotations & Piece Inspection",
            TutorialStage::HoldSlot => "Lesson 4 of 8: The Hold Slot & Piece Swapping",
            TutorialStage::LineClears => "Lesson 5 of 8: Line Clears & 4-Line Tetris",
            TutorialStage::RadarSweep => "Lesson 6 of 8: The Radar Sweep (Board Topography)",
            TutorialStage::ZoneMode => "Lesson 7 of 8: Zone Mode (Time Freeze & Combos)",
            TutorialStage::PowerUpItems => "Lesson 8 of 8: Power-Up Items (Magnet, Laser, Nuke)",
            TutorialStage::Graduation => "Tutorial Complete: Graduation & Arcade Readiness",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TutorialState {
    pub stage: usize,
    pub sub_step: usize,
    pub reached_left: bool,
    pub reached_right: bool,
    pub soft_drops: usize,
    pub rotated_cw: bool,
    pub rotated_ccw: bool,
    pub inspected: bool,
    pub held_first: bool,
    pub swapped_back: bool,
    pub tried_denied: bool,
    pub radar_scanned: bool,
    pub zone_entered: bool,
    pub item_step: usize, // 0: Magnet, 1: Laser, 2: Nuke
    pub game_state: GameState,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self::new(1)
    }
}

impl TutorialState {
    pub fn new(stage: usize) -> Self {
        let mut state = Self {
            stage: stage.clamp(1, 9),
            sub_step: 0,
            reached_left: false,
            reached_right: false,
            soft_drops: 0,
            rotated_cw: false,
            rotated_ccw: false,
            inspected: false,
            held_first: false,
            swapped_back: false,
            tried_denied: false,
            radar_scanned: false,
            zone_entered: false,
            item_step: 0,
            game_state: GameState::new(Difficulty::Easy),
        };
        state.init_stage_board();
        state
    }

    pub fn init_stage_board(&mut self) {
        self.game_state = GameState::new(Difficulty::Easy);
        // Suspend natural gravity timer during tutorial
        self.game_state.fall_timer_ms = 0;

        match self.stage {
            1 => {
                // Lateral movement
                self.reached_left = false;
                self.reached_right = false;
                self.game_state.current_piece = Tetromino::new(TetrominoType::O);
                self.game_state.current_piece.x = 4;
                self.game_state.current_piece.y = 0;
            }
            2 => {
                // Vertical pitch & drop
                self.soft_drops = 0;
                self.game_state.current_piece = Tetromino::new(TetrominoType::I);
                self.game_state.current_piece.x = 3;
                self.game_state.current_piece.y = 0;
            }
            3 => {
                // Rotations & Inspection
                self.rotated_cw = false;
                self.rotated_ccw = false;
                self.inspected = false;
                self.game_state.current_piece = Tetromino::new(TetrominoType::T);
                self.game_state.current_piece.x = 3;
                self.game_state.current_piece.y = 2;
            }
            4 => {
                // Hold Slot
                self.held_first = false;
                self.swapped_back = false;
                self.tried_denied = false;
                self.game_state.hold_piece = None;
                self.game_state.current_piece = Tetromino::new(TetrominoType::S);
                self.game_state.current_piece.x = 3;
                self.game_state.current_piece.y = 0;
            }
            5 => {
                // Line Clears
                if self.sub_step == 0 {
                    // Part 1: Row 20 has 9 blocks, open at Column 5 (col index 4)
                    for col in 0..10 {
                        if col != 4 {
                            self.game_state.board[19][col] = Some(TetrominoType::J);
                        }
                    }
                    self.game_state.current_piece = Tetromino::new(TetrominoType::O);
                    self.game_state.current_piece.x = 0;
                    self.game_state.current_piece.y = 0;
                } else {
                    // Part 2: Rows 17-20 filled in Columns 1-9 (col indices 0-8), well in Column 10 (col index 9)
                    for r in 16..20 {
                        for col in 0..9 {
                            self.game_state.board[r][col] = Some(TetrominoType::J);
                        }
                    }
                    self.game_state.current_piece = Tetromino::new(TetrominoType::I);
                    self.game_state.current_piece.x = 0;
                    self.game_state.current_piece.y = 0;
                    self.game_state.current_piece.rotation = 1; // Vertical
                }
            }
            6 => {
                // Radar Sweep
                self.radar_scanned = false;
                let heights = [2, 5, 1, 7, 3, 0, 6, 4, 8, 2];
                for (col, &h) in heights.iter().enumerate() {
                    for r in (20 - h)..20 {
                        self.game_state.board[r][col] = Some(TetrominoType::T);
                    }
                }
                self.game_state.current_piece = Tetromino::new(TetrominoType::O);
                self.game_state.current_piece.x = 4;
                self.game_state.current_piece.y = 0;
            }
            7 => {
                // Zone Mode
                self.zone_entered = false;
                self.game_state.zone_meter = 100;
                // Setup 2 rows almost full
                for col in 0..10 {
                    if col != 4 {
                        self.game_state.board[19][col] = Some(TetrominoType::L);
                        self.game_state.board[18][col] = Some(TetrominoType::L);
                    }
                }
                self.game_state.current_piece = Tetromino::new(TetrominoType::I);
                self.game_state.current_piece.x = 4;
                self.game_state.current_piece.y = 0;
                self.game_state.current_piece.rotation = 1;
            }
            8 => {
                // Power-Up Items
                self.item_step = 0;
                self.game_state.inventory = Some(ItemType::Magnet);
                // Setup fragmented board for magnet test
                self.game_state.board[17][2] = Some(TetrominoType::S);
                self.game_state.board[17][3] = Some(TetrominoType::S);
                self.game_state.board[16][6] = Some(TetrominoType::Z);
                self.game_state.board[16][7] = Some(TetrominoType::Z);
            }
            _ => {
                // Graduation
            }
        }
    }
}

pub fn render_tutorial(state: &TutorialState) -> (String, String) {
    let stage = TutorialStage::from_usize(state.stage);
    let title = stage.title();

    match stage {
        TutorialStage::LateralMovement => {
            let left_status = if state.reached_left {
                "Done"
            } else {
                "Pending"
            };
            let right_status = if state.reached_right {
                "Done"
            } else {
                "Pending"
            };
            let text = format!(
                "{}\n\nObjective:\nMove left and right across the board.\nNotice stereo audio panning from Column 1 to Column 10.\n\nLeft wall (Column 1): {}\nRight wall (Column 10): {}\n\nControls: Left/Right Arrows or A/D. Press Escape to exit tutorial.",
                title, left_status, right_status
            );
            let spoken = "Lesson 1 of 8: Lateral Movement and Stereo Panning. Use the Left and Right arrows or A and D to move your piece across the board. Notice how the sound pans from left to right. Move all the way left to Column 1, then all the way right to Column 10 to continue.".to_string();
            (text, spoken)
        }
        TutorialStage::VerticalPitch => {
            let text = format!(
                "{}\n\nObjective:\nListen to pitch decrease as piece descends from Row 1 toward Row 20.\n\nSoft drops performed: {} / 5\n\nControls:\n- Down Arrow or S: Soft drop (descending pitch)\n- Spacebar: Hard drop instantly to Row 20\n- Escape: Exit tutorial",
                title, state.soft_drops
            );
            let spoken = "Lesson 2 of 8: Elevation, Soft Drop, and Landing Impact. As a piece descends from Row 1 toward Row 20, its pitch gets lower. Press Down Arrow or S to soft drop 5 rows, then press Spacebar to hard drop the piece to Row 20 and hear the landing impact.".to_string();
            (text, spoken)
        }
        TutorialStage::RotationInspection => {
            let cw_status = if state.rotated_cw { "Done" } else { "Pending" };
            let ccw_status = if state.rotated_ccw { "Done" } else { "Pending" };
            let insp_status = if state.inspected { "Done" } else { "Pending" };
            let text = format!(
                "{}\n\nObjective:\nInspect piece shape and rotate both clockwise and counter-clockwise.\n\nPiece Inspection (V / Semicolon): {}\nClockwise Rotation (X / Period): {}\nCounter-Clockwise Rotation (Z / Comma): {}\n\nControls: V/Semicolon (Inspect), X/Period (CW), Z/Comma (CCW), Spacebar (Drop).",
                title, insp_status, cw_status, ccw_status
            );
            let spoken = "Lesson 3 of 8: Rotations and Piece Inspection. Press V or Semicolon to inspect your piece shape and see which columns it covers. Next, press X or Period to rotate clockwise, and Z or Comma to rotate counter-clockwise. Rotate twice, then press Spacebar to drop your piece.".to_string();
            (text, spoken)
        }
        TutorialStage::HoldSlot => {
            let text = format!(
                "{}\n\nObjective:\nPractice holding, swapping, and the hold-denied audio cue.\n\nControls:\n- C or Slash: Hold piece / Swap held piece\n- Spacebar: Drop piece to advance\n- Escape: Exit tutorial",
                title
            );
            let spoken = "Lesson 4 of 8: The Hold Slot and Piece Swapping. Press C or Slash to hold this piece in reserve. When the next piece appears, press C again to swap it back. Try pressing C a second time on the same turn to hear the hold-denied sound, then press Spacebar to drop and advance.".to_string();
            (text, spoken)
        }
        TutorialStage::LineClears => {
            if state.sub_step == 0 {
                let text = format!(
                    "{}\n\nPart 1: Single Line Clear\nRow 20 has 9 blocks filled with a gap in Column 5.\n\nControls:\n- Left/Right: Move to Column 5\n- Spacebar: Drop and clear row\n- Escape: Exit tutorial",
                    title
                );
                let spoken = "Lesson 5 of 8: Line Clears. Part 1: Single Line Clear. Row 20 has 9 blocks filled with a single open slot in Column 5. Move your piece into Column 5 and press Spacebar to trigger a Single Clear!".to_string();
                (text, spoken)
            } else {
                let text = format!(
                    "{}\n\nPart 2: 4-Line Tetris Fanfare\nRows 17 through 20 have an open well in Column 10.\n\nControls:\n- Left/Right: Move to Column 10\n- Spacebar: Drop for Tetris Fanfare\n- Escape: Exit tutorial",
                    title
                );
                let spoken = "Part 2: 4-Line Tetris Fanfare. Rows 17 through 20 have an open well in Column 10. Drop your Long Bar down Column 10 and press Spacebar to trigger the 4-line Tetris Fanfare!".to_string();
                (text, spoken)
            }
        }
        TutorialStage::RadarSweep => {
            let text = format!(
                "{}\n\nObjective:\nListen to the 10-tone stereo sweep across Columns 1 through 10.\n\nControls:\n- E or L: Activate Radar Sweep\n- Spacebar / Enter: Continue after scan\n- Escape: Exit tutorial",
                title
            );
            let spoken = "Lesson 6 of 8: The Radar Sweep. The board now contains stacks of different heights across Columns 1 through 10. Press E or L to activate the Radar. Listen as it sweeps 10 stereo tones from left to right. Lower tones mean short stacks, while higher tones mean tall stacks. Press E or L to scan, then press Spacebar or Enter to continue.".to_string();
            (text, spoken)
        }
        TutorialStage::ZoneMode => {
            let text = format!(
                "{}\n\nObjective:\nActivate Zone Mode to freeze gravity and clear rows.\n\nControls:\n- Q or K: Enter Zone Mode\n- Spacebar: Drop pieces\n- Escape: Exit tutorial",
                title
            );
            let spoken = "Lesson 7 of 8: Zone Mode. Your Zone meter is now 100% charged! Press Q or K to enter Zone Mode. Notice how gravity completely freezes. Clear two rows before the Zone concludes.".to_string();
            (text, spoken)
        }
        TutorialStage::PowerUpItems => {
            let item_name = match state.item_step {
                0 => "The Magnet (Pulls blocks downward to seal gaps)",
                1 => "The Laser (Incinerates bottom 2 rows)",
                _ => "The Nuke (Demolishes entire board stack)",
            };
            let text = format!(
                "{}\n\nCurrent Power-Up: {}\n\nControls:\n- Left Shift or Right Shift: Activate Power-Up\n- Escape: Exit tutorial",
                title, item_name
            );
            let spoken = match state.item_step {
                0 => "Lesson 8 of 8: Power-Up Items. You acquired The Magnet! Press Left Shift or Right Shift to pull floating blocks downward and seal empty holes.".to_string(),
                1 => "Next power-up: The Laser! Press Shift to fire a high-frequency beam that vaporizes the bottom rows.".to_string(),
                _ => "Final power-up: The Nuke! Press Shift to detonate a massive blast that clears the entire board.".to_string(),
            };
            (text, spoken)
        }
        TutorialStage::Graduation => {
            let text = format!(
                "{}\n\nCongratulations! You have completed the Audio Tetris tutorial.\n\nYou have mastered:\n- Lateral Movement & Stereo Panning (Columns 1 to 10)\n- Pitch-Mapped Elevation (Rows 1 to 20)\n- Rotations & Piece Inspection\n- Hold Queue & Swapping\n- Line Clears & 4-Line Tetris\n- 10-Tone Radar Sweep\n- Zone Mode Time Freeze\n- Power-Up Items (Magnet, Laser, Nuke)\n\n-> Press Enter to Go to Main Menu",
                title
            );
            let spoken = "Congratulations! You have completed the Audio Tetris tutorial and mastered all movement, audio cues, radar sweeps, Zone Mode, and power-up items. You can explore individual keys at any time in Keyboard Help Mode by pressing H on the Main Menu. Press Enter to start your arcade journey!".to_string();
            (text, spoken)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_titles_and_numbers() {
        for s in 1..=9 {
            let stage = TutorialStage::from_usize(s);
            assert_eq!(stage.as_usize(), s);
            assert!(!stage.title().is_empty());
        }
    }

    #[test]
    fn test_tutorial_state_initialization() {
        let state = TutorialState::new(1);
        assert_eq!(state.stage, 1);
        assert!(!state.reached_left);
        assert!(!state.reached_right);
    }

    #[test]
    fn test_render_all_stages() {
        for s in 1..=9 {
            let mut state = TutorialState::new(s);
            let (text, spoken) = render_tutorial(&state);
            assert!(!text.is_empty());
            assert!(!spoken.is_empty());

            if s == 1 {
                assert!(spoken.contains("Lesson 1 of 8"));
            } else if s == 5 {
                assert!(spoken.contains("Lesson 5 of 8"));
                state.sub_step = 1;
                let (t2, s2) = render_tutorial(&state);
                assert!(t2.contains("4-Line Tetris"));
                assert!(s2.contains("Tetris Fanfare"));
            } else if s == 9 {
                assert!(spoken.contains("Congratulations!"));
            }
        }
    }
}
