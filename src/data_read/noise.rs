use lazy_static::lazy_static;
use log::{error, info, warn};
use serde::Deserialize;
use std::fs;
use std::sync::Mutex;

use bracket_lib::{
    noise::{FastNoise, NoiseType},
    prelude::PointF,
};

use crate::noise::{Noise, RawWorldTile};

lazy_static! {
    pub static ref NOISE_DB: Mutex<NoiseDatabase> = Mutex::new(NoiseDatabase::empty());
}

const NOISE_PATH: &str = "raws/noise.json5";

pub struct NoiseDatabase {
    pub noises: Vec<Noise>,
}

#[derive(Deserialize)]
struct RawNoise {
    pub name: String,
    pub noise_type: Option<String>,
    pub octaves: Option<i32>,
    pub frequency: Option<f32>,
    pub lacunarity: Option<f32>,
    pub gain: Option<f32>,
    pub scale: Option<Vec<f32>>,
    pub tile_mapping: Option<Vec<RawWorldTile>>,
}

impl NoiseDatabase {
    pub fn empty() -> Self {
        NoiseDatabase { noises: Vec::new() }
    }

    pub fn load(&mut self) {
        let mut noise_db = Self::empty();
        let raw_noises =
            fs::read_to_string(NOISE_PATH).unwrap_or_else(|_| panic!("Failed to read noise file: {}", NOISE_PATH));
        let raw_noises: Vec<RawNoise> =
            json5::from_str(&raw_noises).unwrap_or_else(|_| panic!("Failed to parse noise file: {}", NOISE_PATH));

        for noise in raw_noises {
            let mut parsed = Noise { name: noise.name, scale: PointF::one(), noise: FastNoise::new(), mapping: vec![] };

            if let Some(noise_type) = noise.noise_type {
                match noise_type.as_str() {
                    "perlin" => parsed.noise.set_noise_type(NoiseType::Perlin),
                    "simplex" => parsed.noise.set_noise_type(NoiseType::Simplex),
                    "cellular" => parsed.noise.set_noise_type(NoiseType::Cellular),
                    _ => error!("Invalid noise type: {}", noise_type),
                }
            }

            if let Some(octaves) = noise.octaves {
                parsed.noise.set_fractal_octaves(octaves);
            }

            if let Some(frequency) = noise.frequency {
                parsed.noise.set_frequency(frequency);
            }

            if let Some(lacunarity) = noise.lacunarity {
                parsed.noise.set_fractal_lacunarity(lacunarity);
            }

            if let Some(gain) = noise.gain {
                parsed.noise.set_fractal_gain(gain);
            }

            if let Some(scale) = noise.scale {
                if scale.len() != 2 {
                    warn!("Noise Parse: {} Scale does not have exactly 2 values resorting to default", parsed.name);
                } else {
                    parsed.scale = PointF::new(scale[0], scale[1]);
                }
            }

            if let Some(tilemap) = noise.tile_mapping {
                parsed.mapping = tilemap;
            }

            noise_db.noises.push(parsed);
        }

        *self = noise_db;
    }

    pub fn reseed(&mut self, seed: u64) {
        info!("seed for world set to {}", seed);
        for noise in &mut self.noises {
            noise.noise.set_seed(seed);
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Noise> {
        self.noises.iter().find(|n| n.name == name)
    }
}
