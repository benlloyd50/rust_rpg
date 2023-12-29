use bracket_lib::{
    pathfinding::{field_of_view_set, Algorithm2D, BaseMap},
    prelude::{Point, SmallVec},
    terminal::{to_char, ColorPair, DistanceAlg, DrawBatch},
};
use specs::{Join, Read, ReadStorage, System, World, WorldExt, WriteStorage};

use crate::{
    camera::get_camera_bounds,
    colors::{DARKBLUE, WHITE},
    components::{Position, Viewshed},
    game_init::PlayerEntity,
    indexing::idx_to_point,
    map::{Map, MapRes},
    CL_EFFECTS,
};

pub struct UpdateViewsheds;

impl<'a> System<'a> for UpdateViewsheds {
    type SystemData = (WriteStorage<'a, Viewshed>, ReadStorage<'a, Position>, Read<'a, MapRes>);

    fn run(&mut self, (mut viewsheds, positions, map): Self::SystemData) {
        for (view, pos) in (&mut viewsheds, &positions).join() {
            view.tiles = field_of_view_set(pos.to_point(), view.range as i32, &map.0);
        }
    }
}

pub fn draw_unseen_area(draw_batch: &mut DrawBatch, ecs: &World) {
    let player_e = ecs.read_resource::<PlayerEntity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let player_view = match viewsheds.get(player_e.0) {
        Some(v) => v,
        None => return,
    };

    draw_batch.target(CL_EFFECTS);
    let bounding_box = get_camera_bounds(ecs);
    for x in bounding_box.x1..bounding_box.x2 {
        for y in bounding_box.y1..bounding_box.y2 {
            if player_view.tiles.contains(&Point { x, y }) {
                continue;
            }

            let screen_x = x - bounding_box.x1;
            let screen_y = y - bounding_box.y1;
            draw_batch.set(Point::new(screen_x, screen_y), ColorPair::new(WHITE, DARKBLUE), to_char(34));
        }
    }
}

// Trait impls for bracket pathfinding

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        !self.tiles[idx].transparent
    }

    fn get_available_exits(&self, starting_idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let Point { x, y } = idx_to_point(starting_idx, self.width);

        let mut exits = SmallVec::new();
        let valid_directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        for &(dx, dy) in &valid_directions {
            let new_x = x + dx;
            let new_y = y + dy;

            // Check if the new position is within bounds and not blocked
            if self.in_bounds(Point::new(new_x, new_y)) {
                let new_pos = Position::new(new_x as usize, new_y as usize);
                if !self.is_blocked(&new_pos) {
                    let new_idx = new_pos.to_idx(self.width);
                    exits.push((new_idx, self.get_pathing_distance(starting_idx, new_idx)));
                }
            }
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Pythagoras.distance2d(self.index_to_point2d(idx1), self.index_to_point2d(idx2))
    }
}

impl Algorithm2D for Map {
    fn point2d_to_index(&self, pt: Point) -> usize {
        pt.to_index(self.width)
    }

    fn index_to_point2d(&self, idx: usize) -> Point {
        idx_to_point(idx, self.width)
    }

    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }

    fn in_bounds(&self, pos: Point) -> bool {
        let bounds = self.dimensions();
        pos.x >= 0 && pos.x < bounds.x && pos.y >= 0 && pos.y < bounds.y
    }
}
