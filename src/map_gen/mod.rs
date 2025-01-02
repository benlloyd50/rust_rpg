use bracket_lib::noise::FastNoise;
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

    // just a random boulder
    let _ = build_world_obj("Boulder".to_string(), Position::new(15, 20), ecs);

    generate_forest_terrain(&mut new_map);

    new_map
}

fn generate_forest_terrain(map: &mut Map) {
    let mut noise = FastNoise::new();
    noise.set_fractal_octaves(4);

    for x in 0..map.width {
        for y in 0..map.height {
            let value = noise.get_noise(x as f32, y as f32);

            let tile = if value > 0.5 {
                WorldTile { atlas_index: 0, transparent: true }
            } else if value > 0.5 {
                WorldTile { atlas_index: 2, transparent: true }
            } else {
                WorldTile { atlas_index: 4, transparent: true }
            };

            map.set_tile(tile, x, y);
        }
    }
}
