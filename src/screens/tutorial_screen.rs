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

    pub fn title(&self) -> String {
        match self {
            TutorialStage::LateralMovement => rust_i18n::t!("tutorial.stage_1_title").to_string(),
            TutorialStage::VerticalPitch => rust_i18n::t!("tutorial.stage_2_title").to_string(),
            TutorialStage::RotationInspection => {
                rust_i18n::t!("tutorial.stage_3_title").to_string()
            }
            TutorialStage::HoldSlot => rust_i18n::t!("tutorial.stage_4_title").to_string(),
            TutorialStage::LineClears => rust_i18n::t!("tutorial.stage_5_title").to_string(),
            TutorialStage::RadarSweep => rust_i18n::t!("tutorial.stage_6_title").to_string(),
            TutorialStage::ZoneMode => rust_i18n::t!("tutorial.stage_7_title").to_string(),
            TutorialStage::PowerUpItems => rust_i18n::t!("tutorial.stage_8_title").to_string(),
            TutorialStage::Graduation => rust_i18n::t!("tutorial.stage_9_title").to_string(),
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
    let done_str = rust_i18n::t!("tutorial.status_done");
    let pending_str = rust_i18n::t!("tutorial.status_pending");

    match stage {
        TutorialStage::LateralMovement => {
            let left_status = if state.reached_left {
                &done_str
            } else {
                &pending_str
            };
            let right_status = if state.reached_right {
                &done_str
            } else {
                &pending_str
            };
            let text = rust_i18n::t!(
                "tutorial.stage_1_text",
                title = &title,
                left_status = left_status,
                right_status = right_status
            )
            .to_string();
            let spoken = rust_i18n::t!("tutorial.stage_1_spoken").to_string();
            (text, spoken)
        }
        TutorialStage::VerticalPitch => {
            let text = rust_i18n::t!(
                "tutorial.stage_2_text",
                title = &title,
                drops = state.soft_drops.to_string()
            )
            .to_string();
            let spoken = rust_i18n::t!("tutorial.stage_2_spoken").to_string();
            (text, spoken)
        }
        TutorialStage::RotationInspection => {
            let cw_status = if state.rotated_cw {
                &done_str
            } else {
                &pending_str
            };
            let ccw_status = if state.rotated_ccw {
                &done_str
            } else {
                &pending_str
            };
            let insp_status = if state.inspected {
                &done_str
            } else {
                &pending_str
            };
            let text = rust_i18n::t!(
                "tutorial.stage_3_text",
                title = &title,
                insp = insp_status,
                cw = cw_status,
                ccw = ccw_status
            )
            .to_string();
            let spoken = rust_i18n::t!("tutorial.stage_3_spoken").to_string();
            (text, spoken)
        }
        TutorialStage::HoldSlot => {
            let text = rust_i18n::t!("tutorial.stage_4_text", title = &title).to_string();
            let spoken = rust_i18n::t!("tutorial.stage_4_spoken").to_string();
            (text, spoken)
        }
        TutorialStage::LineClears => {
            if state.sub_step == 0 {
                let text = rust_i18n::t!("tutorial.stage_5_p1_text", title = &title).to_string();
                let spoken = rust_i18n::t!("tutorial.stage_5_p1_spoken").to_string();
                (text, spoken)
            } else {
                let text = rust_i18n::t!("tutorial.stage_5_p2_text", title = &title).to_string();
                let spoken = rust_i18n::t!("tutorial.stage_5_p2_spoken").to_string();
                (text, spoken)
            }
        }
        TutorialStage::RadarSweep => {
            let text = rust_i18n::t!("tutorial.stage_6_text", title = &title).to_string();
            let spoken = rust_i18n::t!("tutorial.stage_6_spoken").to_string();
            (text, spoken)
        }
        TutorialStage::ZoneMode => {
            let text = rust_i18n::t!("tutorial.stage_7_text", title = &title).to_string();
            let spoken = rust_i18n::t!("tutorial.stage_7_spoken").to_string();
            (text, spoken)
        }
        TutorialStage::PowerUpItems => {
            let item_name = match state.item_step {
                0 => rust_i18n::t!("tutorial.stage_8_item_0_name"),
                1 => rust_i18n::t!("tutorial.stage_8_item_1_name"),
                _ => rust_i18n::t!("tutorial.stage_8_item_2_name"),
            };
            let text = rust_i18n::t!(
                "tutorial.stage_8_text",
                title = &title,
                item_name = &item_name
            )
            .to_string();
            let spoken = match state.item_step {
                0 => rust_i18n::t!("tutorial.stage_8_spoken_0").to_string(),
                1 => rust_i18n::t!("tutorial.stage_8_spoken_1").to_string(),
                _ => rust_i18n::t!("tutorial.stage_8_spoken_2").to_string(),
            };
            (text, spoken)
        }
        TutorialStage::Graduation => {
            let text = rust_i18n::t!("tutorial.stage_9_text", title = &title).to_string();
            let spoken = rust_i18n::t!("tutorial.stage_9_spoken").to_string();
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
