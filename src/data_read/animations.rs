use std::{collections::HashMap, fs, sync::Mutex};

use bracket_color::prelude::PURPLE;
use log::error;
use serde::Deserialize;
use json5::from_str;

use crate::{frame_animation::Animation, colors::WHITE};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ANIMATION_DB: Mutex<AnimationDatabase> = Mutex::new(AnimationDatabase::new());
}

pub struct AnimationDatabase {
    animations: HashMap<String, Animation>,
}

impl AnimationDatabase {
    pub fn get_by_name(&self, name: &str) -> Option<&Animation> {
        self.animations.get(name)
    }
}

impl AnimationDatabase {
    pub fn new() -> Self {
        Self { animations: HashMap::new() }
    }

    pub fn load(&mut self) {
        let mut anim_db = Self::new();
        let data = match fs::read_to_string(ANIMATION_FILE) {
            Ok(d) => d,
            Err(e) => {
                error!("Error while reading animation file: {}", e);
                *self = anim_db;
                return; 
            }
        };

        let anims: Vec<RawAnimation> = match from_str(&data) {
            Ok(e) => e,
            Err(e) => {
                error!("Error while parsing animation file: {}", e);
                *self = anim_db;
                return;
            }
        };


        for anim in anims {
            let mut hash = HashMap::new();
            for (glyph, raw_color) in anim.key_colors {
                let value = match raw_color.as_str() {
                    "white" => WHITE,
                    _ => PURPLE
                };
                hash.insert(glyph, value);
            }
            anim_db
                .animations
                .insert(anim.name, Animation::from_vec(&anim.frames, &anim.frame_keys, &hash, anim.time_between_ms));
        }

        *self = anim_db;
    }
}

const ANIMATION_FILE: &str = "./raws/animations.json5";

#[derive(Deserialize)]
struct RawAnimation {
    name: String,
    time_between_ms: usize,
    // Special Defintions of characters in frame that will be replaced
    frame_keys: HashMap<char, u8>,
    // Colors for certain glyphs
    key_colors: HashMap<char, String>,
    frames: Vec<Vec<String>>,
}
