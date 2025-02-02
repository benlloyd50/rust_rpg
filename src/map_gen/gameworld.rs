use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::map::xy_to_idx_given_width;

use super::WorldConfig;

/* 1. generate shape of world of what is water or land
 *
 */

#[derive(Clone, Default, Debug)]
pub struct GameWorld {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<WorldChunk>,
}

impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, grid: vec![WorldChunk::default(); width * height] }
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
    let mut game_world = GameWorld::new(wc.width, wc.height);
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

pub fn get_random_chunk(ecs: &mut World, rng: &mut RandomNumberGenerator, chunk_type: ChunkType) -> usize {
    let mut chunk: Option<usize> = None;
    if let Some(gw) = ecs.get_mut::<GameWorldRes>() {
        while chunk.is_none() {
            let x = rng.range(0, gw.0.width);
            let y = rng.range(0, gw.0.height);
            let idx = xy_to_idx_given_width(x, y, gw.0.width);
            let tile = gw.0.grid[idx].chunk_type;
            if tile == chunk_type {
                chunk = Some(idx);
            }
        }
    }
    chunk.unwrap_or(0)
}
