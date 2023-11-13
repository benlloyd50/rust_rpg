use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, Rect, TextAlign};

use crate::colors::{to_rgb, INVENTORY_BACKGROUND, INVENTORY_OUTLINE};

use super::drawing::AccentBox;

const INVENTORY_ACTIONS: [&str; 5] = [
    "#[orange]U#[]se with",
    "#[orange]E#[]xamine",
    "#[orange]D#[]rop",
    "#[]E#[orange]q#[]uip",
    "#[lightgray]<Esc>#[]",
];

pub fn draw_use_menu(draw_batch: &mut DrawBatch) {
    draw_batch.draw_accent_box(
        Rect::with_size(28, 6, 10, INVENTORY_ACTIONS.len() + 1),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );
    // NOTE: we would probably want to keep track of what actions are possible for a specific item
    for (idx, action) in INVENTORY_ACTIONS.iter().enumerate() {
        draw_batch.printer(
            Point::new(29, 7 + idx),
            action,
            TextAlign::Left,
            Some(to_rgb(INVENTORY_BACKGROUND).into()),
        );
    }
}
