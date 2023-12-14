/*
 * Defines all the colors used throughout the game.
 * Colors should use their color name
 * Palettes should be named after their use ingame
 */
use bracket_terminal::prelude::{register_palette_color, ColorPair, RGB, WHITESMOKE};

pub type Color = (u8, u8, u8);

pub fn white_fg(rgb: RGB) -> ColorPair {
    ColorPair::new(WHITESMOKE, rgb)
}

pub fn to_rgb(color: Color) -> RGB {
    RGB::from(color)
}

/// Adds all the Palettes to BTerm
pub fn initialize_printer_palette() {
    register_palette_color(PL_KEYBIND, to_rgb(MAROON));
    register_palette_color(PL_MAIN_MENU_TEXT, to_rgb(MIDDLERED));
    register_palette_color(PL_MAIN_MENU_TEXT_HIGHLIGHT, to_rgb(SALMON));

    register_palette_color("orange", to_rgb(SALMON));

    register_palette_color("red", to_rgb(MIDDLERED));
    register_palette_color("bright_green", RGB::from_u8(52, 156, 88));
    register_palette_color("white", to_rgb(WHITE));
    register_palette_color("lightgray", RGB::from_u8(161, 161, 161));
}

// Usage Definitions these should move into their own file
pub const INVENTORY_BACKGROUND: Color = (44, 57, 71);
pub const INVENTORY_OUTLINE: Color = (61, 84, 107);

// Palette Definitions always start with PL_*
pub const PL_KEYBIND: &str = "keybind";
pub const PL_MAIN_MENU_TEXT: &str = "main_menu_text";
pub const PL_MAIN_MENU_TEXT_HIGHLIGHT: &str = "main_menu_text_hl";

// Color Definitions
pub const MIDDLERED: Color = (183, 65, 50);
pub const SALMON: Color = (230, 113, 70);
pub const MAROON: Color = (122, 40, 73);
pub const DARKBLUEPURPLE: Color = (18, 14, 35);
pub const DARKBLUE: Color = (42, 41, 66);
pub const WHITE: Color = (222, 222, 222);
