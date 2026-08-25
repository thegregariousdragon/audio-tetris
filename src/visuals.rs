use rust_i18n::t;
use wxdragon::prelude::*;

use crate::logic::{BOARD_HEIGHT, BOARD_WIDTH, GameState, ItemType, TetrominoType};
use crate::screens::{AppScreen, main_menu, pause_menu};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const STATUS_VISIBLE_SECS: u64 = 4;

pub struct VisualAssets {
    menu_background: Option<Bitmap>,
    menu_background_light: Option<Bitmap>,
    gameplay_background: Option<Bitmap>,
    gameplay_background_light: Option<Bitmap>,
}

#[derive(Clone)]
pub struct VisualStatus {
    pub text: String,
    pub created: Instant,
}

impl VisualStatus {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            created: Instant::now(),
        }
    }
}

pub struct VisualRenderState<'a> {
    pub game_in_progress: bool,
    pub is_dark_mode: bool,
    pub display_text: &'a str,
    pub visual_status: Option<&'a VisualStatus>,
}

impl VisualAssets {
    pub fn load() -> Self {
        Self {
            menu_background: load_png_bitmap("assets/images/main_menu.png"),
            menu_background_light: load_png_bitmap_brightened("assets/images/main_menu.png"),
            gameplay_background: load_png_bitmap("assets/images/gameplay_background.png"),
            gameplay_background_light: load_png_bitmap_brightened(
                "assets/images/gameplay_background.png",
            ),
        }
    }
}

fn load_png_bitmap(path: &str) -> Option<Bitmap> {
    let reader = image::ImageReader::open(path).ok()?;
    let image = reader.decode().ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    Bitmap::from_rgba(image.as_raw(), width, height)
}

fn load_png_bitmap_brightened(path: &str) -> Option<Bitmap> {
    let reader = image::ImageReader::open(path).ok()?;
    let mut image = reader.decode().ok()?.to_rgba8();
    for pixel in image.pixels_mut() {
        let red = pixel.0[0] as u16;
        let green = pixel.0[1] as u16;
        let blue = pixel.0[2] as u16;
        pixel.0[0] = (red + ((255 - red) * 18 / 100)).min(255) as u8;
        pixel.0[1] = (green + ((255 - green) * 34 / 100)).min(255) as u8;
        pixel.0[2] = (blue + ((255 - blue) * 62 / 100)).min(255) as u8;
    }
    let (width, height) = image.dimensions();
    Bitmap::from_rgba(image.as_raw(), width, height)
}

#[derive(Clone, Copy)]
struct VisualPalette {
    is_dark: bool,
    title: Colour,
    text: Colour,
    selected: Colour,
    shadow: Colour,
    panel_bg: Colour,
    panel_border: Colour,
    board_bg: Colour,
    grid: Colour,
    ghost: Colour,
}

impl VisualPalette {
    fn for_theme(is_dark_mode: bool) -> Self {
        if is_dark_mode {
            Self {
                is_dark: true,
                title: Colour::rgb(255, 232, 79),
                text: Colour::rgb(245, 245, 255),
                selected: Colour::rgb(0, 255, 230),
                shadow: Colour::rgb(0, 0, 0),
                panel_bg: Colour::rgb(6, 10, 24),
                panel_border: Colour::rgb(0, 170, 190),
                board_bg: Colour::rgb(8, 12, 28),
                grid: Colour::rgb(20, 45, 68),
                ghost: Colour::rgb(75, 88, 118),
            }
        } else {
            Self {
                is_dark: false,
                title: Colour::rgb(4, 19, 38),
                text: Colour::rgb(4, 19, 38),
                selected: Colour::rgb(0, 48, 76),
                shadow: Colour::rgb(4, 19, 38),
                panel_bg: Colour::rgb(178, 218, 242),
                panel_border: Colour::rgb(30, 91, 130),
                board_bg: Colour::rgb(198, 229, 248),
                grid: Colour::rgb(74, 127, 164),
                ghost: Colour::rgb(90, 123, 150),
            }
        }
    }
}

pub fn draw_app(
    dc: &AutoBufferedPaintDC,
    assets: &VisualAssets,
    screen: &AppScreen,
    game_state: &GameState,
    state: VisualRenderState<'_>,
) {
    let (width, height) = dc.get_size();
    let is_dark_mode = state.is_dark_mode;
    let palette = VisualPalette::for_theme(is_dark_mode);
    let background = match screen {
        AppScreen::InGame | AppScreen::PauseMenu { .. } if is_dark_mode => {
            assets.gameplay_background.as_ref()
        }
        AppScreen::InGame | AppScreen::PauseMenu { .. } => {
            assets.gameplay_background_light.as_ref()
        }
        _ if is_dark_mode => assets.menu_background.as_ref(),
        _ => assets.menu_background_light.as_ref(),
    };
    draw_background(dc, width, height, background, &palette, is_dark_mode);

    match screen {
        AppScreen::MainMenu { selection } => {
            draw_main_menu(
                dc,
                width,
                height,
                *selection,
                state.game_in_progress,
                &palette,
            );
        }
        AppScreen::InGame => {
            draw_game(dc, width, height, game_state, &palette, is_dark_mode);
        }
        AppScreen::PauseMenu { .. } => {
            draw_game(dc, width, height, game_state, &palette, is_dark_mode);
            draw_pause_menu(dc, width, height, screen, &palette, is_dark_mode);
        }
        AppScreen::Tutorial { stage } => {
            draw_tutorial_screen(
                dc,
                width,
                height,
                *stage,
                state.display_text,
                &palette,
                is_dark_mode,
            );
        }
        _ => {
            draw_text_screen(
                dc,
                width,
                height,
                state.display_text,
                &palette,
                is_dark_mode,
            );
        }
    }

    if let Some(status) = state.visual_status
        && status.created.elapsed() <= Duration::from_secs(STATUS_VISIBLE_SECS)
    {
        draw_status_banner(dc, width, height, screen, &status.text, &palette);
    }
}

fn draw_background(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    background: Option<&Bitmap>,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let clear = if is_dark_mode {
        Colour::rgb(7, 8, 20)
    } else {
        Colour::rgb(235, 242, 250)
    };
    dc.set_background(clear);
    dc.set_background_mode(BackgroundMode::Solid);
    dc.clear();
    dc.set_background_mode(BackgroundMode::Transparent);

    if let Some(bitmap) = background
        && bitmap.is_ok()
    {
        let x = (width - bitmap.get_width()) / 2;
        let y = (height - bitmap.get_height()) / 2;
        dc.draw_bitmap(bitmap, x, y, false);
    }

    let floor_grid = if is_dark_mode {
        Colour::rgb(20, 92, 132)
    } else {
        Colour::rgb(75, 130, 164)
    };
    dc.set_pen(floor_grid, 1, PenStyle::Solid);
    let horizon = (height as f32 * 0.76) as i32;
    let mut x = -width;
    while x < width * 2 {
        dc.draw_line(width / 2, horizon, x, height);
        x += 80;
    }
    let mut y = horizon;
    while y < height {
        dc.draw_line(0, y, width, y);
        y += ((y - horizon) / 4 + 12).max(12);
    }

    if background.is_none() {
        let accents = [
            palette.selected,
            Colour::rgb(122, 38, 112),
            Colour::rgb(104, 92, 0),
            Colour::rgb(28, 96, 38),
        ];
        for (i, colour) in accents.iter().enumerate() {
            let block = 18;
            let base_x = if i % 2 == 0 { 32 } else { width - 110 };
            let base_y = 60 + (i as i32 * 92);
            dc.set_pen(Colour::rgb(0, 0, 0), 2, PenStyle::Solid);
            dc.set_brush(*colour, BrushStyle::Solid);
            dc.draw_rectangle(base_x, base_y, block, block);
            dc.draw_rectangle(base_x + block, base_y, block, block);
            dc.draw_rectangle(base_x, base_y + block, block, block);
        }
    }
}

fn draw_main_menu(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    selection: usize,
    game_in_progress: bool,
    palette: &VisualPalette,
) {
    let title_font = Font::new_with_details(
        30,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    );
    let menu_font = Font::new_with_details(
        18,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    );

    if let Some(font) = title_font.as_ref() {
        dc.set_font(font);
    }
    draw_shadowed_centered_text(
        dc,
        "AUDIO TETRIS",
        width,
        height / 7,
        palette.title,
        palette,
    );

    if let Some(font) = menu_font.as_ref() {
        dc.set_font(font);
    }
    let options = main_menu::get_main_menu_options(game_in_progress);
    let selected = selection.min(options.len().saturating_sub(1));
    let line_h = 32;
    let start_y = (height / 2) - ((options.len() as i32 * line_h) / 2);
    for (i, option) in options.iter().enumerate() {
        let y = start_y + i as i32 * line_h;
        let colour = if i == selected {
            palette.selected
        } else {
            palette.text
        };
        let text_x = draw_shadowed_centered_text(dc, option, width, y, colour, palette);
        if i == selected {
            draw_selector_piece(dc, text_x - 70, y + 12, 12, palette);
        }
    }
}

fn draw_game(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    gs: &GameState,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let hud_font = Font::new_with_details(
        15,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    );
    if let Some(font) = hud_font.as_ref() {
        dc.set_font(font);
    }

    let top_margin = 64;
    let bottom_margin = 42;
    let side_space = 520;
    let cell = ((height - top_margin - bottom_margin) / BOARD_HEIGHT as i32)
        .min((width - side_space) / BOARD_WIDTH as i32)
        .clamp(22, 44);
    let board_w = cell * BOARD_WIDTH as i32;
    let board_h = cell * BOARD_HEIGHT as i32;
    let board_x = (width - board_w) / 2;
    let board_y = top_margin + ((height - top_margin - bottom_margin - board_h) / 2).max(0);

    let left_x = 24;
    let right_x = board_x + board_w + 36;
    draw_hud_panel(
        dc,
        left_x,
        top_margin,
        board_x - left_x - 32,
        168,
        gs,
        palette,
    );
    draw_info_panel(
        dc,
        right_x,
        top_margin,
        width - right_x - 24,
        168,
        gs,
        palette,
    );

    dc.set_pen(palette.panel_border, 2, PenStyle::Solid);
    dc.set_brush(palette.board_bg, BrushStyle::Solid);
    dc.draw_rectangle(board_x - 4, board_y - 4, board_w + 8, board_h + 8);

    draw_board_cells(dc, board_x, board_y, cell, gs, palette, is_dark_mode);
    draw_active_piece_callout(dc, board_x, board_y, cell, gs, palette, is_dark_mode);
}

fn draw_tutorial_screen(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    stage: usize,
    display_text: &str,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let title_font = Font::new_with_details(
        19,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    );
    let body_font = Font::new_with_details(
        14,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    );

    let content_top = 92;
    let demo_cell = (height / 28).clamp(16, 26);
    let demo_w = demo_cell * BOARD_WIDTH as i32;
    let demo_h = demo_cell * BOARD_HEIGHT as i32;
    let demo_x = 44;
    let demo_y = ((height - demo_h) / 2).max(76);

    if width >= 980 {
        draw_demo_board(dc, demo_x, demo_y, demo_cell, stage, palette, is_dark_mode);
    }

    let text_left = if width >= 980 {
        demo_x + demo_w + 52
    } else {
        40
    };
    let text_width = (width - text_left - 44).max(300);
    let mut y = content_top;

    let mut lines = display_text.lines();
    if let Some(title) = lines.next() {
        if let Some(font) = title_font.as_ref() {
            dc.set_font(font);
        }
        dc.set_text_foreground(palette.title);
        draw_shadowed_text(dc, title.trim(), text_left, y, palette.title, palette);
        y += 48;
    }

    if let Some(font) = body_font.as_ref() {
        dc.set_font(font);
    }
    for raw in lines {
        let line = raw.trim();
        if line.is_empty() {
            y += 18;
            continue;
        }
        let colour = palette.text;
        for wrapped in wrap_for_width(dc, line, text_width) {
            if y > height - 130 {
                break;
            }
            draw_shadowed_text(dc, &wrapped, text_left, y, colour, palette);
            y += 29;
        }
    }
}

fn draw_pause_menu(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    screen: &AppScreen,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let selection = match screen {
        AppScreen::PauseMenu { selection } => *selection,
        _ => 0,
    };
    let options = pause_menu::get_pause_menu_options();
    let selected = selection.min(options.len().saturating_sub(1));

    if let Some(title_font) = Font::new_with_details(
        32,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    ) {
        dc.set_font(&title_font);
    }
    draw_shadowed_centered_text(
        dc,
        &t!("visuals.paused"),
        width,
        height / 4,
        palette.title,
        palette,
    );

    if let Some(menu_font) = Font::new_with_details(
        19,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    ) {
        dc.set_font(&menu_font);
    }

    let line_h = 34;
    let start_y = height / 3;
    for (i, option) in options.iter().enumerate() {
        let colour = if i == selected {
            palette.selected
        } else {
            palette.text
        };
        let y = start_y + i as i32 * line_h;
        let text_x = draw_shadowed_centered_text(dc, option, width, y, colour, palette);
        if i == selected {
            draw_selector_piece(dc, text_x - 72, y + 13, 12, palette);
        }
    }
    let _ = is_dark_mode;
}

fn draw_text_screen(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    display_text: &str,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    if display_text.trim().is_empty() {
        return;
    }

    if let Some(font) = Font::new_with_details(
        18,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    ) {
        dc.set_font(&font);
    }

    let max_text_width = (width - 96).clamp(280, 980);
    let mut visual_lines = Vec::new();
    for line in display_text.lines() {
        if line.trim().is_empty() {
            visual_lines.push(String::new());
        } else {
            visual_lines.extend(wrap_for_width(dc, line.trim_end(), max_text_width));
        }
    }

    let line_h = 31;
    let text_h = visual_lines.len() as i32 * line_h;
    let start_y = ((height - text_h) / 2).max(60);

    for (i, line) in visual_lines.iter().enumerate() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let selected_line = trimmed.trim_start().starts_with("->");
        let visual_text = if selected_line {
            trimmed.trim_start().trim_start_matches("->").trim_start()
        } else {
            trimmed
        };
        let colour = if selected_line {
            palette.selected
        } else if i == 0 {
            palette.title
        } else {
            palette.text
        };
        let text_x = draw_shadowed_centered_text(
            dc,
            visual_text,
            width,
            start_y + i as i32 * line_h,
            colour,
            palette,
        );
        if selected_line {
            draw_selector_piece(
                dc,
                text_x - 72,
                start_y + i as i32 * line_h + 13,
                12,
                palette,
            );
        }
    }
    let _ = is_dark_mode;
}

fn draw_status_banner(
    dc: &AutoBufferedPaintDC,
    width: i32,
    height: i32,
    screen: &AppScreen,
    text: &str,
    palette: &VisualPalette,
) {
    let text = compact_single_line(text);
    if text.is_empty() {
        return;
    }

    if let Some(font) = Font::new_with_details(
        16,
        FontFamily::Modern.as_i32(),
        FontStyle::Normal.as_i32(),
        FontWeight::Bold.as_i32(),
        false,
        "Consolas",
    ) {
        dc.set_font(&font);
    }

    let in_game = matches!(screen, AppScreen::InGame | AppScreen::PauseMenu { .. });
    let max_chars = if in_game {
        42
    } else if width > 1100 {
        96
    } else {
        66
    };
    let text = ellipsize(&text, max_chars);
    let (text_w, text_h) = dc.get_text_extent(&text);
    let padding_x = 18;
    let padding_y = 10;
    let mut box_w = (text_w + padding_x * 2).min(width - 48);
    let box_h = text_h + padding_y * 2;
    let (x, y) = if in_game {
        let top_margin = 64;
        let bottom_margin = 42;
        let side_space = 520;
        let cell = ((height - top_margin - bottom_margin) / BOARD_HEIGHT as i32)
            .min((width - side_space) / BOARD_WIDTH as i32)
            .clamp(22, 44);
        let board_w = cell * BOARD_WIDTH as i32;
        let board_x = (width - board_w) / 2;
        let side_w = (board_x - 56).max(220);
        box_w = box_w.min(side_w);
        (24, 282.min((height - box_h - 24).max(12)))
    } else {
        ((width - box_w) / 2, (height - box_h - 34).max(12))
    };

    dc.set_pen(palette.panel_border, 1, PenStyle::Solid);
    dc.set_brush(palette.panel_bg, BrushStyle::Solid);
    dc.draw_rectangle(x, y, box_w, box_h);
    dc.set_text_foreground(palette.title);
    dc.draw_text(&text, x + padding_x, y + padding_y);
}

fn draw_hud_panel(
    dc: &AutoBufferedPaintDC,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    gs: &GameState,
    palette: &VisualPalette,
) {
    draw_panel_backing(dc, x, y, width, height, palette);
    dc.set_text_foreground(palette.title);
    dc.draw_text(&t!("visuals.status_header"), x + 16, y + 12);
    dc.set_text_foreground(palette.text);
    dc.draw_text(&t!("visuals.level", level = gs.level), x + 16, y + 46);
    dc.draw_text(&t!("visuals.score", score = gs.score), x + 16, y + 74);
    dc.draw_text(
        &t!("visuals.lines", lines = gs.total_lines),
        x + 16,
        y + 102,
    );
    dc.draw_text(&t!("visuals.zone", zone = gs.zone_meter), x + 16, y + 130);
}

fn draw_info_panel(
    dc: &AutoBufferedPaintDC,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    gs: &GameState,
    palette: &VisualPalette,
) {
    draw_panel_backing(dc, x, y, width, height, palette);
    dc.set_text_foreground(palette.selected);
    dc.draw_text(&t!("visuals.piece_header"), x + 16, y + 12);
    dc.set_text_foreground(palette.text);
    dc.draw_text(
        &t!(
            "visuals.current",
            piece = piece_name(Some(gs.current_piece.t_type))
        ),
        x + 16,
        y + 46,
    );
    dc.draw_text(
        &t!("visuals.hold", piece = piece_name(gs.hold_piece)),
        x + 16,
        y + 74,
    );
    dc.draw_text(
        &t!("visuals.item", item = item_name(gs.inventory)),
        x + 16,
        y + 102,
    );
    dc.draw_text(&t!("visuals.screen_reader_hint"), x + 16, y + 130);
}

fn draw_panel_backing(
    dc: &AutoBufferedPaintDC,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    palette: &VisualPalette,
) {
    if width < 120 {
        return;
    }
    dc.set_pen(palette.panel_border, 1, PenStyle::Solid);
    dc.set_brush(palette.panel_bg, BrushStyle::Solid);
    dc.draw_rectangle(x, y, width, height);
}

fn draw_board_cells(
    dc: &AutoBufferedPaintDC,
    board_x: i32,
    board_y: i32,
    cell: i32,
    gs: &GameState,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let mut visual = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
    for (y, row) in gs.board.iter().enumerate() {
        for (x, piece) in row.iter().enumerate() {
            visual[y][x] = piece.map(CellVisual::Locked);
        }
    }

    let mut ghost = gs.current_piece.clone();
    ghost.y = gs.get_ghost_y();
    for (x, y) in ghost.get_blocks() {
        if visible(x, y) && visual[y as usize][x as usize].is_none() {
            visual[y as usize][x as usize] = Some(CellVisual::Ghost);
        }
    }

    for (x, y) in gs.current_piece.get_blocks() {
        if visible(x, y) {
            visual[y as usize][x as usize] = Some(CellVisual::Active(gs.current_piece.t_type));
        }
    }

    for (y, row) in visual.iter().enumerate() {
        for (x, cell_visual) in row.iter().enumerate() {
            let px = board_x + x as i32 * cell;
            let py = board_y + y as i32 * cell;
            dc.set_pen(palette.grid, 1, PenStyle::Solid);
            dc.set_brush(palette.board_bg, BrushStyle::Solid);
            dc.draw_rectangle(px, py, cell, cell);

            if let Some(cell_visual) = cell_visual {
                let colour = match *cell_visual {
                    CellVisual::Locked(piece) | CellVisual::Active(piece) => piece_colour(piece),
                    CellVisual::Ghost => palette.ghost,
                };
                dc.set_pen(palette.shadow, 1, PenStyle::Solid);
                dc.set_brush(colour, BrushStyle::Solid);
                dc.draw_rectangle(px + 2, py + 2, cell - 4, cell - 4);
                if matches!(*cell_visual, CellVisual::Active(_)) {
                    dc.set_pen(pulse_colour(is_dark_mode), 3, PenStyle::Solid);
                    dc.set_brush(Colour::rgb(0, 0, 0), BrushStyle::Transparent);
                    dc.draw_rectangle(px + 3, py + 3, cell - 6, cell - 6);
                }
            }

            if let Some(item) = gs.item_board[y][x] {
                dc.set_text_foreground(palette.text);
                dc.draw_text(item_symbol(item), px + cell / 3, py + cell / 5);
            }
        }
    }
}

fn draw_active_piece_callout(
    dc: &AutoBufferedPaintDC,
    board_x: i32,
    board_y: i32,
    cell: i32,
    gs: &GameState,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let blocks = gs.current_piece.get_blocks();
    let visible_blocks: Vec<(i32, i32)> = blocks
        .iter()
        .copied()
        .filter(|&(x, y)| visible(x, y))
        .collect();
    if visible_blocks.is_empty() {
        return;
    }

    let min_x = visible_blocks.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x = visible_blocks.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y = visible_blocks.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y = visible_blocks.iter().map(|(_, y)| *y).max().unwrap_or(0);
    let x = board_x + min_x * cell;
    let y = board_y + min_y * cell;
    let w = (max_x - min_x + 1) * cell;
    let h = (max_y - min_y + 1) * cell;

    dc.set_pen(pulse_colour(is_dark_mode), 3, PenStyle::Solid);
    dc.set_brush(Colour::rgb(0, 0, 0), BrushStyle::Transparent);
    dc.draw_rectangle(x - 5, y - 5, w + 10, h + 10);

    let center_x = x + w / 2;
    dc.set_pen(palette.selected, 1, PenStyle::Dot);
    dc.draw_line(
        center_x,
        board_y,
        center_x,
        board_y + BOARD_HEIGHT as i32 * cell,
    );
}

#[derive(Clone, Copy)]
enum CellVisual {
    Locked(TetrominoType),
    Active(TetrominoType),
    Ghost,
}

fn draw_shadowed_centered_text(
    dc: &AutoBufferedPaintDC,
    text: &str,
    width: i32,
    y: i32,
    colour: Colour,
    palette: &VisualPalette,
) -> i32 {
    let (text_w, _) = dc.get_text_extent(text);
    let x = (width - text_w) / 2;
    draw_shadowed_text(dc, text, x, y, colour, palette);
    x
}

fn draw_shadowed_text(
    dc: &AutoBufferedPaintDC,
    text: &str,
    x: i32,
    y: i32,
    colour: Colour,
    palette: &VisualPalette,
) {
    if palette.is_dark {
        dc.set_text_foreground(palette.shadow);
        dc.draw_text(text, x + 3, y + 3);
        dc.draw_text(text, x + 2, y + 2);
    }
    dc.set_text_foreground(colour);
    dc.draw_text(text, x, y);
}

fn draw_selector_piece(
    dc: &AutoBufferedPaintDC,
    x: i32,
    y: i32,
    block: i32,
    palette: &VisualPalette,
) {
    let colour = pulse_piece_colour(palette);
    let cells = [(0, 0), (1, 0), (2, 0), (3, 0)];
    let outline = if palette.is_dark {
        palette.shadow
    } else {
        palette.selected
    };
    dc.set_pen(outline, 2, PenStyle::Solid);
    dc.set_brush(colour, BrushStyle::Solid);
    for (cx, cy) in cells {
        dc.draw_rectangle(x + cx * block, y + cy * block, block, block);
    }
}

fn draw_demo_board(
    dc: &AutoBufferedPaintDC,
    x: i32,
    y: i32,
    cell: i32,
    stage: usize,
    palette: &VisualPalette,
    is_dark_mode: bool,
) {
    let board_w = cell * BOARD_WIDTH as i32;
    let board_h = cell * BOARD_HEIGHT as i32;
    dc.set_pen(palette.panel_border, 2, PenStyle::Solid);
    dc.set_brush(palette.board_bg, BrushStyle::Solid);
    dc.draw_rectangle(x - 4, y - 4, board_w + 8, board_h + 8);

    for row in 0..BOARD_HEIGHT as i32 {
        for col in 0..BOARD_WIDTH as i32 {
            dc.set_pen(palette.grid, 1, PenStyle::Solid);
            dc.set_brush(palette.board_bg, BrushStyle::Solid);
            dc.draw_rectangle(x + col * cell, y + row * cell, cell, cell);
        }
    }

    let (piece, px, py, rotation) = match stage {
        1 => (TetrominoType::T, 3, 6, 0),
        2 => (TetrominoType::I, 3, 8, 1),
        3 => (TetrominoType::L, 3, 7, 1),
        4 => (TetrominoType::S, 3, 13, 0),
        5 => (TetrominoType::O, 3, 10, 0),
        6 => (TetrominoType::T, 3, 15, 0),
        7 => (TetrominoType::Z, 3, 8, 0),
        _ => (TetrominoType::J, 3, 9, 0),
    };
    let demo = crate::logic::Tetromino {
        t_type: piece,
        x: px,
        y: py,
        rotation,
        item: if stage == 7 {
            Some(ItemType::Laser)
        } else {
            None
        },
    };
    for (bx, by) in demo.get_blocks() {
        if visible(bx, by) {
            let px = x + bx * cell;
            let py = y + by * cell;
            dc.set_pen(pulse_colour(is_dark_mode), 2, PenStyle::Solid);
            dc.set_brush(piece_colour(piece), BrushStyle::Solid);
            dc.draw_rectangle(px + 2, py + 2, cell - 4, cell - 4);
        }
    }
}

fn compact_single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ellipsize(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let shortened: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{shortened}...")
    } else {
        shortened
    }
}

fn wrap_for_width(dc: &AutoBufferedPaintDC, text: &str, max_width: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let (candidate_w, _) = dc.get_text_extent(&candidate);
        if candidate_w <= max_width || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn pulse_colour(is_dark_mode: bool) -> Colour {
    let phase = pulse_phase();
    if is_dark_mode {
        let green = 180 + (phase * 75.0) as u8;
        Colour::rgb(255, green, 40)
    } else {
        let channel = (18.0 + phase * 46.0) as u8;
        Colour::rgb(channel, channel + 18, channel + 34)
    }
}

fn pulse_piece_colour(palette: &VisualPalette) -> Colour {
    let phase = pulse_phase();
    if palette.is_dark {
        let red = (80.0 + phase * 175.0) as u8;
        let blue = (255.0 - phase * 130.0) as u8;
        Colour::rgb(red, 255, blue)
    } else {
        let blue = (64.0 + phase * 58.0) as u8;
        Colour::rgb(0, 34, blue)
    }
}

fn pulse_phase() -> f32 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f32)
        .unwrap_or(0.0);
    ((millis / 500.0).sin() + 1.0) / 2.0
}

fn visible(x: i32, y: i32) -> bool {
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
}

fn piece_colour(piece: TetrominoType) -> Colour {
    match piece {
        TetrominoType::I => Colour::rgb(0, 220, 255),
        TetrominoType::J => Colour::rgb(42, 112, 255),
        TetrominoType::L => Colour::rgb(255, 156, 42),
        TetrominoType::O => Colour::rgb(255, 220, 40),
        TetrominoType::S => Colour::rgb(65, 232, 93),
        TetrominoType::T => Colour::rgb(180, 82, 255),
        TetrominoType::Z => Colour::rgb(255, 68, 98),
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

fn item_symbol(item: ItemType) -> &'static str {
    match item {
        ItemType::Magnet => "M",
        ItemType::Nuke => "N",
        ItemType::Laser => "L",
    }
}
