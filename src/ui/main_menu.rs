use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, Rect, PURPLE, WHITESMOKE};

use crate::{
    colors::{INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
    player::MenuSelection,
    CL_TEXT, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

use super::drawing::AccentBox;

// Menu contianing the new, load, and settings
const MENU_WIDTH: usize = 14;
const MENU_HEIGHT: usize = 8;
const MENU_START_Y: usize = DISPLAY_HEIGHT * 2 - MENU_HEIGHT - 10;
const MENU_START_X: usize = DISPLAY_WIDTH - MENU_WIDTH / 2;

const MENU_OPTIONS: [&str; 3] = ["new game", "load game", "settings"];

pub fn draw_main_menu(draw_batch: &mut DrawBatch, hovered: &MenuSelection) {
    draw_batch.target(CL_TEXT);
    draw_batch.draw_accent_box(
        Rect::with_size(MENU_START_X, MENU_START_Y, MENU_WIDTH, MENU_HEIGHT),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    for (idx, opt) in MENU_OPTIONS.iter().enumerate() {
        let bg = if hovered.to_lowercase() == opt.to_owned() { PURPLE } else { INVENTORY_BACKGROUND };
        draw_batch.print_color(
            Point::new(MENU_START_X + 3, MENU_START_Y + 2 + (2 * idx)),
            opt,
            ColorPair::new(WHITESMOKE, bg),
        );
    }
}
