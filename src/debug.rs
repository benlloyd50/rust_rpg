use bracket_terminal::prelude::{to_char, BTerm, TextAlign, VirtualKeyCode, RGB, RGBA, WHITESMOKE};
use itertools::Itertools;
use specs::{Join, ReadStorage, World, WorldExt};

use crate::{
    camera::mouse_to_map_pos,
    components::{HealthStats, InBag, Interactor, Item, Name, Position, SelectedInventoryItem, Transform},
    config::{InventoryConfig, SortMode},
    game_init::PlayerEntity,
    inventory::UseMenuResult,
    map::Map,
    CL_INTERACTABLES, CL_TEXT, CL_WORLD,
};

pub const CLEAR: RGBA = RGBA { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

pub fn debug_info(ctx: &mut BTerm, ecs: &World, cfg: &InventoryConfig) {
    draw_interaction_mode(ctx, ecs);
    draw_inventory_state(ctx, ecs, cfg);
    draw_health(ctx, ecs);
}

fn draw_health(ctx: &mut BTerm, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let health_stats = ecs.read_storage::<HealthStats>();
    if let Some(stats) = health_stats.get(player_entity.0) {
        ctx.printer(2, 6, format!("#[white]{}/{}#[]", stats.hp, stats.max_hp), TextAlign::Left, None);
    }
}

// NOTE: This may be better in user interface once we figure out a cool way to display it, maybe an
// icon?
fn draw_interaction_mode(ctx: &mut BTerm, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let interactors = ecs.read_storage::<Interactor>();
    let player_mode = match interactors.get(player_entity.0) {
        Some(p) => p.mode.to_string(),
        None => "#[red]Mode Missing#[]".to_string(),
    };
    ctx.set_active_console(CL_TEXT);
    ctx.printer(
        1,
        50,
        format!("#[white]> {} <#[]", player_mode),
        TextAlign::Left,
        Some(RGB::from_u8(61, 84, 107).into()),
    );
}

fn draw_inventory_state(ctx: &mut BTerm, ecs: &World, cfg: &InventoryConfig) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let selected_idxs = ecs.read_storage::<SelectedInventoryItem>();
    let selection_status = match selected_idxs.get(player_entity.0) {
        Some(selection) => {
            let message = match &selection.intended_action {
                Some(action) => match action {
                    UseMenuResult::Drop => "Drop",
                    UseMenuResult::Craft => "Craft",
                    UseMenuResult::Equip => "Equip",
                    UseMenuResult::Cancel => "Cancel",
                    UseMenuResult::Consume => "Consume",
                    UseMenuResult::Examine => "Examine",
                }
                .to_string(),
                None => "none".to_string(),
            };

            let items: ReadStorage<Item> = ecs.read_storage();
            let inbags: ReadStorage<InBag> = ecs.read_storage();
            let names: ReadStorage<Name> = ecs.read_storage();
            let entities = ecs.entities();
            match (&entities, &items, &inbags, &names)
                .join()
                .filter(|inv_item| inv_item.2.owner == player_entity.0)
                .sorted_by(|a, b| match cfg.sort_mode {
                    SortMode::NameABC => a.3.cmp(b.3),
                    SortMode::IDAsc => a.1.id.cmp(&b.1.id),
                    _ => a.1.id.cmp(&b.1.id),
                })
                .position(|inv_item| inv_item.0 == selection.first_item)
            {
                Some(idx_selected) => format!("Selected: {} | Action: {}", idx_selected, message),
                None => "Inventory selection index mismatch".to_string(),
            }
        }
        None => "No selection made".to_string(),
    };
    ctx.set_active_console(CL_TEXT);
    ctx.print_color(1, 49, WHITESMOKE, RGB::from_u8(61, 84, 107), selection_status);

    let sort_mode = cfg.sort_mode.to_string();
    ctx.print_color(41, 49, WHITESMOKE, RGB::from_u8(61, 84, 107), sort_mode);
}

pub fn debug_input(ctx: &mut BTerm, ecs: &World) {
    if !ctx.control {
        return;
    }
    // All controls past this point require CTRL to be held. ================
    draw_cursor(ctx);

    if ctx.left_click {
        print_tile_contents(ctx, ecs);
    }

    if ctx.key.is_some() && ctx.key == Some(VirtualKeyCode::V) {
        print_position(ecs);
    }
}

fn draw_cursor(ctx: &mut BTerm) {
    ctx.set_active_console(CL_INTERACTABLES);
    ctx.printer(
        ctx.mouse_pos().0,
        ctx.mouse_pos().1,
        format!("#[white]{}#[]", to_char(254)),
        TextAlign::Left,
        Some(CLEAR),
    );
}

fn print_position(ecs: &World) {
    let positions = ecs.read_storage::<Position>();
    let transforms = ecs.read_storage::<Transform>();

    for (pos, fpos) in (&positions, &transforms).join() {
        println!("Position: {} || FancyPos: {:?}", pos, fpos.sprite_pos);
    }
}

fn print_tile_contents(ctx: &mut BTerm, ecs: &World) {
    let map = ecs.read_resource::<Map>();
    ctx.set_active_console(CL_WORLD);
    print!("MousePos on CL_WORLD: {:?} | ", &ctx.mouse_pos());

    let cursor_map_pos = mouse_to_map_pos(&ctx.mouse_pos(), ecs);

    let tile_idx = match cursor_map_pos {
        Some(pos) => pos.to_idx(map.width),
        None => {
            println!("Cannot print tile entities at {:?}", &cursor_map_pos);
            return;
        }
    };

    print!("Tileidx {} | ", tile_idx);
    if !map.tile_entities[tile_idx].is_empty() {
        println!("Contents: {:?} | BLOCKED: {}", map.tile_entities[tile_idx], map.is_blocked(&cursor_map_pos.unwrap()),);
    } else {
        println!("There are no entities at {:?}", cursor_map_pos);
    }
}
