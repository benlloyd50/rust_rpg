/* Defines all the colors used throughout the game.
 * Colors should be defined with a name or color depending on their usage.
 */
use bracket_terminal::prelude::{ColorPair, RGB, WHITESMOKE};

type Color = (u8, u8, u8);

pub fn white_fg(rgb: RGB) -> ColorPair {
    ColorPair::new(WHITESMOKE, rgb)
}

pub fn to_rgb(color: Color) -> RGB {
    RGB::from(color)
}

pub const INVENTORY_BACKGROUND: Color = (44, 57, 71);
pub const INVENTORY_OUTLINE: Color = (61, 84, 107);
