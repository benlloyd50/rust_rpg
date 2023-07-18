use bracket_terminal::prelude::{ColorPair, DrawBatch, Point, BLACK, WHITE};
use specs::{World, System, WriteExpect, ReadStorage, Join};

use crate::components::{Blocking, Position};

pub struct Map {
    tiles: Vec<WorldTile>,
    pub blocked: Vec<bool>,
    width: usize,
    height: usize,
}

#[derive(Clone)]
struct WorldTile {
    atlas_index: usize,
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
            blocked: vec![false; width * height],
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

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Blocking>);
    
    fn run(&mut self, (mut map, pos, blocking): Self::SystemData) {
        map.blocked.fill(false);

        for (pos, _) in (&pos, &blocking).join() {
            let idx = map.xy_to_idx(pos.x, pos.y);
            map.blocked[idx] = true;
        }
    }

}
