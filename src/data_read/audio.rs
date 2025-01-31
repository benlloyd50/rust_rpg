use crate::audio::SoundFiles;
use std::{collections::HashMap, fs, sync::Mutex};

use kira::{sound::static_sound::StaticSoundData, AudioManager, AudioManagerSettings, DefaultBackend};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use serde::Deserialize;

lazy_static! {
    pub static ref AUDIOMAN: Mutex<AudioPlayer> = Mutex::new(AudioPlayer::new());
    pub static ref AUDIO_DB: Mutex<AudioDatabase> = Mutex::new(AudioDatabase::new());
}

pub const AUDIO_DIRECTORY: &str = "./resources/sounds";
const AUDIO_DEFINTIONS_FILE: &str = "./raws/audio.json5";

pub struct AudioPlayer {
    pub player: Option<AudioManager>,
}

impl AudioPlayer {
    fn new() -> Self {
        Self {
            player: match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
                Ok(am) => Some(am),
                Err(e) => {
                    error!("Error setting up audio manager, music will NOT be played for this session of the game.");
                    error!("Internal audio loading error: {}", e);
                    info!("Since audio is glitched, you may want to try restarting/reinstalling to fix this issue.");
                    None
                }
            },
        }
    }
}

pub struct AudioDatabase {
    pub sounds: HashMap<String, SoundFiles>,
}

impl AudioDatabase {
    fn new() -> Self {
        Self { sounds: HashMap::new() }
    }

    pub fn load(&mut self) {
        let raw_audio_defs = match fs::read_to_string(AUDIO_DEFINTIONS_FILE) {
            Ok(ad) => ad,
            Err(e) => {
                warn!("Can't load the audio defintions, the audio database will be empty.");
                error!("Internal Reading Error: {}", e);
                return;
            }
        };

        let audio_defs: Vec<AudioDefinitions> = match json5::from_str(&raw_audio_defs) {
            Ok(ad) => ad,
            Err(e) => {
                warn!("Can't load the audio defintions, the audio database will be empty.");
                error!("Internal Parsing Error: {}", e);
                return;
            }
        };

        for def in audio_defs {
            let mut samples = vec![];
            let path = match def.directory {
                Some(dir) => format!("{}/{}/", AUDIO_DIRECTORY, dir),
                None => format!("{}/", AUDIO_DIRECTORY),
            };
            for file_name in def.file_names.iter() {
                let sound = match StaticSoundData::from_file(format!("{}/{}", path, file_name)) {
                    Ok(ssd) => ssd,
                    Err(e) => {
                        warn!("Error trying to read in {:?}. Skipping file.", path);
                        error!("Internal Audio File Loading Error: {}", e);
                        continue;
                    }
                };
                samples.push(sound.volume(def.volume));
            }

            if samples.is_empty() {
                warn!("{} had no audio files successfully added so it will be skipped in the database.", def.name);
                continue;
            }
            if def.file_names.len() == 1 {
                let single = samples[0].clone();
                self.sounds.insert(def.name.clone(), SoundFiles::Single(Box::new(single)));
                debug!("inserted {} with sound ", def.name);
                continue;
            }

            self.sounds.insert(def.name.clone(), SoundFiles::Sample(samples.clone()));
        }
    }
}

#[derive(Deserialize)]
struct AudioDefinitions {
    name: String,
    directory: Option<String>,
    file_names: Vec<String>,
    volume: f32,
}
