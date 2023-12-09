use bracket_terminal::prelude::DrawBatch;
use specs::World;

use crate::{
    config::InventoryConfig,
    inventory::{check_inventory_selection, SelectionStatus},
    AppState, CL_EFFECTS, CL_EFFECTS2, CL_TEXT,
};

use self::{
    fishing::draw_fishing_bar,
    inventory::draw_inventory,
    main_menu::draw_main_menu,
    message_log::{draw_message_log, draw_turn_counter},
    use_menu::draw_use_menu,
};

mod drawing;
mod fishing;
mod inventory;
mod main_menu;
pub(crate) mod message_log;
mod use_menu;

pub fn draw_ui(ecs: &World, appstate: &AppState, cfg: &InventoryConfig) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_EFFECTS).cls();
    draw_batch.target(CL_EFFECTS2).cls();
    draw_batch.target(CL_TEXT).cls();

    match *appstate {
        AppState::InGame => {
            // TEST: this was moved from above, this may have to be updated in more states than just this one
            // for a fact this will cause problems in PlayerInInventory and ActivityBound
            draw_message_log(&mut draw_batch, ecs);
            draw_turn_counter(&mut draw_batch, ecs);
        }
        AppState::PlayerInInventory => {
            draw_inventory(&mut draw_batch, ecs, cfg);
            if check_inventory_selection(ecs) == SelectionStatus::SelectionWithoutAction {
                draw_use_menu(&mut draw_batch, ecs);
            }
        }
        AppState::ActivityBound { .. } => {
            draw_fishing_bar(&mut draw_batch, ecs);
        }
        AppState::MainMenu { hovering } => {
            draw_main_menu(&mut draw_batch, &hovering);
        }
        _ => {}
    }

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}
