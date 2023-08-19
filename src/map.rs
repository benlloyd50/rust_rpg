use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, BLACK, WHITE};
use specs::{Entity, World};

use crate::camera::get_bounding_box;

pub struct Map {
    pub tiles: Vec<WorldTile>,
    pub tile_entities: Vec<Vec<TileEntity>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct WorldTile {
    pub atlas_index: usize,
}

/// Defines the type of entity existing in a tile for quick lookup and action handling
/// Discrimnants are the priority for action handling, lower taking priority
#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum TileEntity {
    Fishable(Entity) = 9,
    Breakable(Entity) = 15,
    Blocking = 20,
}

impl WorldTile {
    pub fn default() -> Self {
        Self { atlas_index: 4 }
    }
}

impl Map {
    pub fn empty() -> Self {
        Self {
            tiles: vec![],
            tile_entities: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        Map {
            tiles: vec![WorldTile::default(); width * height],
            tile_entities: vec![vec![]; width * height],
            width,
            height,
        }
    }

    pub fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

/// Renders the current map resource to the current console layer
pub fn render_map(ecs: &World, batch: &mut DrawBatch) {
    let map = ecs.fetch::<Map>();

    let bounding_box = get_bounding_box(ecs);

    for x in bounding_box.x1..bounding_box.x2 {
        for y in bounding_box.y1..bounding_box.y2 {
            let atlas_index = if x < map.width as i32 && y < map.height as i32 && x >= 0 && y >= 0 {
                map.tiles[map.xy_to_idx(x as usize, y as usize)].atlas_index
            } else {
                xy_to_idx_given_width(0, 2, 16)
            };

            let batch_x = if x > 0 {
                x + (-bounding_box.x1)
            } else {
                x - bounding_box.x1
            };
            let batch_y = if y > 0 {
                y + (-bounding_box.y1)
            } else {
                y - bounding_box.y1
            };

            batch.set(
                Point::new(batch_x, batch_y),
                ColorPair::new(WHITE, BLACK),
                atlas_index,
            );
        }
    }
}

fn xy_to_idx_given_width(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}
