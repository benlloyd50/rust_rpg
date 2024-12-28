/*
 * Defines all the colors used throughout the game.
 * Colors should use their color name
 * Palettes should be named after their use ingame
 */
use bracket_lib::terminal::{register_palette_color, ColorPair, RGB, WHITESMOKE};

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

    register_palette_color(PL_SETTINGS_TEXT, to_rgb(WHITE));
    register_palette_color(PL_SETTINGS_HIGHLIGHT, to_rgb(MIDDLERED));

    register_palette_color(PL_ORANGE, to_rgb(SALMON));
    register_palette_color(PL_MENU_TEXT, to_rgb(DARKERBROWN));
    register_palette_color(PL_MENU_ACCENT_TEXT, to_rgb(DARKBROWN));

    register_palette_color(PL_MAX_HP, RGB::from_hex("#67fc3a").unwrap());
    register_palette_color(PL_MED_HP, RGB::from_hex("#48aa2a").unwrap());
    register_palette_color(PL_LOW_HP, RGB::from_hex("#eab838").unwrap());
    register_palette_color(PL_CRITICAL_HP, RGB::from_hex("#fc321b").unwrap());

    register_palette_color("red", to_rgb(MIDDLERED));
    register_palette_color("bright_green", RGB::from_u8(52, 156, 88));
    register_palette_color("white", to_rgb(WHITE));
    register_palette_color("lightgray", RGB::from_u8(161, 161, 161));
}

// Palette Definitions always start with PL_*
pub const PL_KEYBIND: &str = "keybind";
pub const PL_MAIN_MENU_TEXT: &str = "main_menu_text";
pub const PL_MAIN_MENU_TEXT_HIGHLIGHT: &str = "main_menu_text_hl";

pub const PL_SETTINGS_TEXT: &str = "settings_text";
pub const PL_SETTINGS_HIGHLIGHT: &str = "settings_highlight";

pub const PL_ORANGE: &str = "orange";
pub const PL_MENU_TEXT: &str = "menu_text";
pub const PL_MENU_ACCENT_TEXT: &str = "menu_accent_text";

pub const PL_MAX_HP: &str = "max_hp";
pub const PL_MED_HP: &str = "med_hp";
pub const PL_LOW_HP: &str = "low_hp";
pub const PL_CRITICAL_HP: &str = "critical_hp";

// Color Definitions
pub const MIDDLERED: Color = (183, 65, 50);
pub const SALMON: Color = (230, 113, 70);
pub const MAROON: Color = (122, 40, 73);
pub const DARKBLUEPURPLE: Color = (18, 14, 35);
pub const DARKBLUE: Color = (42, 41, 66);
pub const WHITE: Color = (222, 222, 222);
pub const PARCHMENT: Color = (255, 241, 169);
pub const TEXASROSE: Color = (235, 184, 91);
#[allow(unused)]
pub const DARKESTBROWN: Color = (64, 46, 43);
pub const DARKERBROWN: Color = (118, 64, 50);
pub const DARKBROWN: Color = (161, 92, 52);
