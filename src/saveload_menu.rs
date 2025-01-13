use std::fs;

use bracket_lib::prelude::{BTerm, VirtualKeyCode};

use crate::saveload::SAVE_PATH;

pub fn get_save_games() -> Vec<String> {
    let paths = fs::read_dir(SAVE_PATH).unwrap();
    // that's just a silly looking conversion but that's without error handling
    paths.map(|p| p.unwrap().file_name().into_string().unwrap()).collect()
}

#[derive(Default)]
pub struct GameSaves {
    pub saves: Vec<String>,
}

#[derive(Default)]
pub struct LoadedWorld {
    pub file_name: Option<String>,
    pub temp_input: String,
}

pub enum LoadMenuAction {
    MoveDown,
    MoveUp,
    Select,
    Back,
    Waiting,
}

pub fn p_input_load_game_menu(ctx: &mut BTerm) -> LoadMenuAction {
    if let Some(key) = ctx.key {
        return match key {
            VirtualKeyCode::Down | VirtualKeyCode::S => LoadMenuAction::MoveDown,
            VirtualKeyCode::Up | VirtualKeyCode::W => LoadMenuAction::MoveUp,
            VirtualKeyCode::Return => LoadMenuAction::Select,
            VirtualKeyCode::Escape => LoadMenuAction::Back,
            _ => LoadMenuAction::Waiting,
        };
    }

    LoadMenuAction::Waiting
}
