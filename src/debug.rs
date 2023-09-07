use bracket_terminal::prelude::{BTerm, TextAlign, to_char, RGBA};
use specs::WorldExt;

use crate::{camera::mouse_to_map_pos, map::Map, State, CL_WORLD, CL_INTERACTABLES};

const CLEAR: RGBA = RGBA { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

pub fn debug_input(ctx: &mut BTerm, state: &State) {
    if ctx.control {
        let previous_active = ctx.active_console;
        ctx.set_active_console(CL_INTERACTABLES);
        ctx.printer(ctx.mouse_pos().0, ctx.mouse_pos().1, format!("#[white]{}#[]", to_char(254)), TextAlign::Left, Some(CLEAR));
        ctx.set_active_console(previous_active);
    }
    if ctx.control && ctx.left_click {
        print_tile_contents(ctx, state);
    }
}

fn print_tile_contents(ctx: &mut BTerm, state: &State) {
    let map = state.ecs.read_resource::<Map>();
    ctx.set_active_console(CL_WORLD);
    print!("MousePos on CL_WORLD: {:?} | ", &ctx.mouse_pos());

    let cursor_map_pos = mouse_to_map_pos(&ctx.mouse_pos(), &state.ecs);

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
