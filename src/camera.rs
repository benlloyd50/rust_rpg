use bracket_terminal::prelude::Rect;
use specs::{Join, World, WorldExt};

use crate::{components::Position, player::Player, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const PLAYER_CAMERA_OFFSET_X: i32 = 13;
const PLAYER_CAMERA_OFFSET_Y: i32 = 13;

// returns the corners of the rectangle
pub fn get_bounding_box(ecs: &World) -> Rect {
    let player = ecs.read_storage::<Player>();
    let positions = ecs.read_storage::<Position>();
    // let map = ecs.read_resource::<Map>();

    if let Some((pos, _player)) = (&positions, &player).join().next() {
        let x_offset = pos.x as i32 - PLAYER_CAMERA_OFFSET_X;
        let y_offset = pos.y as i32 - PLAYER_CAMERA_OFFSET_Y;
        Rect::with_size(
            x_offset,
            y_offset,
            DISPLAY_WIDTH as i32,
            DISPLAY_HEIGHT as i32,
        )
    } else {
        Rect::with_size(
            PLAYER_CAMERA_OFFSET_X,
            PLAYER_CAMERA_OFFSET_Y,
            PLAYER_CAMERA_OFFSET_X + DISPLAY_WIDTH as i32,
            PLAYER_CAMERA_OFFSET_Y + DISPLAY_HEIGHT as i32,
        )
    }
}
