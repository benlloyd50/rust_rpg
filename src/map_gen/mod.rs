use std::hash::{DefaultHasher, Hash, Hasher};

use bracket_lib::random::RandomNumberGenerator;
use log::error;
use specs::{Builder, World, WorldExt};

mod gameworld;

use crate::{
    components::{Blocking, Position, Water},
    data_read::prelude::{build_world_obj, NOISE_DB},
    game_init::InputWorldConfig,
    indexing::idx_to_point,
    map::{sqrt_distance, Map, WorldCoords, WorldTile},
    saveload::{save_game_exists, SAVE_EXTENSION},
    FONT_TERRAIN_FOREST,
};

pub mod prelude {
    pub use crate::map_gen::gameworld::{generate_world, get_random_chunk, ChunkType, GameWorld, GameWorldRes};
}

#[derive(Clone, PartialEq, Eq, Debug)]
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
pub fn generate_map(ecs: &mut World, wc: &WorldConfig, map_idx: usize) -> Map {
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
    remove_land_outside_circle(&mut new_map);
    fill_water_to_level(&mut new_map, wc.sea_level, ecs);
    generate_resources(&mut new_map, ecs, &mut rng);

    let pt = idx_to_point(map_idx, new_map.width);
    new_map.chunk_coords =
        WorldCoords { x: if pt.x >= 0 { pt.x as usize } else { 0 }, y: if pt.y >= 0 { pt.y as usize } else { 0 } };

    new_map
}

fn remove_land_outside_circle(map: &mut Map) {
    // a circle with a radius of 30 centered in the center of the map
    let center = Position::new(map.width / 2, map.height / 2);
    let radius: f32 = 30.;
    for x in 0..map.width {
        for y in 0..map.height {
            // if point is outside circle set it to water
            let dist_from_center = sqrt_distance(&center, &Position::new(x, y));
            if dist_from_center > radius {
                let prev_height = map.tiles.get(map.xy_to_idx(x, y)).unwrap().height;
                map.set_tile(&WorldTile::water(prev_height), x, y);
            }
        }
    }
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
                let check = rng.range(0.0, 1.0);
                if check > weight {
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
