use bracket_terminal::prelude::{to_char, BTerm, TextAlign, VirtualKeyCode, RGB, RGBA, WHITESMOKE};
use specs::{Join, World, WorldExt};

use crate::{
    camera::mouse_to_map_pos,
    components::{Interactor, Position, SelectedInventoryIdx, Transform},
    game_init::PlayerEntity,
    inventory::UseMenuResult,
    map::Map,
    CL_INTERACTABLES, CL_TEXT, CL_WORLD,
};

const CLEAR: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

pub fn debug_info(ctx: &mut BTerm, ecs: &World) {
    draw_interaction_mode(ctx, ecs);
    draw_inventory_state(ctx, ecs);
}

// NOTE: This may be better in user interface once we figure out a cool way to display it, maybe an
// icon?
fn draw_interaction_mode(ctx: &mut BTerm, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let interactors = ecs.read_storage::<Interactor>();
    let player_interactor = match interactors.get(player_entity.0) {
        Some(p) => p,
        None => {
            eprintln!("Player entity does not have interactor component");
            return;
        }
    };
    let previous_active = ctx.active_console;
    ctx.set_active_console(CL_TEXT);
    ctx.print_color(
        1,
        50,
        WHITESMOKE,
        RGB::from_u8(61, 84, 107),
        format!("> {} <", player_interactor.mode),
    );
    ctx.set_active_console(previous_active);
}

fn draw_inventory_state(ctx: &mut BTerm, ecs: &World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let selected_idxs = ecs.read_storage::<SelectedInventoryIdx>();
    let selection_status = match selected_idxs.get(player_entity.0) {
        Some(selection) => {
            let message = match &selection.intended_action {
                Some(action) => match action {
                    UseMenuResult::Drop => "Drop",
                    UseMenuResult::Craft => "Craft",
                    UseMenuResult::Cancel => "Cancel",
                    UseMenuResult::Examine => "Examine",
                }
                .to_string(),
                None => "none".to_string(),
            };
            format!("Selected: {} | Action: {}", selection.first_idx, message)
        }
        None => "No selection made".to_string(),
    };
    let previous_active = ctx.active_console;
    ctx.set_active_console(CL_TEXT);
    ctx.print_color(
        1,
        49,
        WHITESMOKE,
        RGB::from_u8(61, 84, 107),
        selection_status,
    );
    ctx.set_active_console(previous_active);
}

pub fn debug_input(ctx: &mut BTerm, ecs: &World) {
    if !ctx.control {
        return;
    }

    draw_sprite_under_cursor(ctx);

    if ctx.left_click {
        print_tile_contents(ctx, ecs);
    }

    if ctx.key.is_some() {
        match ctx.key.unwrap() {
            VirtualKeyCode::V => {
                print_position(&ecs);
            }
            _ => {}
        }
    }
}

fn draw_sprite_under_cursor(ctx: &mut BTerm) {
    let previous_active = ctx.active_console;
    ctx.set_active_console(CL_INTERACTABLES);
    ctx.printer(
        ctx.mouse_pos().0,
        ctx.mouse_pos().1,
        format!("#[white]{}#[]", to_char(254)),
        TextAlign::Left,
        Some(CLEAR),
    );
    ctx.set_active_console(previous_active);
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

    let cursor_map_pos = mouse_to_map_pos(&ctx.mouse_pos(), &ecs);

    let tile_idx = match cursor_map_pos {
        Some(pos) => pos.to_idx(map.width),
        None => {
            println!("Cannot print tile entities at {:?}", &cursor_map_pos);
            return;
        }
    };

    print!("Tileidx {} | ", tile_idx);
    if !map.tile_entities[tile_idx].is_empty() {
        println!(
            "Contents: {:?} | BLOCKED: {}",
            map.tile_entities[tile_idx],
            map.is_blocked(&cursor_map_pos.unwrap()),
        );
    } else {
        println!("There are no entities at {:?}", cursor_map_pos);
    }
}
