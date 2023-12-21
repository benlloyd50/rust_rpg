use std::{collections::HashMap, fs, sync::Mutex};

use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};

lazy_static! {
    pub static ref AUDIOMAN: Mutex<AudioPlayer> = Mutex::new(AudioPlayer::new());
    pub static ref AUDIO_DB: Mutex<AudioDatabase> = Mutex::new(AudioDatabase::new());
}

pub const AUDIO_DIRECTORY: &str = "./resources/sounds";

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
    pub sounds: HashMap<String, StaticSoundData>,
}

impl AudioDatabase {
    fn new() -> Self {
        Self { sounds: HashMap::new() }
    }

    pub fn load(&mut self) {
        let paths = match fs::read_dir(AUDIO_DIRECTORY) {
            Ok(p) => p,
            Err(e) => {
                error!("Error trying to read in audio files from {} NO audio files will be loaded.", AUDIO_DIRECTORY);
                error!("Internal Read Error: {}", e);
                return;
            }
        };
        for path in paths.filter_map(|de| de.ok()) {
            if path.file_type().is_ok_and(|ft| ft.is_file()) {
                let sound = match StaticSoundData::from_file(path.path(), StaticSoundSettings::default()) {
                    Ok(ssd) => ssd,
                    Err(e) => {
                        warn!("Error trying to read in {:?}. Skipping file.", path);
                        error!("Internal Audio File Loading Error: {}", e);
                        continue;
                    }
                };
                match path.file_name().to_os_string().into_string() {
                    Ok(file_name) => {
                        debug!("file name inserted as {}", file_name);
                        self.sounds.insert(file_name, sound);
                    }
                    Err(e) => {
                        warn!("Improper file name for {:#?}, Skipping file.", e);
                    }
                }
            }
        }
    }
}
