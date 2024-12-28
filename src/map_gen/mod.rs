use specs::World;

use crate::{
    components::Position,
    data_read::prelude::build_world_obj,
    map::{Map, WorldTile},
    FONT_TERRAIN_FOREST,
};

pub struct WorldConfig {
    pub width: usize,
    pub height: usize,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { width: 100, height: 100 }
    }
}

// Generates a map and populates ecs with relavent tiles
pub fn gen_world(ecs: &mut World, wc: &WorldConfig) -> Map {
    let mut new_map = Map::new(wc.width, wc.height, (0, 0));

    new_map.tile_atlas_index = FONT_TERRAIN_FOREST;
    for x in 0..wc.width {
        for y in 0..wc.height {
            new_map.set_tile(WorldTile::default(), x, y);
        }
    }
    let _ = build_world_obj("Boulder".to_string(), Position::new(15, 20), ecs);

    new_map
}
