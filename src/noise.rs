use bracket_lib::noise::FastNoise;
use pathfinding::num_traits::Signed;
use serde::Deserialize;

use crate::map::WorldTile;

// `Noise` is a terrain generator with configured noise to create an endless pattern
// and tile mappings to specify each type of terrain
pub struct Noise {
    pub name: String,
    pub noise: FastNoise,
    pub mapping: Vec<TileMapping>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct TileMapping {
    pub chance: f32,
    pub atlas_idx: usize,
}

pub struct Seed {
    pub seed: u64,
}

impl From<&str> for Seed {
    fn from(seed: &str) -> Self {
        Self { seed: seed.parse().unwrap_or(0) }
    }
}

impl Noise {
    // Shorthand function to generate the tile based on the configuration of this world generator
    pub fn gen_tile(&self, x: usize, y: usize) -> WorldTile {
        let value = self.get_normal_2d(x as f32, y as f32);
        if let Some(tile) = self.find_tile_map(value) {
            let world_tile = WorldTile { atlas_idx: tile.atlas_idx, ..Default::default() };
            return world_tile;
        }

        return WorldTile::default();
    }

    // noise value in the bounds of 0 to 1
    pub fn get_normal_2d(&self, x: f32, y: f32) -> f32 {
        (self.noise.get_noise(x, y) + 1.0) * 0.5
    }

    // TODO: make unit tests for this
    pub fn find_tile_map(&self, value: f32) -> Option<TileMapping> {
        // need to find the id that the value > chance at the highest is true
        // value - chance ie value = 0.8, chance = 0.9, 0.75, 0.5, 0.25
        // -0.1, *0.05*, 0.3, 0.65
        // min [] > 0
        if let Some(tm) = &self
            .mapping
            .iter()
            .map(|tm| (tm, value - tm.chance))
            .filter(|(_, diff)| diff.is_positive())
            .min_by(|(_, chance), (_, chance2)| chance.total_cmp(chance2))
        {
            return Some(*tm.0);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_find_tile_map() {
        let noise = Noise {
            name: "test".to_string(),
            noise: FastNoise::new(),
            mapping: vec![
                TileMapping { chance: 0.5, atlas_idx: 0 },
                TileMapping { chance: 0.75, atlas_idx: 1 },
                TileMapping { chance: 1.0, atlas_idx: 2 },
            ],
        };
        assert_eq!(noise.find_tile_map(0.5).unwrap().atlas_idx, 0);
    }

    #[test]
    fn test_tile_map_out_of_bounds() {
        let noise = Noise {
            name: "test".to_string(),
            noise: FastNoise::new(),
            mapping: vec![
                TileMapping { chance: 0.5, atlas_idx: 0 },
                TileMapping { chance: 0.75, atlas_idx: 1 },
                TileMapping { chance: 1.0, atlas_idx: 2 },
            ],
        };

        assert_eq!(noise.find_tile_map(0.2), None);
    }
}