use crate::{
    colors::{to_rgb, white_fg, INVENTORY_BACKGROUND, INVENTORY_OUTLINE},
    components::{Equipped, InBag, Item, Name},
};
use bracket_terminal::prelude::{ColorPair, DrawBatch};
use bracket_terminal::prelude::{Point, Rect};
use specs::{Join, LendJoin, ReadStorage, World, WorldExt};

use crate::{components::SelectedInventoryItem, game_init::PlayerEntity};

use super::drawing::AccentBox;

pub(crate) fn draw_inventory(draw_batch: &mut DrawBatch, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let items: ReadStorage<Item> = ecs.read_storage();
    let inbags: ReadStorage<InBag> = ecs.read_storage();
    let names: ReadStorage<Name> = ecs.read_storage();
    let equipped: ReadStorage<Equipped> = ecs.read_storage();

    let entities = ecs.entities();
    let mut data: Vec<(specs::Entity, &Item, &InBag, &Name, Option<&Equipped>)> =
        (&entities, &items, &inbags, &names, (&equipped).maybe())
            // important: this must match in src/inventory.rs until a better solution is found to share code
            .join()
            .filter(|(_, _, bag, _, _)| bag.owner == player_entity.0)
            .collect();
    data.sort_by(|a, b| a.3.cmp(b.3));

    // TODO: show empty in inventory if inv_count == 0
    let inv_count = data.len();
    draw_batch.draw_accent_box(
        Rect::with_size(40, 2, 35, inv_count + 1),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    let selected_items = ecs.read_storage::<SelectedInventoryItem>();
    let selected_item = selected_items
        .get(player_entity.0)
        .map(|SelectedInventoryItem { first_item, .. }| first_item);

    // Draw each item in inventory
    for (offset, (item_entity, item, _, Name(name), equipped)) in data.iter().enumerate() {
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

        if Some(item_entity) == selected_item {
            draw_batch.print(Point::new(41, 3 + offset), ">");
        }
    }
}
