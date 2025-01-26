//! this file describes how the audio can be used and gives pub fn that enable this by leveraging
//! what is already loaded
//! data_read/audio.rs describes how the audio data is loaded and setup
use bracket_lib::random::RandomNumberGenerator;
use kira::sound::static_sound::StaticSoundData;
use log::warn;

use crate::data_read::prelude::{AUDIOMAN, AUDIO_DB};

pub enum SoundFiles {
    Single(Box<StaticSoundData>),
    // The order of the sample should be random because that is how it will be accessed.
    Sample(Vec<StaticSoundData>),
}

pub fn play_sound_effect(sfx: &str) {
    let mut audioman = AUDIOMAN.lock().unwrap();
    if audioman.player.is_none() {
        return;
    }

    let adb = AUDIO_DB.lock().unwrap();
    let sfx_file = match adb.sounds.get(sfx) {
        Some(s) => s,
        None => {
            warn!("{} does not exist as a sound file.", sfx);
            return;
        }
    };

    let sfx: StaticSoundData = match sfx_file {
        SoundFiles::Single(single) => *single.clone(),
        SoundFiles::Sample(sample) => {
            let mut rng = RandomNumberGenerator::new();
            let idx = rng.range(0, sample.len());
            sample[idx].clone()
        }
    };

    let _ = audioman.player.as_mut().unwrap().play(sfx);
}
