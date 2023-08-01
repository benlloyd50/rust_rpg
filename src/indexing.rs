/* Indexing.rs
 *   Contains the many systems that index entities or tiles in order to update the index when
 *   needed.
 * */

use specs::{Join, System, WriteExpect, ReadStorage, Entities};

use crate::{components::{Fishable, Position, Breakable, Blocking}, map::{Map, TileEntity}};

/// Clears the entity contents of every tile in the map
pub struct IndexReset;

impl<'a> System<'a> for IndexReset {
    type SystemData = (WriteExpect<'a, Map>,);

    fn run(&mut self, (mut map,): Self::SystemData) {
        for content in map.tile_entities.iter_mut() {
            content.clear();
        }
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
            map.tile_entities[idx].push(TileEntity::Blocking);
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
            map.tile_entities[idx].push(TileEntity::Breakable(id));
        }
    }
}

pub struct IndexFishableTiles;

impl<'a> System<'a> for IndexFishableTiles {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Fishable>,
        Entities<'a>,
    );

    fn run(&mut self, (mut map, pos, fishable, entities): Self::SystemData) {
        for (entity, pos, _) in (&entities, &pos, &fishable).join() {
            let idx = map.xy_to_idx(pos.x, pos.y);
            map.tile_entities[idx].push(TileEntity::Fishable(entity));
        }
    }
}

