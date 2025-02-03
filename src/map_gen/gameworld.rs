use std::collections::HashMap;

use bracket_lib::random::RandomNumberGenerator;

use crate::map::{xy_to_idx_given_width, Map};

use super::WorldConfig;

/* 1. generate shape of world of what is water or land
 *
 */

#[derive(Clone, Default, Debug)]
pub struct GameWorld {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<WorldChunk>,

    pub generated: HashMap<usize, Map>,

    pub world_config: WorldConfig,
}

impl GameWorld {
    pub fn new(wc: WorldConfig) -> Self {
        Self {
            width: wc.width,
            height: wc.height,
            grid: vec![WorldChunk::default(); wc.width * wc.height],
            generated: HashMap::new(),
            world_config: wc,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WorldChunk {
    pub chunk_type: ChunkType,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum ChunkType {
    Land,
    #[default]
    Water,
}

#[derive(Default, Clone)]
pub struct GameWorldRes(pub GameWorld);

pub fn generate_world(wc: &WorldConfig) -> GameWorld {
    let mut game_world = GameWorld::new(wc.clone());
    let mut rng = RandomNumberGenerator::seeded(wc.seed);

    for x in 0..wc.width {
        for y in 0..wc.height {
            let rand_val = rng.range(0.0, 1.0);
            game_world.grid[xy_to_idx_given_width(x, y, game_world.width)] = if rand_val < 0.5 {
                WorldChunk { chunk_type: ChunkType::Water }
            } else {
                WorldChunk { chunk_type: ChunkType::Land }
            }
        }
    }

    game_world
}

pub fn get_random_chunk(gw: &GameWorld, seed: u64, chunk_type: ChunkType) -> usize {
    let mut rng = RandomNumberGenerator::seeded(seed);
    let mut chunk: Option<usize> = None;
    while chunk.is_none() {
        let x = rng.range(0, gw.width);
        let y = rng.range(0, gw.height);
        let idx = xy_to_idx_given_width(x, y, gw.width);
        let tile = gw.grid[idx].chunk_type;
        if tile == chunk_type {
            chunk = Some(idx);
        }
    }
    chunk.unwrap_or(0)
}
