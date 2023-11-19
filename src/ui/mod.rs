use bracket_terminal::prelude::DrawBatch;
use specs::World;

use crate::{
    config::InventoryConfig,
    inventory::{check_inventory_selection, SelectionStatus},
    AppState, CL_TEXT,
};

use self::{inventory::draw_inventory, message_log::{draw_message_log, draw_turn_counter}, use_menu::draw_use_menu};

mod drawing;
mod inventory;
pub(crate) mod message_log;
mod use_menu;

pub fn draw_ui(ecs: &World, appstate: &AppState, cfg: &InventoryConfig) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();

    draw_message_log(&mut draw_batch, ecs);
    draw_turn_counter(&mut draw_batch, ecs);

    match *appstate {
        AppState::PlayerInInventory => {
            draw_inventory(&mut draw_batch, ecs, cfg);
            if check_inventory_selection(ecs) == SelectionStatus::SelectionWithoutAction {
                draw_use_menu(&mut draw_batch, ecs);
            }
        }
        _ => {}
    }

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}
