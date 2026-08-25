use crate::logic::{BOARD_HEIGHT, BOARD_WIDTH, GameState, ItemType, TetrominoType};

pub fn render_in_game(gs: &GameState) -> (String, String) {
    let display_text = format!(
        "AUDIO TETRIS\n\
         Level {:>2}   Score {:>8}   Lines {:>3}   Zone {:>3}%\n\
         Hold: {:<10}   Current: {:<10}   Item: {:<10}\n\
         {}\n\
         {}\n\
         {}\n\
         Press Escape to pause game.",
        gs.level,
        gs.score,
        gs.total_lines,
        gs.zone_meter,
        piece_name(gs.hold_piece),
        piece_name(Some(gs.current_piece.t_type)),
        item_name(gs.inventory),
        if gs.is_in_zone {
            format!("Zone active: {} ms", gs.zone_timer_ms.max(0))
        } else {
            "Zone inactive".to_string()
        },
        render_board(gs),
        "Legend: [] active | ## locked | .. ghost | $$ item |    empty"
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

fn piece_name(piece: Option<TetrominoType>) -> &'static str {
    match piece {
        Some(TetrominoType::I) => "I",
        Some(TetrominoType::J) => "J",
        Some(TetrominoType::L) => "L",
        Some(TetrominoType::O) => "O",
        Some(TetrominoType::S) => "S",
        Some(TetrominoType::T) => "T",
        Some(TetrominoType::Z) => "Z",
        None => "None",
    }
}

fn item_name(item: Option<ItemType>) -> &'static str {
    match item {
        Some(ItemType::Magnet) => "Magnet",
        Some(ItemType::Nuke) => "Nuke",
        Some(ItemType::Laser) => "Laser",
        None => "None",
    }
}
