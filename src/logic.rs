use crate::settings::Difficulty;
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ItemType {
    Magnet,
    Nuke,
    Laser,
}

impl ItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemType::Magnet => "The Magnet",
            ItemType::Nuke => "The Nuke",
            ItemType::Laser => "The Laser",
        }
    }
}

impl TetrominoType {
    #[allow(dead_code)]
    pub fn as_str(&self, piece_callouts_technical: bool) -> &'static str {
        if piece_callouts_technical {
            match self {
                TetrominoType::I => "Bar",
                TetrominoType::J => "Left Angle",
                TetrominoType::L => "Right Angle",
                TetrominoType::O => "Square",
                TetrominoType::S => "Right Step",
                TetrominoType::T => "T",
                TetrominoType::Z => "Left Step",
            }
        } else {
            match self {
                TetrominoType::I => "Long bar",
                TetrominoType::J => "Left L-shape",
                TetrominoType::L => "Right L-shape",
                TetrominoType::O => "Square",
                TetrominoType::S => "Right zig-zag",
                TetrominoType::T => "T-shape",
                TetrominoType::Z => "Left zig-zag",
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tetromino {
    pub t_type: TetrominoType,
    pub x: i32,
    pub y: i32,
    pub rotation: usize,
    pub item: Option<ItemType>,
}

impl Tetromino {
    pub fn new(t_type: TetrominoType) -> Self {
        Self {
            t_type,
            x: 3,
            y: 0,
            rotation: 0,
            item: None, // Initialized later
        }
    }

    pub fn get_blocks(&self) -> [(i32, i32); 4] {
        let blocks = match self.t_type {
            TetrominoType::I => [
                [(0, 1), (1, 1), (2, 1), (3, 1)],
                [(2, 0), (2, 1), (2, 2), (2, 3)],
                [(0, 2), (1, 2), (2, 2), (3, 2)],
                [(1, 0), (1, 1), (1, 2), (1, 3)],
            ],
            TetrominoType::J => [
                [(0, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (2, 2)],
                [(1, 0), (1, 1), (0, 2), (1, 2)],
            ],
            TetrominoType::L => [
                [(2, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 2)],
                [(0, 1), (1, 1), (2, 1), (0, 2)],
                [(0, 0), (1, 0), (1, 1), (1, 2)],
            ],
            TetrominoType::O => [
                [(1, 0), (2, 0), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (2, 1)],
            ],
            TetrominoType::S => [
                [(1, 0), (2, 0), (0, 1), (1, 1)],
                [(1, 0), (1, 1), (2, 1), (2, 2)],
                [(1, 1), (2, 1), (0, 2), (1, 2)],
                [(0, 0), (0, 1), (1, 1), (1, 2)],
            ],
            TetrominoType::T => [
                [(1, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (1, 2)],
                [(1, 0), (0, 1), (1, 1), (1, 2)],
            ],
            TetrominoType::Z => [
                [(0, 0), (1, 0), (1, 1), (2, 1)],
                [(2, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 1), (1, 1), (1, 2), (2, 2)],
                [(1, 0), (0, 1), (1, 1), (0, 2)],
            ],
        };

        let mut final_blocks = [(0, 0); 4];
        for (i, &(bx, by)) in blocks[self.rotation % 4].iter().enumerate() {
            final_blocks[i] = (self.x + bx, self.y + by);
        }
        final_blocks
    }

    pub fn left_column(&self) -> i32 {
        let min_x = self
            .get_blocks()
            .iter()
            .map(|&(bx, _)| bx)
            .min()
            .unwrap_or(self.x);
        min_x + 1
    }

    pub fn right_column(&self) -> i32 {
        let max_x = self
            .get_blocks()
            .iter()
            .map(|&(bx, _)| bx)
            .max()
            .unwrap_or(self.x);
        max_x + 1
    }

    pub fn width(&self) -> i32 {
        self.right_column() - self.left_column() + 1
    }
}

pub struct LockResult {
    pub cleared_lines: u32,
    pub is_t_spin: bool,
    pub b2b_bonus: bool,
    pub combo: u32,
    pub zone_lines_cleared_this_turn: u32,
    pub zone_meter_full: bool,
}

impl Tetromino {
    pub fn get_kicks(&self, start_rot: usize, end_rot: usize) -> [(i32, i32); 5] {
        if self.t_type == TetrominoType::O {
            return [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)];
        }

        let state = (start_rot, end_rot);

        if self.t_type == TetrominoType::I {
            match state {
                (0, 1) => [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
                (1, 0) => [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
                (1, 2) => [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
                (2, 1) => [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
                (2, 3) => [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
                (3, 2) => [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
                (3, 0) => [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
                (0, 3) => [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
                _ => [(0, 0); 5],
            }
        } else {
            match state {
                (0, 1) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (1, 0) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (1, 2) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (2, 1) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (2, 3) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (3, 2) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (3, 0) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (0, 3) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                _ => [(0, 0); 5],
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub board: [[Option<TetrominoType>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub item_board: [[Option<ItemType>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_piece: Tetromino,
    pub score: u32,
    pub level: u32,
    pub total_lines: u32,
    pub is_game_over: bool,
    pub difficulty: Difficulty,
    pub hold_piece: Option<TetrominoType>,
    pub has_held: bool,
    pub bag: Vec<TetrominoType>,
    pub b2b: u32,
    pub combo: u32,
    pub last_move_was_spin: bool,
    pub lock_delay_active: bool,
    pub lock_delay_timer_ms: i32,
    pub fall_timer_ms: i32,
    pub moves_since_lock_delay: u32,
    pub is_in_zone: bool,
    pub zone_meter: u32, // 0 to 100
    pub zone_lines_cleared: u32,
    pub zone_timer_ms: i32,
    pub inventory: Option<ItemType>,
    pub item_spawned: Option<ItemType>,
    pub item_acquired: Option<ItemType>,
}

impl GameState {
    pub fn new(difficulty: Difficulty) -> Self {
        let mut gs = Self {
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            item_board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Tetromino::new(TetrominoType::I), // Temporary
            score: 0,
            level: 1,
            total_lines: 0,
            is_game_over: false,
            difficulty,
            hold_piece: None,
            has_held: false,
            bag: Vec::new(),
            b2b: 0,
            combo: 0,
            last_move_was_spin: false,
            lock_delay_active: false,
            lock_delay_timer_ms: 0,
            fall_timer_ms: 0,
            moves_since_lock_delay: 0,
            is_in_zone: false,
            zone_meter: 0,
            zone_lines_cleared: 0,
            zone_timer_ms: 0,
            inventory: None,
            item_spawned: None,
            item_acquired: None,
        };
        gs.fill_bag();
        gs.current_piece = gs.next_piece();
        gs
    }

    pub fn current_speed_ms(&self) -> i32 {
        let base_speed = match self.difficulty {
            Difficulty::Easy => 1000,
            Difficulty::Moderate => 800,
            Difficulty::Difficult => 500,
        };
        let mut speed = base_speed - ((self.level as i32 - 1) * 100);
        if speed < 100 {
            speed = 100;
        }
        speed
    }

    pub fn fill_bag(&mut self) {
        let mut types = vec![
            TetrominoType::I,
            TetrominoType::J,
            TetrominoType::L,
            TetrominoType::O,
            TetrominoType::S,
            TetrominoType::T,
            TetrominoType::Z,
        ];
        let mut rng = rand::thread_rng();
        types.shuffle(&mut rng);
        self.bag.extend(types);
    }

    pub fn next_piece(&mut self) -> Tetromino {
        if self.bag.is_empty() {
            self.fill_bag();
        }
        let t_type = self.bag.remove(0);
        let mut piece = Tetromino::new(t_type);

        let mut rng = rand::thread_rng();
        if rng.gen_range(0..100) < 15 {
            let items = [ItemType::Magnet, ItemType::Nuke, ItemType::Laser];
            piece.item = Some(*items.choose(&mut rng).unwrap());
            self.item_spawned = piece.item;
        } else {
            self.item_spawned = None;
        }

        piece
    }

    pub fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for &(x, y) in &piece.get_blocks() {
            if x < 0 || x >= BOARD_WIDTH as i32 || y < 0 || y >= BOARD_HEIGHT as i32 {
                return false;
            }
            if y >= 0 && self.board[y as usize][x as usize].is_some() {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn can_move_down(&self) -> bool {
        let mut new_piece = self.current_piece.clone();
        new_piece.y += 1;
        self.is_valid_position(&new_piece)
    }

    pub fn get_ghost_y(&self) -> i32 {
        let mut ghost = self.current_piece.clone();
        while self.is_valid_position(&Tetromino {
            y: ghost.y + 1,
            ..ghost.clone()
        }) {
            ghost.y += 1;
        }
        ghost.y
    }

    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        let mut new_piece = self.current_piece.clone();
        new_piece.x += dx;
        new_piece.y += dy;

        if self.is_valid_position(&new_piece) {
            self.current_piece = new_piece;
            if dx != 0 || dy > 0 {
                self.last_move_was_spin = false;
            }
            true
        } else {
            false
        }
    }

    pub fn rotate_piece(&mut self) -> bool {
        let start_rot = self.current_piece.rotation;
        let end_rot = (start_rot + 1) % 4;
        let kicks = self.current_piece.get_kicks(start_rot, end_rot);

        for &(kx, ky) in &kicks {
            let mut test_piece = self.current_piece.clone();
            test_piece.rotation = end_rot;
            test_piece.x += kx;
            test_piece.y += ky;

            if self.is_valid_position(&test_piece) {
                self.current_piece = test_piece;
                self.last_move_was_spin = true;
                return true;
            }
        }
        false
    }

    pub fn rotate_piece_ccw(&mut self) -> bool {
        let start_rot = self.current_piece.rotation;
        let end_rot = if start_rot == 0 { 3 } else { start_rot - 1 };
        let kicks = self.current_piece.get_kicks(start_rot, end_rot);

        for &(kx, ky) in &kicks {
            let mut test_piece = self.current_piece.clone();
            test_piece.rotation = end_rot;
            test_piece.x += kx;
            test_piece.y += ky;

            if self.is_valid_position(&test_piece) {
                self.current_piece = test_piece;
                self.last_move_was_spin = true;
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn is_perfect_fit(&self) -> bool {
        let mut ghost = self.current_piece.clone();

        while self.is_valid_position(&Tetromino {
            y: ghost.y + 1,
            ..ghost.clone()
        }) {
            ghost.y += 1;
        }

        let blocks = ghost.get_blocks();
        for &(x, y) in &blocks {
            for check_y in (y + 1)..(BOARD_HEIGHT as i32) {
                if blocks.contains(&(x, check_y)) {
                    continue;
                }
                if self.board[check_y as usize][x as usize].is_none() {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_t_spin(&self) -> bool {
        if self.current_piece.t_type != TetrominoType::T {
            return false;
        }
        if !self.last_move_was_spin {
            return false;
        }

        let mut corners = 0;
        let x = self.current_piece.x;
        let y = self.current_piece.y;

        let corner_coords = [(x, y), (x + 2, y), (x, y + 2), (x + 2, y + 2)];
        for &(cx, cy) in &corner_coords {
            if cx < 0
                || cx >= BOARD_WIDTH as i32
                || cy >= BOARD_HEIGHT as i32
                || (cy >= 0 && self.board[cy as usize][cx as usize].is_some())
            {
                corners += 1;
            }
        }
        corners >= 3
    }

    pub fn lock_piece(&mut self) -> LockResult {
        let is_t_spin = self.is_t_spin();

        for &(x, y) in &self.current_piece.get_blocks() {
            if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                self.board[y as usize][x as usize] = Some(self.current_piece.t_type);
            }
        }

        if let Some(item) = self.current_piece.item
            && let Some(&(x, y)) = self.current_piece.get_blocks().first()
            && y >= 0
            && y < BOARD_HEIGHT as i32
            && x >= 0
            && x < BOARD_WIDTH as i32
        {
            self.item_board[y as usize][x as usize] = Some(item);
        }

        let cleared_lines = self.clear_lines();
        let mut b2b_bonus = false;
        let mut zone_lines_cleared_this_turn = 0;
        let mut zone_meter_full = false;
        let mut reported_clears = 0;

        if cleared_lines > 0 {
            if self.is_in_zone {
                self.zone_lines_cleared += cleared_lines;
                zone_lines_cleared_this_turn = cleared_lines;
            } else {
                reported_clears = cleared_lines;
                self.combo += 1;
                let is_hard_clear = cleared_lines == 4 || is_t_spin;

                if is_hard_clear && self.b2b > 0 {
                    b2b_bonus = true;
                }

                if is_hard_clear {
                    self.b2b += 1;
                } else {
                    self.b2b = 0;
                }

                self.total_lines += cleared_lines;
                self.level = 1 + (self.total_lines / 10);

                let base = match cleared_lines {
                    1 => {
                        if is_t_spin {
                            800
                        } else {
                            100
                        }
                    }
                    2 => {
                        if is_t_spin {
                            1200
                        } else {
                            300
                        }
                    }
                    3 => {
                        if is_t_spin {
                            1600
                        } else {
                            500
                        }
                    }
                    4 => 800,
                    _ => 0,
                };

                let b2b_mult = if b2b_bonus { 3 } else { 2 };
                self.score += (base * b2b_mult / 2 + (50 * (self.combo - 1))) * self.level;

                let charge = match cleared_lines {
                    1 => 10,
                    2 => 20,
                    3 => 30,
                    4 => 50,
                    _ => 0,
                };
                let charge = if is_t_spin { charge + 20 } else { charge };
                let old_meter = self.zone_meter;
                self.zone_meter = (self.zone_meter + charge).min(100);
                if self.zone_meter == 100 && old_meter < 100 {
                    zone_meter_full = true;
                }
            }
        } else {
            self.combo = 0;
        }

        self.current_piece = self.next_piece();
        if !self.is_valid_position(&self.current_piece) {
            self.is_game_over = true;
        }
        self.has_held = false;
        self.lock_delay_active = false;

        LockResult {
            cleared_lines: reported_clears,
            is_t_spin,
            b2b_bonus,
            combo: self.combo,
            zone_lines_cleared_this_turn,
            zone_meter_full,
        }
    }

    pub fn start_zone(&mut self) -> bool {
        if self.is_in_zone || self.zone_meter < 100 {
            return false;
        }
        self.is_in_zone = true;
        self.zone_timer_ms = 10000; // 10 seconds for Zone mode
        true
    }

    pub fn end_zone(&mut self) -> u32 {
        if !self.is_in_zone {
            return 0;
        }
        self.is_in_zone = false;
        let lines = self.zone_lines_cleared;
        self.zone_lines_cleared = 0;
        self.zone_meter = 0;

        if lines > 0 {
            // Massive bonus for zone clears
            let multiplier = if lines >= 8 {
                3
            } else if lines >= 4 {
                2
            } else {
                1
            };
            self.score += 100 * lines * lines * multiplier * self.level;
        }
        lines
    }

    /// Returns Some((is_swap, held_piece_name, new_piece_name)) on success, None if hold is locked out this turn.
    pub fn hold(&mut self) -> Option<(bool, TetrominoType, TetrominoType)> {
        if self.has_held {
            return None;
        }

        let current_type = self.current_piece.t_type;
        let is_swap = self.hold_piece.is_some();
        let new_piece = if let Some(held) = self.hold_piece {
            Tetromino::new(held)
        } else {
            self.next_piece()
        };

        let new_type = new_piece.t_type;
        self.hold_piece = Some(current_type);
        self.current_piece = new_piece;
        self.has_held = true;

        Some((is_swap, current_type, new_type))
    }

    fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0;
        let mut y = (BOARD_HEIGHT - 1) as i32;
        self.item_acquired = None;

        while y >= 0 {
            let row_full = self.board[y as usize].iter().all(|cell| cell.is_some());

            if row_full {
                cleared += 1;

                for x in 0..BOARD_WIDTH {
                    if let Some(item) = self.item_board[y as usize][x] {
                        self.inventory = Some(item);
                        self.item_acquired = Some(item);
                    }
                }

                for move_y in (0..y).rev() {
                    self.board[(move_y + 1) as usize] = self.board[move_y as usize];
                    self.item_board[(move_y + 1) as usize] = self.item_board[move_y as usize];
                }
                self.board[0] = [None; BOARD_WIDTH];
                self.item_board[0] = [None; BOARD_WIDTH];
            } else {
                y -= 1;
            }
        }
        cleared
    }

    pub fn use_item(&mut self) -> Option<ItemType> {
        let item = self.inventory.take()?;

        match item {
            ItemType::Magnet => {
                for x in 0..BOARD_WIDTH {
                    let mut write_y = BOARD_HEIGHT as i32 - 1;
                    for read_y in (0..BOARD_HEIGHT as i32).rev() {
                        if self.board[read_y as usize][x].is_some() {
                            self.board[write_y as usize][x] = self.board[read_y as usize][x];
                            self.item_board[write_y as usize][x] =
                                self.item_board[read_y as usize][x];
                            if write_y != read_y {
                                self.board[read_y as usize][x] = None;
                                self.item_board[read_y as usize][x] = None;
                            }
                            write_y -= 1;
                        }
                    }
                }
            }
            ItemType::Nuke => {
                for y in (0..BOARD_HEIGHT - 4).rev() {
                    self.board[y + 4] = self.board[y];
                    self.item_board[y + 4] = self.item_board[y];
                }
                for y in 0..4 {
                    self.board[y] = [None; BOARD_WIDTH];
                    self.item_board[y] = [None; BOARD_WIDTH];
                }
            }
            ItemType::Laser => {
                let topo = self.get_topography();
                let mut max_h = 0;
                let mut max_col = 0;
                for (x, &h) in topo.iter().enumerate().take(BOARD_WIDTH) {
                    if h > max_h {
                        max_h = h;
                        max_col = x;
                    }
                }
                for y in 0..BOARD_HEIGHT {
                    self.board[y][max_col] = None;
                    self.item_board[y][max_col] = None;
                }
            }
        }

        Some(item)
    }

    pub fn get_topography(&self) -> Vec<u32> {
        let mut heights = vec![0; BOARD_WIDTH];
        for (x, height) in heights.iter_mut().enumerate().take(BOARD_WIDTH) {
            for y in 0..BOARD_HEIGHT {
                if self.board[y][x].is_some() {
                    *height = (BOARD_HEIGHT - y) as u32;
                    break;
                }
            }
        }
        heights
    }

    pub fn move_left(&mut self) -> bool {
        self.move_piece(-1, 0)
    }

    pub fn move_right(&mut self) -> bool {
        self.move_piece(1, 0)
    }

    pub fn move_down(&mut self) -> bool {
        self.move_piece(0, 1)
    }

    pub fn soft_drop(&mut self) -> bool {
        self.move_piece(0, 1)
    }

    pub fn rotate_cw(&mut self) -> bool {
        self.rotate_piece()
    }

    pub fn rotate_ccw(&mut self) -> bool {
        self.rotate_piece_ccw()
    }

    pub fn hard_drop(&mut self) -> LockResult {
        self.current_piece.y = self.get_ghost_y();
        self.lock_piece()
    }

    pub fn max_column_height(&self) -> u32 {
        self.get_topography().into_iter().max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tetromino_initial_blocks_and_bounds() {
        let gs = GameState::new(Difficulty::Easy);
        let piece = Tetromino::new(TetrominoType::I);
        assert_eq!(piece.x, 3);
        assert_eq!(piece.y, 0);
        assert_eq!(piece.rotation, 0);

        assert!(gs.is_valid_position(&piece));

        let mut invalid_piece = piece.clone();
        invalid_piece.x = -5;
        assert!(!gs.is_valid_position(&invalid_piece));
    }

    #[test]
    fn test_piece_movement_and_rotation_kicks() {
        let mut gs = GameState::new(Difficulty::Easy);
        gs.current_piece = Tetromino::new(TetrominoType::T);

        let initial_x = gs.current_piece.x;
        assert!(gs.move_piece(1, 0));
        assert_eq!(gs.current_piece.x, initial_x + 1);

        assert!(gs.rotate_piece());
        assert_eq!(gs.current_piece.rotation, 1);

        assert!(gs.rotate_piece_ccw());
        assert_eq!(gs.current_piece.rotation, 0);
    }

    #[test]
    fn test_7_bag_randomizer() {
        let mut gs = GameState::new(Difficulty::Easy);
        gs.bag.clear();
        gs.fill_bag();

        assert_eq!(gs.bag.len(), 7);
        let mut types = gs.bag.clone();
        types.sort_by_key(|t| format!("{:?}", t));
        types.dedup();
        assert_eq!(types.len(), 7);
    }

    #[test]
    fn test_hold_piece_mechanics() {
        let mut gs = GameState::new(Difficulty::Easy);
        let initial_type = gs.current_piece.t_type;

        let result = gs.hold();
        assert!(result.is_some());
        let (is_swap, held, _new_p) = result.unwrap();
        assert!(!is_swap);
        assert_eq!(held, initial_type);
        assert_eq!(gs.hold_piece, Some(initial_type));
        assert!(gs.has_held);

        assert!(gs.hold().is_none());
    }

    #[test]
    fn test_line_clear_and_row_shifting() {
        let mut gs = GameState::new(Difficulty::Easy);

        let bottom_y = BOARD_HEIGHT - 1;
        for x in 0..BOARD_WIDTH {
            gs.board[bottom_y][x] = Some(TetrominoType::I);
        }

        let initial_lines = gs.total_lines;
        let lock_res = gs.lock_piece();
        assert!(lock_res.cleared_lines >= 1);
        assert_eq!(gs.total_lines, initial_lines + lock_res.cleared_lines);
    }

    #[test]
    fn test_t_spin_detection() {
        let mut gs = GameState::new(Difficulty::Easy);
        gs.current_piece = Tetromino::new(TetrominoType::T);
        gs.current_piece.x = 3;
        gs.current_piece.y = 17;
        gs.last_move_was_spin = true;

        let x = gs.current_piece.x as usize;
        let y = gs.current_piece.y as usize;
        gs.board[y][x] = Some(TetrominoType::O);
        gs.board[y][x + 2] = Some(TetrominoType::O);
        gs.board[y + 2][x] = Some(TetrominoType::O);

        assert!(gs.is_t_spin());
    }

    #[test]
    fn test_zone_mode_lifecycle() {
        let mut gs = GameState::new(Difficulty::Easy);
        assert!(!gs.start_zone());

        gs.zone_meter = 100;
        assert!(gs.start_zone());
        assert!(gs.is_in_zone);
        assert_eq!(gs.zone_timer_ms, 10000);

        gs.zone_lines_cleared = 4;
        let lines = gs.end_zone();
        assert_eq!(lines, 4);
        assert!(!gs.is_in_zone);
        assert_eq!(gs.zone_meter, 0);
    }

    #[test]
    fn test_powerup_items() {
        let mut gs = GameState::new(Difficulty::Easy);

        gs.inventory = Some(ItemType::Nuke);
        for x in 0..BOARD_WIDTH {
            gs.board[BOARD_HEIGHT - 1][x] = Some(TetrominoType::I);
        }
        let used = gs.use_item();
        assert_eq!(used, Some(ItemType::Nuke));

        gs.inventory = Some(ItemType::Magnet);
        gs.board[10][0] = Some(TetrominoType::O);
        gs.use_item();
        assert!(gs.board[BOARD_HEIGHT - 1][0].is_some());

        gs.inventory = Some(ItemType::Laser);
        gs.board[5][2] = Some(TetrominoType::I);
        gs.use_item();
        assert!(gs.board[5][2].is_none());
    }

    #[test]
    fn test_topography_calculation() {
        let mut gs = GameState::new(Difficulty::Easy);
        assert_eq!(gs.get_topography(), vec![0; BOARD_WIDTH]);

        gs.board[BOARD_HEIGHT - 5][0] = Some(TetrominoType::I);
        let topo = gs.get_topography();
        assert_eq!(topo[0], 5);
        assert_eq!(topo[1], 0);
    }

    #[test]
    fn test_tetromino_column_calculation() {
        let mut i_piece = Tetromino::new(TetrominoType::I);
        i_piece.x = 0;
        assert_eq!(i_piece.left_column(), 1);
        assert_eq!(i_piece.right_column(), 4);

        i_piece.x = 6;
        assert_eq!(i_piece.left_column(), 7);
        assert_eq!(i_piece.right_column(), 10);

        let mut t_piece = Tetromino::new(TetrominoType::T);
        t_piece.x = 0;
        assert_eq!(t_piece.left_column(), 1);
        assert_eq!(t_piece.right_column(), 3);

        t_piece.x = 7;
        assert_eq!(t_piece.left_column(), 8);
        assert_eq!(t_piece.right_column(), 10);
    }

    #[test]
    fn test_tetromino_width() {
        let i_piece = Tetromino::new(TetrominoType::I);
        assert_eq!(i_piece.width(), 4);

        let t_piece = Tetromino::new(TetrominoType::T);
        assert_eq!(t_piece.width(), 3);

        let o_piece = Tetromino::new(TetrominoType::O);
        assert_eq!(o_piece.width(), 2);
    }
}
