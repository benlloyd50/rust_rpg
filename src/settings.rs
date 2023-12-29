use crate::{CL_INTERACTABLES, FONT_INTERACTABLES, FONT_INTERACTABLES_OUTLINE};
use bracket_lib::terminal::BTerm;
use std::fs;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

const GENERAL_CONFIG_PATH: &str = "./config.json";

// TODO: when config is loaded if values fail only change the failing values in favor of default, leave
// everything else unchanged

/// The config for how the in game settings menu are configured
#[derive(Serialize, Deserialize)]
pub struct SettingsConfig {
    pub sprite_mode: SpriteMode,
    pub text_font: TextFonts,
}

impl SettingsConfig {
    pub fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap(); // doubt a generic serialize will fail so probably safe
        if let Err(e) = fs::write(GENERAL_CONFIG_PATH, data) {
            error!("General Config Saving Error: {}", e);
        }
    }

    /// Loads the config file located at `GENERAL_CONFIG_PATH` or creates a default
    pub fn load() -> Self {
        match fs::read_to_string(GENERAL_CONFIG_PATH) {
            Ok(raw_config) => match serde_json::from_str(&raw_config) {
                Ok(g) => g,
                Err(e) => {
                    error!("General Config Reading Error: {}", e);
                    warn!("Config file at {} could not be read. Falling back to default.", GENERAL_CONFIG_PATH);
                    SettingsConfig::default()
                }
            },
            Err(e) => {
                error!("General Config Loading Error: {}", e);
                SettingsConfig::default()
            }
        }
    }
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self { sprite_mode: SpriteMode::default(), text_font: TextFonts::default() }
    }
}

#[derive(Serialize, Deserialize)]
pub enum TextFonts {
    Zaratustra,
    Terminal,
}

impl Default for TextFonts {
    fn default() -> Self {
        Self::Zaratustra
    }
}

/// How the interactables sprites will be drawn on screen
#[derive(Serialize, Deserialize)]
pub enum SpriteMode {
    Outline,
    Blocked,
}

impl Default for SpriteMode {
    fn default() -> Self {
        Self::Outline
    }
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub enum SettingsSelection {
    SpriteMode,
}

pub enum SettingsAction {
    Selected,
    Waiting,
    ReturnToMainMenu,
}

pub fn handle_setting_selected(setting: &SettingsSelection, cfg: &mut SettingsConfig, ctx: &mut BTerm) {
    match setting {
        SettingsSelection::SpriteMode => toggle_sprite_mode(cfg, ctx),
    }
}

fn toggle_sprite_mode(cfg: &mut SettingsConfig, ctx: &mut BTerm) {
    cfg.sprite_mode = match cfg.sprite_mode {
        SpriteMode::Outline => SpriteMode::Blocked,
        SpriteMode::Blocked => SpriteMode::Outline,
    };
    ctx.set_active_console(CL_INTERACTABLES);
    let active_id = match cfg.sprite_mode {
        SpriteMode::Outline => FONT_INTERACTABLES_OUTLINE,
        SpriteMode::Blocked => FONT_INTERACTABLES,
    };
    debug!("Active id is {}", active_id);
    ctx.set_active_font(active_id, false);
}
