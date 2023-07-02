use specs::{prelude::*, Component};
use bracket_terminal::prelude::{VirtualKeyCode, BTerm, Point};
use std::process::exit;
use crate::{State, Position, DISPLAY_WIDTH, DISPLAY_HEIGHT};

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Player {}

pub fn manage_player_input(state: &mut State, ctx: &BTerm) {
    match ctx.key {
        None => {},
        Some(key) => {
            match key  {
                VirtualKeyCode::W | VirtualKeyCode::Up => try_move_player(0, -1, &mut state.ecs),
                VirtualKeyCode::S | VirtualKeyCode::Down => try_move_player(0, 1, &mut state.ecs),
                VirtualKeyCode::A | VirtualKeyCode::Left => try_move_player(-1, 0, &mut state.ecs),
                VirtualKeyCode::D | VirtualKeyCode::Right => try_move_player(1, 0, &mut state.ecs),
                VirtualKeyCode::Escape => {exit(0)}
                _ => {}
            }
        }
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();

    for (pos, _) in (&mut positions, &players).join() {
        let target_pos = Point::new(pos.x as i32 + delta_x, pos.y as i32 + delta_y);
        
        // check target_pos is a valid position to move into (in map bounds, not blocked by another entity or tile)
        if target_pos.x < 0 || target_pos.y < 0 || target_pos.x >= DISPLAY_WIDTH as i32 || target_pos.y >= DISPLAY_HEIGHT as i32 {
            return;
        }

        pos.x = target_pos.x as usize;
        pos.y = target_pos.y as usize;
    }
}