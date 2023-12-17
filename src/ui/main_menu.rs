use bracket_color::prelude::GREY4;
use bracket_terminal::prelude::{to_char, to_cp437, ColorPair, DrawBatch, Point, Rect, TextAlign, WHITESMOKE};

use crate::{
    colors::{Color, DARKBLUE, DARKBLUEPURPLE, MIDDLERED, PL_SETTINGS_HIGHLIGHT, PL_SETTINGS_TEXT, SALMON},
    player::MenuSelection,
    saveload::save_game_exists,
    settings::{SettingsConfig, SpriteMode},
    CL_EFFECTS, CL_TEXT, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

// Menu contianing the new, load, and settings
const MENU_WIDTH: usize = 15;
const MENU_HEIGHT: usize = 8;
const MENU_START_Y: usize = DISPLAY_HEIGHT * 2 - MENU_HEIGHT - 10;
const MENU_START_X: usize = DISPLAY_WIDTH - MENU_WIDTH / 2;

const MENU_OPTIONS: [&str; 3] = ["new game", "load game", "settings"];

const MAIN_MENU_ACCENT: Color = MIDDLERED;
const MAIN_MENU_BG: Color = DARKBLUEPURPLE;
const MAIN_MENU_HL: Color = DARKBLUE;
const MAIN_MENU_TEXT_HL: Color = SALMON;

pub fn draw_main_menu(draw_batch: &mut DrawBatch, hovered: &MenuSelection) {
    // This should target a layer that is low so then layers can be drawn on top of it if needed
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
        let colors = if opt == &"load game" && !save_game_exists() {
            ColorPair::new(GREY4, MAIN_MENU_BG)
        } else if hovered.to_lowercase() == opt.to_owned() {
            ColorPair::new(MAIN_MENU_TEXT_HL, MAIN_MENU_HL)
        } else {
            ColorPair::new(MAIN_MENU_ACCENT, MAIN_MENU_BG)
        };
        let text = if hovered.to_lowercase() == opt.to_owned() {
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
        Rect::with_size(MENU_START_X, MENU_START_Y, MENU_WIDTH, MENU_HEIGHT),
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
