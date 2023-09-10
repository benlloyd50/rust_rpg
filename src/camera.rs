use bracket_terminal::prelude::{Point, Rect};
use specs::{Join, World, WorldExt};

use crate::{components::Position, map::Map, player::Player, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const PLAYER_CAMERA_OFFSET_X: i32 = 13;
const PLAYER_CAMERA_OFFSET_Y: i32 = 13;

/// Gets the bounds of the camera positioned at the player.
pub fn get_camera_bounds(ecs: &World) -> Rect {
    let player = ecs.read_storage::<Player>();
    let positions = ecs.read_storage::<Position>();

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

/// Tries to transform the position of the cursor into a position on the map.
/// Will return None if the mouse is outside the bounds of the map relative to the camera on the
/// player.
pub fn mouse_to_map_pos(mouse_pos: &(i32, i32), ecs: &World) -> Option<Position> {
    let bounds = get_camera_bounds(ecs);

    let map = ecs.read_resource::<Map>();
    let tile_pos = Point::new(bounds.x1 + mouse_pos.0, bounds.y1 + mouse_pos.1);

    if !map.in_bounds(tile_pos) {
        eprintln!("{:?} is outside of the map", mouse_pos);
        return None;
    }

    Some(Position::from(tile_pos))
}
