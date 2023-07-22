use crate::{
    map::{Map, TileEntity},
    Position, State, DISPLAY_HEIGHT, DISPLAY_WIDTH, components::BreakAction,
};
use bracket_terminal::prelude::{BTerm, Point, VirtualKeyCode};
use specs::{prelude::*, Component};
use std::process::exit;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Player {}

pub fn manage_player_input(state: &mut State, ctx: &BTerm) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => try_move_player(0, -1, &mut state.ecs),
            VirtualKeyCode::S | VirtualKeyCode::Down => try_move_player(0, 1, &mut state.ecs),
            VirtualKeyCode::A | VirtualKeyCode::Left => try_move_player(-1, 0, &mut state.ecs),
            VirtualKeyCode::D | VirtualKeyCode::Right => try_move_player(1, 0, &mut state.ecs),
            VirtualKeyCode::Escape => exit(0),
            _ => {}
        },
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut break_actions = ecs.write_storage::<BreakAction>();

    for (entity, pos, _) in (&entities, &mut positions, &players).join() {
        let target_pos = Point::new(pos.x as i32 + delta_x, pos.y as i32 + delta_y);

        // check target_pos is a valid position to move into (in map bounds, not blocked by another entity or tile)
        if target_pos.x < 0
            || target_pos.y < 0
            || target_pos.x >= DISPLAY_WIDTH as i32
            || target_pos.y >= DISPLAY_HEIGHT as i32
        {
            return;
        }

        let target_idx = map.xy_to_idx(target_pos.x as usize, target_pos.y as usize);
        match map.tile_entity[target_idx] {
            TileEntity::Blocking => {
                println!("Map is blocked at {}, {}", target_pos.x, target_pos.y);
                return;
            }
            TileEntity::Breakable(id) => {
                println!(
                    "Map is breakable at {}, {} : id: {}",
                    target_pos.x,
                    target_pos.y,
                    id.gen().id()
                );
                break_actions.insert(entity, BreakAction{target: id}).expect("Player couldn't break");
                return;
            }
            TileEntity::Empty => {
                pos.x = target_pos.x as usize;
                pos.y = target_pos.y as usize;
            }
        }
    }
}
