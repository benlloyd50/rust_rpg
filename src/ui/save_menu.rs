use bracket_terminal::prelude::{to_char, ColorPair, DrawBatch, Point, Rect, TextAlign};

use crate::{
    colors::{MIDDLERED, PL_KEYBIND, SALMON},
    CL_TEXT, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

use super::drawing::AccentBox;

const MENU_X: usize = DISPLAY_WIDTH - MENU_WIDTH / 2;
const MENU_WIDTH: usize = DISPLAY_WIDTH * 2 / 4;
const MENU_Y: usize = DISPLAY_HEIGHT - MENU_HEIGHT / 2;
const MENU_HEIGHT: usize = DISPLAY_HEIGHT / 5;

pub fn draw_save_menu(draw_batch: &mut DrawBatch) {
    draw_batch.target(CL_TEXT);
    draw_batch
        .draw_accent_box(Rect::with_size(MENU_X, MENU_Y, MENU_WIDTH, MENU_HEIGHT), ColorPair::new(MIDDLERED, SALMON));
    draw_batch.printer(
        Point::new(MENU_X + 1, MENU_Y),
        format!("#[white]{}{} Quitting{}#[]", to_char(180), to_char(2), to_char(195)),
        TextAlign::Left,
        Some(MIDDLERED.into()),
    );
    draw_batch.printer(
        Point::new(MENU_X + 2, MENU_Y + 2),
        format!("#[{}]s#[white]ave", PL_KEYBIND),
        TextAlign::Left,
        Some(SALMON.into()),
    );
    draw_batch.printer(
        Point::new(MENU_X + 2, MENU_Y + 3),
        format!("#[{}]d#[white]ont save", PL_KEYBIND),
        TextAlign::Left,
        Some(SALMON.into()),
    );
    draw_batch.printer(
        Point::new(MENU_X + 2, MENU_Y + 4),
        format!("#[{}]<esc>", PL_KEYBIND),
        TextAlign::Left,
        Some(SALMON.into()),
    );
}
