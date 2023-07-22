use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, BLACK, WHITE};
use specs::{Entities, Entity, Join, ReadStorage, System, World, WriteExpect};

use crate::components::{Blocking, Breakable, Position};

pub struct Map {
    tiles: Vec<WorldTile>,
    pub tile_entity: Vec<TileEntity>,
    width: usize,
    height: usize,
}

#[derive(Clone)]
struct WorldTile {
    atlas_index: usize,
}

/// Defines the type of entity existing in a tile for quick lookup and action handling
/// This limits tile entities to be one of these types and cannot be two.
#[derive(Clone)]
pub enum TileEntity {
    Breakable(Entity),
    Blocking,
    Empty,
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
            // blocked: vec![false; width * height],
            tile_entity: vec![TileEntity::Empty; width * height],
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
pub struct IndexReset;

impl<'a> System<'a> for IndexReset {
    type SystemData = (WriteExpect<'a, Map>,);

    fn run(&mut self, (mut map, ): Self::SystemData) {
        map.tile_entity.fill(TileEntity::Empty);
    }
}

pub struct IndexBlockedTiles;

impl<'a> System<'a> for IndexBlockedTiles {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Blocking>,
    );

    fn run(&mut self, (mut map, pos, blocking): Self::SystemData) {
        for (pos, _) in (&pos, &blocking).join() {
            let idx = map.xy_to_idx(pos.x, pos.y);
            map.tile_entity[idx] = TileEntity::Blocking;
        }
    }
}

pub struct IndexBreakableTiles;

impl<'a> System<'a> for IndexBreakableTiles {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Breakable>,
        Entities<'a>,
    );

    fn run(&mut self, (mut map, pos, breakable, entities): Self::SystemData) {
        for (id, pos, _) in (&entities, &pos, &breakable).join() {
            let idx = map.xy_to_idx(pos.x, pos.y);
            map.tile_entity[idx] = TileEntity::Breakable(id);
        }
    }
}
