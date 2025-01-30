use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::map::xy_to_idx_given_width;

use super::WorldConfig;

/* 1. generate shape of world of what is water or land
 *
 */

#[derive(Clone, Default)]
pub struct GameWorld {
    width: usize,
    height: usize,
    grid: Vec<WorldChunk>,
}

impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, grid: vec![WorldChunk::default(); width * height] }
    }
}

#[derive(Default, Clone, Copy)]
pub struct WorldChunk {
    chunk_type: ChunkType,
}

#[derive(Default, Clone, Copy)]
pub enum ChunkType {
    Land,
    #[default]
    Water,
}

#[derive(Default, Clone)]
pub struct GameWorldRes(pub GameWorld);

pub fn generate_world(ecs: &mut World, wc: &WorldConfig) {
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

    ecs.insert(GameWorldRes(game_world));
}
