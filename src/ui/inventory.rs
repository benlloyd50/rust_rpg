use crate::colors::{PL_MENU_ACCENT_TEXT, PL_MENU_TEXT};
use crate::{
    colors::{self, to_rgb, Color},
    components::{Equipped, InBag, Item, Name},
    config::{InventoryConfig, SortMode},
};
use bracket_lib::terminal::{ColorPair, DrawBatch, TextAlign};
use bracket_lib::terminal::{Point, Rect};
use specs::{Join, LendJoin, ReadStorage, World, WorldExt};

use crate::{components::SelectedInventoryItem, game_init::PlayerEntity};

use super::drawing::AccentBox;
//
// Usage Definitions these should move into their own file
pub const INVENTORY_BACKGROUND: Color = colors::PARCHMENT; // (44, 57, 71);
pub const INVENTORY_OUTLINE: Color = colors::TEXASROSE; //(61, 84, 107);

pub(crate) fn draw_inventory(draw_batch: &mut DrawBatch, ecs: &World, cfg: &InventoryConfig) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let items: ReadStorage<Item> = ecs.read_storage();
    let inbags: ReadStorage<InBag> = ecs.read_storage();
    let names: ReadStorage<Name> = ecs.read_storage();
    let equipped: ReadStorage<Equipped> = ecs.read_storage();

    let entities = ecs.entities();
    // important: this must match in src/inventory.rs until a better solution is found to share code
    let mut data = (&entities, &items, &inbags, &names, (&equipped).maybe())
        .join()
        .filter(|(_, _, bag, _, _)| bag.owner == player_entity.0)
        .collect::<Vec<(specs::Entity, &Item, &InBag, &Name, Option<&Equipped>)>>();
    data.sort_by(|a, b| match cfg.sort_mode {
        SortMode::NameABC => a.3.cmp(b.3),
        SortMode::IDAsc => a.1.id.cmp(&b.1.id),
        _ => a.1.id.cmp(&b.1.id),
    });

    // TODO: show empty in inventory if inv_count == 0
    let inv_count = data.len();
    draw_batch.draw_accent_box(
        Rect::with_size(40, 2, 35, inv_count + 1),
        ColorPair::new(INVENTORY_OUTLINE, INVENTORY_BACKGROUND),
    );

    let selected_items = ecs.read_storage::<SelectedInventoryItem>();
    let selected_item = selected_items.get(player_entity.0).map(|SelectedInventoryItem { first_item, .. }| first_item);

    // Draw each item in inventory
    for (offset, (item_entity, item, _, Name(name), equipped)) in data.iter().enumerate() {
        let status = if equipped.is_some() { "(E)" } else { "" };
        let qty = if item.qty.0 > 1 { format!("{}x ", item.qty) } else { "".to_string() };
        draw_batch.printer(
            Point::new(42, 2 + offset + 1),
            format!("#[{PL_MENU_TEXT}]{:X}| #[{PL_MENU_ACCENT_TEXT}]{status}{qty}{name}", offset + 1),
            TextAlign::Left,
            Some(to_rgb(INVENTORY_BACKGROUND).into()),
        );

        if Some(item_entity) == selected_item {
            draw_batch.print(Point::new(41, 3 + offset), ">");
        }
    }
}
