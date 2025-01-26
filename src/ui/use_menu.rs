use bracket_lib::terminal::{ColorPair, DrawBatch, Point, Rect, TextAlign};
use specs::{World, WorldExt};

use crate::{
    colors::to_rgb,
    components::{Consumable, Equipable, SelectedInventoryItem},
    game_init::PlayerEntity,
};

use super::{
    drawing::AccentBox,
    inventory::{INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
};

const BASE_ACTIONS: [&str; 4] =
    ["#[orange]U#[]se with", "#[orange]E#[]xamine", "#[orange]D#[]rop", "#[lightgray]<Esc>#[]"];
const EQUIP_ACTION: &str = "#[]E#[orange]q#[]uip";
const CONSUME_ACTION: &str = "#[orange]C#[]onsume";

pub fn draw_use_menu(draw_batch: &mut DrawBatch, ecs: &World) {
    let selected_items = ecs.read_storage::<SelectedInventoryItem>();
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut use_menu_actions = BASE_ACTIONS.to_vec();
    let selected_inv = selected_items.get(player_entity.0).unwrap();

    let consumables = ecs.read_storage::<Consumable>();
    if consumables.get(selected_inv.first_item).is_some() {
        use_menu_actions.insert(3, CONSUME_ACTION);
    }

    let equipables = ecs.read_storage::<Equipable>();
    if equipables.get(selected_inv.first_item).is_some() {
        use_menu_actions.insert(3, EQUIP_ACTION);
    }

    draw_batch.draw_accent_box(
        Rect::with_size(28, 6, 10, use_menu_actions.len() + 1),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    for (idx, action) in use_menu_actions.iter().enumerate() {
        draw_batch.printer(Point::new(29, 7 + idx), action, TextAlign::Left, Some(to_rgb(INVENTORY_BACKGROUND).into()));
    }
}
