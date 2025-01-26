use bracket_lib::{noise::FastNoise, prelude::PointF};
use pathfinding::num_traits::Signed;
use serde::Deserialize;

use crate::map::WorldTile;

// `Noise` is a terrain generator with configured noise to create an endless pattern
// and tile mappings to specify each type of terrain
pub struct Noise {
    pub name: String,
    pub noise: FastNoise,
    pub scale: PointF,
    pub mapping: Vec<RawWorldTile>,
}

#[derive(Deserialize, Clone, PartialEq, Debug, Default)]
pub struct RawWorldTile {
    pub height: f32,
    pub name: String,
    pub atlas_idx: Option<usize>,
    pub is_blocked: Option<String>,
    pub weight: Option<f32>,
}

impl Noise {
    // Shorthand function to generate the tile based on the configuration of this world generator
    pub fn gen_tile(&self, x: usize, y: usize) -> WorldTile {
        let value = self.get_normal_2d(x as f32, y as f32);
        if let Some(tile) = self.find_tile_map(value) {
            let world_tile = WorldTile {
                name: tile.name,
                atlas_idx: tile.atlas_idx.unwrap(),
                height: (value * 255.0).round() as u8,
                is_blocked: tile.is_blocked.is_some(),
                ..Default::default()
            };
            return world_tile;
        }

        return WorldTile::default();
    }

    pub fn get_name_of(&self, x: usize, y: usize) -> Option<(String, f32)> {
        let value = self.get_normal_2d(x as f32, y as f32);
        if let Some(raw) = self.find_tile_map(value) {
            if let Some(weight) = raw.weight {
                return Some((raw.name, weight));
            }
        }
        None
    }

    // noise value in the bounds of 0 to 1
    pub fn get_normal_2d(&self, x: f32, y: f32) -> f32 {
        (self.noise.get_noise(x * self.scale.x, y * self.scale.y) + 1.0) * 0.5
    }

    // TODO: make unit tests for this
    pub fn find_tile_map(&self, value: f32) -> Option<RawWorldTile> {
        // need to find the id that the value > chance at the highest is true
        // value - chance ie value = 0.8, chance = 0.9, 0.75, 0.5, 0.25
        // -0.1, *0.05*, 0.3, 0.65
        // min [] > 0
        if let Some(tm) = &self
            .mapping
            .iter()
            .map(|tm| (tm, value - tm.height))
            .filter(|(_, diff)| diff.is_positive())
            .min_by(|(_, chance), (_, chance2)| chance.total_cmp(chance2))
        {
            return Some(tm.0.clone());
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
            scale: PointF::one(),
            noise: FastNoise::new(),
            mapping: vec![
                RawWorldTile { height: 0.5, atlas_idx: Some(0), ..Default::default() },
                RawWorldTile { height: 0.75, atlas_idx: Some(1), ..Default::default() },
                RawWorldTile { height: 1.0, atlas_idx: Some(2), ..Default::default() },
            ],
        };
        assert_eq!(noise.find_tile_map(0.5).unwrap().atlas_idx.unwrap(), 0);
    }

    #[test]
    fn test_tile_map_out_of_bounds() {
        let noise = Noise {
            name: "test".to_string(),
            scale: PointF::one(),
            noise: FastNoise::new(),
            mapping: vec![
                RawWorldTile { height: 0.5, atlas_idx: Some(0), ..Default::default() },
                RawWorldTile { height: 0.75, atlas_idx: Some(1), ..Default::default() },
                RawWorldTile { height: 1.0, atlas_idx: Some(2), ..Default::default() },
            ],
        };

        assert_eq!(noise.find_tile_map(0.2), None);
    }
}
