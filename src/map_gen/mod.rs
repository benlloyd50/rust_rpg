use std::hash::{DefaultHasher, Hash, Hasher};

use bracket_lib::random::RandomNumberGenerator;
use log::{error, info};
use specs::{Builder, World, WorldExt};

use crate::{
    components::{Blocking, Position, Water},
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
    pub sea_level: u8,
    pub seed: u64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            world_name: "".to_string(),
            width: 100,
            height: 100,
            sea_level: (0.13f32 * 255.0).round() as u8,
            seed: 0,
        }
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

        let sea_level = match iwc.sea_level.parse::<u8>() {
            Ok(h) => h,
            Err(_) => {
                errors.push("Invalid sea_level must be 0 - 255".to_string());
                0
            }
        };
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
        Ok(Self { world_name: iwc.world_name.clone(), width, height, seed, sea_level })
    }
}

// Generates a map and populates ecs with relavent objects and world things
pub fn gen_world(ecs: &mut World, wc: &WorldConfig) -> Map {
    {
        let mut noise_db = NOISE_DB.lock().unwrap();
        noise_db.reseed(wc.seed);
    }
    let mut rng = RandomNumberGenerator::seeded(wc.seed);

    let mut new_map = Map::new(wc.width, wc.height, (0, 0));

    new_map.tile_atlas_index = FONT_TERRAIN_FOREST;
    for x in 0..wc.width {
        for y in 0..wc.height {
            new_map.set_tile(&WorldTile::grass(), x, y);
        }
    }

    // just a random boulder
    // let _ = build_world_obj("Boulder".to_string(), Position::new(15, 20), ecs);

    generate_heights(&mut new_map);
    fill_water_to_level(&mut new_map, wc.sea_level, ecs);
    generate_resources(&mut new_map, ecs, &mut rng);

    new_map
}

fn generate_resources(map: &mut Map, ecs: &mut World, rng: &mut RandomNumberGenerator) {
    let noise_db = NOISE_DB.lock().unwrap();
    let r_noise = noise_db.get_by_name("resources").unwrap();

    for x in 0..map.width {
        for y in 0..map.height {
            let map_tile = &map.tiles[map.xy_to_idx(x, y)];
            if ["Mountain", "Water"].contains(&map_tile.name.as_str()) {
                continue;
            }

            if let Some((name, weight)) = r_noise.get_name_of(x, y) {
                let check = rng.rand::<u64>() as f32 / u64::MAX as f32;
                info!("{}", check);
                if check > weight {
                    info!("skipping tile");
                    continue;
                }

                match map_tile.name.as_str() {
                    "Grass" => {
                        if name == "Boulder" {
                            let check = rng.rand::<u64>() as f32 / u64::MAX as f32;
                            if check > 0.5 {
                                continue;
                            }
                        }
                    }
                    _ => {}
                }

                if let Err(e) = build_world_obj(name, Position::new(x, y), ecs) {
                    error!("Resources failed to build: {:?}", e);
                }
            }
        }
    }
}

fn fill_water_to_level(map: &mut Map, level: u8, ecs: &mut World) {
    for x in 0..map.width {
        for y in 0..map.height {
            if let Some(tile) = map.tiles.get(map.xy_to_idx(x, y)) {
                if tile.height < level {
                    map.set_tile(&WorldTile::water(tile.height), x, y);
                    ecs.create_entity().with(Water {}).with(Position::new(x, y)).with(Blocking {}).build();
                }
            }
        }
    }
}

fn generate_heights(map: &mut Map) {
    let noise_db = NOISE_DB.lock().unwrap();
    let noise = noise_db.get_by_name("height").unwrap();

    for x in 0..map.width {
        for y in 0..map.height {
            let world_tile = noise.gen_tile(x, y);
            map.set_tile(&world_tile, x, y);
        }
    }
}
