use bracket_lib::color::GREY4;
use bracket_lib::terminal::{to_char, to_cp437, ColorPair, DrawBatch, Point, Rect, TextAlign, WHITESMOKE};

use crate::game_init::{InputWorldConfig, NewGameMenuSelection};
use crate::saveload::any_save_game_exists;
use crate::{
    colors::{Color, DARKBLUE, DARKBLUEPURPLE, MIDDLERED, PL_SETTINGS_HIGHLIGHT, PL_SETTINGS_TEXT, SALMON},
    player::MenuSelection,
    settings::{SettingsConfig, SpriteMode},
    CL_EFFECTS, CL_TEXT, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

// Menu contianing the starting options for the player
const MENU_WIDTH: usize = 15;
const MENU_HEIGHT: usize = 2 + MENU_OPTIONS.len() * 2;
const MENU_START_Y: usize = DISPLAY_HEIGHT * 2 - MENU_HEIGHT - 10;
const MENU_START_X: usize = DISPLAY_WIDTH - MENU_WIDTH / 2;

// TODO: Somehow this should be generated from an enum like MenuSelection
const MENU_OPTIONS: [&str; 4] = ["new game", "load game", "settings", "quit game"];

const MAIN_MENU_ACCENT: Color = MIDDLERED;
const MAIN_MENU_BG: Color = DARKBLUEPURPLE;
const MAIN_MENU_HL: Color = DARKBLUE;
const MAIN_MENU_TEXT_HL: Color = SALMON;

pub fn draw_main_menu(draw_batch: &mut DrawBatch, hovered: &MenuSelection) {
    // Background, on effects as to not interfere with title anim
    draw_batch.target(CL_EFFECTS);
    draw_batch.fill_region(
        Rect::with_size(0, 0, DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    draw_batch.target(CL_TEXT);
    let menu_rect = Rect::with_size(MENU_START_X, MENU_START_Y, MENU_WIDTH, MENU_HEIGHT);
    draw_batch.draw_hollow_double_box(menu_rect, ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG));
    draw_batch.fill_region(
        Rect::with_exact(menu_rect.x1 + 1, menu_rect.y1 + 1, menu_rect.x2, menu_rect.y2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    for (idx, opt) in MENU_OPTIONS.iter().enumerate() {
        let colors = if opt == &"load game" && !any_save_game_exists() {
            ColorPair::new(GREY4, MAIN_MENU_BG)
        } else if hovered.as_lowercase() == *opt {
            ColorPair::new(MAIN_MENU_TEXT_HL, MAIN_MENU_HL)
        } else {
            ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG)
        };

        let text = if hovered.as_lowercase() == *opt {
            format!("{}{}{}", to_char(16), opt.to_uppercase(), to_char(17))
        } else {
            opt.to_string()
        };

        draw_batch.print_color(Point::new(MENU_START_X + 3, MENU_START_Y + 2 + (2 * idx)), text, colors);
    }
}

pub fn draw_settings(draw_batch: &mut DrawBatch, cfg: &SettingsConfig) {
    draw_batch.target(CL_TEXT);
    // Background
    draw_batch.fill_region(
        Rect::with_size(0, 0, DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    // Settings Box
    draw_batch.draw_hollow_double_box(
        Rect::with_size(MENU_START_X, MENU_START_Y, MENU_WIDTH * 2, MENU_HEIGHT),
        ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG),
    );

    // Sprite Mode
    let (blocked, outline) = match cfg.sprite_mode {
        SpriteMode::Outline => (PL_SETTINGS_TEXT, PL_SETTINGS_HIGHLIGHT),
        SpriteMode::Blocked => (PL_SETTINGS_HIGHLIGHT, PL_SETTINGS_TEXT),
    };
    draw_batch.printer(
        Point::new(MENU_START_X + 1, MENU_START_Y + 2),
        format!("#[{}]Sprite Mode: #[{}]Blocked #[{}]Outline", PL_SETTINGS_TEXT, blocked, outline),
        TextAlign::Left,
        Some(MAIN_MENU_BG.into()),
    );
}

pub fn draw_load_game_menu(draw_batch: &mut DrawBatch, save_games: &[String], hovering: usize) {
    // Background
    draw_batch.target(CL_TEXT);
    draw_batch.fill_region(
        Rect::with_size(0, 0, DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    let menu_height = save_games.len() + 2;

    // Draw box for menu
    draw_batch.target(CL_TEXT);
    let menu_rect = Rect::with_size(MENU_START_X, MENU_START_Y - menu_height, 19, menu_height);
    draw_batch.draw_hollow_double_box(menu_rect, ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG));
    draw_batch.fill_region(
        Rect::with_exact(menu_rect.x1 + 1, menu_rect.y1 + 1, menu_rect.x2, menu_rect.y2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    for (idx, file_name) in save_games.iter().enumerate() {
        let colors = if hovering == idx {
            ColorPair::new(MAIN_MENU_TEXT_HL, MAIN_MENU_HL)
        } else {
            ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG)
        };
        let text = if hovering == idx {
            format!("{}{}{}", to_char(16), file_name.to_uppercase(), to_char(17))
        } else {
            file_name.to_string()
        };
        draw_batch.print_color(Point::new(MENU_START_X + 1, (MENU_START_Y - menu_height) + 1 + idx), text, colors);
    }
}

pub fn draw_new_game_menu(
    draw_batch: &mut DrawBatch,
    hovering: &NewGameMenuSelection,
    world_cfg: &InputWorldConfig,
    form_errors: &[String],
) {
    // Background
    draw_batch.target(CL_TEXT);
    draw_batch.fill_region(
        Rect::with_size(0, 0, DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    let menu_start_x = MENU_START_X - 10;
    let menu_height = 20;

    draw_batch.target(CL_TEXT);
    let menu_rect = Rect::with_size(menu_start_x, MENU_START_Y - menu_height, 29, menu_height);
    draw_batch.draw_hollow_double_box(menu_rect, ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG));
    draw_batch.fill_region(
        Rect::with_exact(menu_rect.x1 + 1, menu_rect.y1 + 1, menu_rect.x2, menu_rect.y2),
        ColorPair::new(WHITESMOKE, MAIN_MENU_BG),
        to_cp437(' '),
    );

    let hl = ColorPair::new(MAIN_MENU_TEXT_HL, MAIN_MENU_HL);
    let no = ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG);
    let (name, width, height, seed, finish) = match hovering {
        NewGameMenuSelection::WorldName => (hl, no, no, no, no),
        NewGameMenuSelection::Width => (no, hl, no, no, no),
        NewGameMenuSelection::Height => (no, no, hl, no, no),
        NewGameMenuSelection::Seed => (no, no, no, hl, no),
        NewGameMenuSelection::Finalize => (no, no, no, no, hl),
    };

    draw_batch.print_color(
        Point::new(menu_start_x + 1, MENU_START_Y - menu_height + 1),
        format!("World Name: {}", world_cfg.world_name),
        name,
    );

    draw_batch.print_color(Point::new(menu_start_x + 1, MENU_START_Y - menu_height + 3), "==Map==", no);
    draw_batch.print_color(
        Point::new(menu_start_x + 1, MENU_START_Y - menu_height + 4),
        format!("Width: {}", world_cfg.width),
        width,
    );
    draw_batch.print_color(
        Point::new(menu_start_x + 1, MENU_START_Y - menu_height + 5),
        format!("Height: {}", world_cfg.height),
        height,
    );
    draw_batch.print_color(
        Point::new(menu_start_x + 1, MENU_START_Y - menu_height + 6),
        format!("Sea Level (0-255): {}", world_cfg.height),
        height,
    );
    draw_batch.print_color(
        Point::new(menu_start_x + 1, MENU_START_Y - menu_height + 7),
        format!("Seed: {}", world_cfg.seed),
        seed,
    );

    draw_batch.print_color(Point::new(menu_start_x + 29 / 2, MENU_START_Y), "Finish".to_string(), finish);

    for (idx, err) in form_errors.iter().enumerate() {
        draw_batch.print_color(Point::new(menu_start_x + 29 / 2, MENU_START_Y - (menu_height + idx + 1)), err, hl);
    }
}
