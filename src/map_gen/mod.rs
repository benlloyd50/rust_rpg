use std::hash::{DefaultHasher, Hash, Hasher};

use specs::World;

use crate::{
    components::Position,
    data_read::prelude::{build_world_obj, NOISE_DB},
    game_init::InputWorldConfig,
    map::{Map, WorldTile},
    saveload::{save_game_exists, SAVE_EXTENSION},
    FONT_TERRAIN_FOREST,
};

#[derive(Clone, PartialEq, Eq)]
pub struct WorldConfig {
    pub world_name: String,
    pub width: usize,
    pub height: usize,
    pub seed: u64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { world_name: "".to_string(), width: 100, height: 100, seed: 0 }
    }
}

impl WorldConfig {
    pub fn try_from(iwc: &InputWorldConfig) -> Result<Self, Vec<String>> {
        let mut errors = vec![];

        if iwc.world_name.is_empty() {
            errors.push("World name cannot be empty".to_string());
        }
        if save_game_exists(&format!("{}.{}", iwc.world_name, SAVE_EXTENSION)) {
            errors.push("World name already exists".to_string());
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
        let seed = if iwc.seed.is_empty() {
            0
        } else {
            let mut hasher = DefaultHasher::new();
            iwc.seed.hash(&mut hasher);
            hasher.finish()
        };

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self { world_name: iwc.world_name.clone(), width, height, seed })
    }
}

// Generates a map and populates ecs with relavent objects and world things
pub fn gen_world(ecs: &mut World, wc: &WorldConfig) -> Map {
    {
        let mut noise_db = NOISE_DB.lock().unwrap();
        noise_db.reseed(wc.seed);
    }

    let mut new_map = Map::new(wc.width, wc.height, (0, 0));

    new_map.tile_atlas_index = FONT_TERRAIN_FOREST;
    for x in 0..wc.width {
        for y in 0..wc.height {
            new_map.set_tile(&WorldTile::default(), x, y);
        }
    }

    // just a random boulder
    let _ = build_world_obj("Boulder".to_string(), Position::new(15, 20), ecs);

    generate_forest_terrain(&mut new_map);

    new_map
}

fn generate_forest_terrain(map: &mut Map) {
    let noise_db = NOISE_DB.lock().unwrap();
    let noise = noise_db.get_by_name("forest").unwrap();

    for x in 0..map.width {
        for y in 0..map.height {
            let world_tile = noise.gen_tile(x, y);
            map.set_tile(&world_tile, x, y);
        }
    }
}
