use crate::{
    colors::{to_rgb, white_fg, INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
    components::{Equipped, InBag, Item, Name},
};
use bracket_terminal::prelude::{ColorPair, DrawBatch};
use bracket_terminal::prelude::{Point, Rect};
use specs::{Join, LendJoin, World, WorldExt};

use crate::{components::SelectedInventoryIdx, game_init::PlayerEntity};

use super::drawing::AccentBox;

pub(crate) fn draw_inventory(draw_batch: &mut DrawBatch, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let items = ecs.read_storage::<Item>();
    let in_bags = ecs.read_storage::<InBag>();
    let names = ecs.read_storage::<Name>();
    let equipped = ecs.read_storage::<Equipped>();

    // TODO: show empty in inventory if inv_count == 0
    let inv_count = (&items, &in_bags, &names)
        .join()
        .filter(|(_, bag, _)| bag.owner == player_entity.0)
        .count();
    draw_batch.draw_accent_box(
        Rect::with_size(40, 2, 35, inv_count + 1),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    // Draw each item in inventory
    for (offset, (item, _, Name(name), equipped)) in (&items, &in_bags, &names, (&equipped).maybe())
        .join()
        .filter(|(_, bag, _, _)| bag.owner == player_entity.0)
        .enumerate()
    {
        let status = if equipped.is_some() { "(E)" } else { "" };
        let qty = if item.qty.0 > 1 {
            format!("{}x ", item.qty)
        } else {
            "".to_string()
        };
        draw_batch.print_color(
            Point::new(42, 2 + offset + 1),
            format!("{:X}| {status}{qty}{name}", offset + 1),
            white_fg(to_rgb(INVENTORY_BACKGROUND)),
        );
    }

    // Draw cursor for selected item
    let selected_indices = ecs.read_storage::<SelectedInventoryIdx>();
    if let Some(selection) = selected_indices.get(player_entity.0) {
        if selection.first_idx < inv_count {
            draw_batch.print(Point::new(41, 3 + selection.first_idx), ">");
        }
    }
}
