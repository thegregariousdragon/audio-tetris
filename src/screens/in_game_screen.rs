use rust_i18n::t;

use crate::logic::{BOARD_HEIGHT, BOARD_WIDTH, GameState, ItemType, TetrominoType};

pub fn render_in_game(gs: &GameState) -> (String, String) {
    let stats_line = t!(
        "in_game.ascii_stats",
        level = format!("{:>2}", gs.level),
        score = format!("{:>8}", gs.score),
        lines = format!("{:>3}", gs.total_lines),
        zone = format!("{:>3}", gs.zone_meter)
    );
    let slots_line = t!(
        "in_game.ascii_slots",
        hold = piece_name(gs.hold_piece),
        current = piece_name(Some(gs.current_piece.t_type)),
        item = item_name(gs.inventory)
    );
    let zone_status = if gs.is_in_zone {
        t!("in_game.zone_active_ms", ms = gs.zone_timer_ms.max(0)).to_string()
    } else {
        t!("in_game.zone_inactive").to_string()
    };
    let display_text = format!(
        "AUDIO TETRIS\n\
         {}\n\
         {}\n\
         {}\n\
         {}\n\
         {}\n\
         {}",
        stats_line,
        slots_line,
        zone_status,
        render_board(gs),
        t!("in_game.board_legend"),
        t!("in_game.pause_hint")
    );
    (display_text, "".to_string())
}

fn render_board(gs: &GameState) -> String {
    let mut cells = [["  "; BOARD_WIDTH]; BOARD_HEIGHT];

    for (y, row) in gs.board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if let Some(piece) = cell {
                cells[y][x] = piece_cell(*piece);
            }
            if gs.item_board[y][x].is_some() {
                cells[y][x] = "$$";
            }
        }
    }

    let mut ghost = gs.current_piece.clone();
    ghost.y = gs.get_ghost_y();
    for (x, y) in ghost.get_blocks() {
        if is_visible_cell(x, y) && cells[y as usize][x as usize] == "  " {
            cells[y as usize][x as usize] = "..";
        }
    }

    for (x, y) in gs.current_piece.get_blocks() {
        if is_visible_cell(x, y) {
            cells[y as usize][x as usize] = "[]";
        }
    }

    let mut out = String::from("+--------------------+\n");
    for row in cells {
        out.push('|');
        for cell in row {
            out.push_str(cell);
        }
        out.push_str("|\n");
    }
    out.push_str("+--------------------+");
    out
}

fn is_visible_cell(x: i32, y: i32) -> bool {
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
}

fn piece_cell(piece: TetrominoType) -> &'static str {
    match piece {
        TetrominoType::I => "II",
        TetrominoType::J => "JJ",
        TetrominoType::L => "LL",
        TetrominoType::O => "OO",
        TetrominoType::S => "SS",
        TetrominoType::T => "TT",
        TetrominoType::Z => "ZZ",
    }
}

fn piece_name(piece: Option<TetrominoType>) -> String {
    match piece {
        Some(TetrominoType::I) => "I".to_string(),
        Some(TetrominoType::J) => "J".to_string(),
        Some(TetrominoType::L) => "L".to_string(),
        Some(TetrominoType::O) => "O".to_string(),
        Some(TetrominoType::S) => "S".to_string(),
        Some(TetrominoType::T) => "T".to_string(),
        Some(TetrominoType::Z) => "Z".to_string(),
        None => t!("common.none").to_string(),
    }
}

fn item_name(item: Option<ItemType>) -> String {
    match item {
        Some(ItemType::Magnet) => t!("items.magnet_short").to_string(),
        Some(ItemType::Nuke) => t!("items.nuke_short").to_string(),
        Some(ItemType::Laser) => t!("items.laser_short").to_string(),
        None => t!("common.none").to_string(),
    }
}
