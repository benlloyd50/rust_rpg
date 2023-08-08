use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, BLACK, WHITE};
use specs::{Entity, World};

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

    for x in 0..map.width {
        for y in 0..map.height {
            batch.set(
                Point::new(x, y),
                ColorPair::new(WHITE, BLACK),
                map.tiles[map.xy_to_idx(x, y)].atlas_index,
            );
        }
    }
}
