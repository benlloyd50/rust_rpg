use crate::{
    colors::{to_rgb, white_fg, INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
    data_read::ENTITY_DB,
};
use bracket_terminal::prelude::{ColorPair, DrawBatch};
use bracket_terminal::prelude::{Point, Rect};
use specs::{World, WorldExt};

use crate::{
    components::{Backpack, SelectedInventoryIdx},
    game_init::PlayerEntity,
};

use super::drawing::AccentBox;

pub(crate) fn draw_inventory(draw_batch: &mut DrawBatch, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let player_bag = ecs.read_storage::<Backpack>();
    let items_in_bag = match player_bag.get(player_entity.0) {
        Some(bag) => bag,
        None => {
            panic!("Player entity does not have a Backpack component.");
        }
    };

    draw_batch.draw_accent_box(
        Rect::with_size(40, 2, 35, items_in_bag.len() + 1),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    let mut idx = 1;
    let edb = &ENTITY_DB.lock().unwrap();
    for (iid, qty) in items_in_bag.iter() {
        let name = match edb.items.get_by_id(iid.0) {
            Some(info) => &info.name,
            None => {
                eprintln!("ItemID: {:?} was not found in the Entity database", iid);
                "{} MISSING ITEM NAME"
            }
        };
        draw_batch.print_color(
            Point::new(42, 2 + idx),
            format!("{:X}| {:03} {}", idx, qty.0, name),
            white_fg(to_rgb(INVENTORY_BACKGROUND)),
        );
        idx += 1;
    }

    let selected_indices = ecs.read_storage::<SelectedInventoryIdx>();
    if let Some(selection) = selected_indices.get(player_entity.0) {
        if selection.first_idx < items_in_bag.len() {
            draw_batch.print(Point::new(41, 3 + selection.first_idx), ">");
        }
    }
}
