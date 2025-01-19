use specs::World;

use crate::{
    components::Position,
    data_read::prelude::build_world_obj,
    game_init::InputWorldConfig,
    map::{Map, WorldTile},
    FONT_TERRAIN_FOREST,
};

#[derive(Clone, PartialEq, Eq)]
pub struct WorldConfig {
    pub world_name: String,
    pub width: usize,
    pub height: usize,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { world_name: "".to_string(), width: 100, height: 100 }
    }
}

impl WorldConfig {
    pub fn try_from(iwc: &InputWorldConfig) -> Result<Self, Vec<String>> {
        let mut errors = vec![];

        if iwc.world_name.is_empty() {
            errors.push("World name cannot be empty".to_string());
        }

        let height = match iwc.height.parse::<usize>() {
            Ok(h) => h,
            Err(_) => {
                errors.push("Invalid height".to_string());
                0
            }
        };
        let width = match iwc.width.parse::<usize>() {
            Ok(w) => w,
            Err(_) => {
                errors.push("Invalid width".to_string());
                0
            }
        };

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self { world_name: iwc.world_name.clone(), width, height })
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
