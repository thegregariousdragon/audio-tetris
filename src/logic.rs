use rand::Rng;
use crate::settings::Difficulty;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TetrominoType {
    I, J, L, O, S, T, Z,
}

impl TetrominoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TetrominoType::I => "I-Piece",
            TetrominoType::J => "J-Piece",
            TetrominoType::L => "L-Piece",
            TetrominoType::O => "O-Piece",
            TetrominoType::S => "S-Piece",
            TetrominoType::T => "T-Piece",
            TetrominoType::Z => "Z-Piece",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tetromino {
    pub t_type: TetrominoType,
    pub x: i32,
    pub y: i32,
    pub rotation: usize,
}

impl Tetromino {
    pub fn new(t_type: TetrominoType) -> Self {
        Self {
            t_type,
            x: 3, // Spawn in middle
            y: 0,
            rotation: 0,
        }
    }

    pub fn get_blocks(&self) -> [(i32, i32); 4] {
        let blocks = match self.t_type {
            TetrominoType::I => [[(0,1), (1,1), (2,1), (3,1)], [(2,0), (2,1), (2,2), (2,3)], [(0,2), (1,2), (2,2), (3,2)], [(1,0), (1,1), (1,2), (1,3)]],
            TetrominoType::J => [[(0,0), (0,1), (1,1), (2,1)], [(1,0), (2,0), (1,1), (1,2)], [(0,1), (1,1), (2,1), (2,2)], [(1,0), (1,1), (0,2), (1,2)]],
            TetrominoType::L => [[(2,0), (0,1), (1,1), (2,1)], [(1,0), (1,1), (1,2), (2,2)], [(0,1), (1,1), (2,1), (0,2)], [(0,0), (1,0), (1,1), (1,2)]],
            TetrominoType::O => [[(1,0), (2,0), (1,1), (2,1)], [(1,0), (2,0), (1,1), (2,1)], [(1,0), (2,0), (1,1), (2,1)], [(1,0), (2,0), (1,1), (2,1)]],
            TetrominoType::S => [[(1,0), (2,0), (0,1), (1,1)], [(1,0), (1,1), (2,1), (2,2)], [(1,1), (2,1), (0,2), (1,2)], [(0,0), (0,1), (1,1), (1,2)]],
            TetrominoType::T => [[(1,0), (0,1), (1,1), (2,1)], [(1,0), (1,1), (2,1), (1,2)], [(0,1), (1,1), (2,1), (1,2)], [(1,0), (0,1), (1,1), (1,2)]],
            TetrominoType::Z => [[(0,0), (1,0), (1,1), (2,1)], [(2,0), (1,1), (2,1), (1,2)], [(0,1), (1,1), (1,2), (2,2)], [(1,0), (0,1), (1,1), (0,2)]],
        };
        
        let mut final_blocks = [(0, 0); 4];
        for (i, &(bx, by)) in blocks[self.rotation % 4].iter().enumerate() {
            final_blocks[i] = (self.x + bx, self.y + by);
        }
        final_blocks
    }
}

pub struct GameState {
    pub board: [[Option<TetrominoType>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_piece: Tetromino,
    pub score: u32,
    pub level: u32,
    pub total_lines: u32,
    pub is_game_over: bool,
    pub difficulty: Difficulty,
    pub hold_piece: Option<TetrominoType>,
    pub has_held: bool,
}

impl GameState {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Self::random_piece(),
            score: 0,
            level: 1,
            total_lines: 0,
            is_game_over: false,
            difficulty,
            hold_piece: None,
            has_held: false,
        }
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

    pub fn random_piece() -> Tetromino {
        let mut rng = rand::thread_rng();
        let types = [
            TetrominoType::I, TetrominoType::J, TetrominoType::L,
            TetrominoType::O, TetrominoType::S, TetrominoType::T, TetrominoType::Z,
        ];
        let idx = rng.gen_range(0..types.len());
        Tetromino::new(types[idx])
    }

    pub fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for &(x, y) in &piece.get_blocks() {
            if x < 0 || x >= BOARD_WIDTH as i32 || y < 0 || y >= BOARD_HEIGHT as i32 {
                return false;
            }
            if y >= 0 {
                if self.board[y as usize][x as usize].is_some() {
                    return false;
                }
            }
        }
        true
    }

    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        let mut new_piece = self.current_piece.clone();
        new_piece.x += dx;
        new_piece.y += dy;

        if self.is_valid_position(&new_piece) {
            self.current_piece = new_piece;
            true
        } else {
            false
        }
    }

    pub fn rotate_piece(&mut self) -> bool {
        let mut new_piece = self.current_piece.clone();
        new_piece.rotation = (new_piece.rotation + 1) % 4;

        if self.is_valid_position(&new_piece) {
            self.current_piece = new_piece;
            true
        } else {
            false
        }
    }

    pub fn rotate_piece_ccw(&mut self) -> bool {
        let mut new_piece = self.current_piece.clone();
        new_piece.rotation = (new_piece.rotation + 3) % 4;

        if self.is_valid_position(&new_piece) {
            self.current_piece = new_piece;
            true
        } else {
            false
        }
    }

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

    pub fn lock_piece(&mut self) -> u32 {
        for &(x, y) in &self.current_piece.get_blocks() {
            if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                self.board[y as usize][x as usize] = Some(self.current_piece.t_type);
            }
        }

        let cleared_lines = self.clear_lines();
        self.total_lines += cleared_lines;
        self.level = 1 + (self.total_lines / 10);

        self.score += match cleared_lines {
            1 => 100 * self.level,
            2 => 300 * self.level,
            3 => 500 * self.level,
            4 => 800 * self.level,
            _ => 0,
        };

        self.current_piece = Self::random_piece();
        if !self.is_valid_position(&self.current_piece) {
            self.is_game_over = true;
        }
        self.has_held = false;
        
        cleared_lines
    }

    /// Returns Some((is_swap, held_piece_name, new_piece_name)) on success, None if hold is locked out this turn.
    pub fn hold(&mut self) -> Option<(bool, &'static str, &'static str)> {
        if self.has_held {
            return None;
        }

        let current_type = self.current_piece.t_type;
        let is_swap = self.hold_piece.is_some();
        let new_piece = if let Some(held) = self.hold_piece {
            Tetromino::new(held)
        } else {
            Self::random_piece()
        };

        let new_str = new_piece.t_type.as_str();
        self.hold_piece = Some(current_type);
        self.current_piece = new_piece;
        self.has_held = true;

        Some((is_swap, current_type.as_str(), new_str))
    }

    fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0;
        let mut y = (BOARD_HEIGHT - 1) as i32;

        while y >= 0 {
            let row_full = self.board[y as usize].iter().all(|cell| cell.is_some());
            
            if row_full {
                cleared += 1;
                for move_y in (0..y).rev() {
                    self.board[(move_y + 1) as usize] = self.board[move_y as usize];
                }
                self.board[0] = [None; BOARD_WIDTH];
            } else {
                y -= 1;
            }
        }
        cleared
    }

    pub fn get_topography(&self) -> Vec<u32> {
        let mut heights = vec![0; BOARD_WIDTH];
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if self.board[y][x].is_some() {
                    heights[x] = (BOARD_HEIGHT - y) as u32;
                    break;
                }
            }
        }
        heights
    }
}
