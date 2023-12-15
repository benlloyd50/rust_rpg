use bracket_terminal::prelude::DrawBatch;
use specs::World;

use crate::{
    config::ConfigMaster,
    inventory::{check_inventory_selection, SelectionStatus},
    AppState, CL_EFFECTS, CL_EFFECTS2, CL_TEXT,
};

use self::{
    fishing::draw_fishing_bar,
    inventory::draw_inventory,
    main_menu::{draw_main_menu, draw_settings},
    message_log::{draw_message_log, draw_turn_counter},
    save_menu::draw_save_menu,
    use_menu::draw_use_menu,
};

mod drawing;
mod fishing;
mod inventory;
mod main_menu;
pub(crate) mod message_log;
mod save_menu;
mod use_menu;

pub fn draw_ui(ecs: &World, appstate: &AppState, cfg: &ConfigMaster) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_EFFECTS).cls();
    draw_batch.target(CL_EFFECTS2).cls();
    draw_batch.target(CL_TEXT).cls();

    match *appstate {
        AppState::InGame => {
            draw_message_log(&mut draw_batch, ecs);
            draw_turn_counter(&mut draw_batch, ecs);
        }
        AppState::PlayerInInventory => {
            draw_inventory(&mut draw_batch, ecs, &cfg.inventory);
            if check_inventory_selection(ecs) == SelectionStatus::SelectionWithoutAction {
                draw_use_menu(&mut draw_batch, ecs);
            }

            draw_message_log(&mut draw_batch, ecs);
            draw_turn_counter(&mut draw_batch, ecs);
        }
        AppState::ActivityBound { .. } => {
            draw_fishing_bar(&mut draw_batch, ecs);
        }
        AppState::MainMenu { hovering } => {
            draw_main_menu(&mut draw_batch, &hovering);
        }
        AppState::SaveGame => {
            draw_save_menu(&mut draw_batch);
        }
        AppState::SettingsMenu { .. } => {
            draw_settings(&mut draw_batch, &cfg.general);
        }
        _ => {}
    }

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}
