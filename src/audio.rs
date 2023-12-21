use log::warn;

use crate::data_read::prelude::{AUDIOMAN, AUDIO_DB};

/// this file describes how the audio can be used and gives pub fn that enable this by leveraging
/// what is already loaded
/// data_read/audio.rs describes how the audio data is loaded and setup

pub fn play_sound_effect(sfx: &str) {
    let mut audioman = AUDIOMAN.lock().unwrap();
    if audioman.player.is_none() {
        return;
    }

    let sfx_file = match AUDIO_DB.lock().unwrap().sounds.get(&format!("{}.wav", sfx)) {
        Some(s) => s.clone(),
        None => {
            warn!("{} does not exist as a sound file.", sfx);
            return;
        }
    };

    let _ = audioman.player.as_mut().unwrap().play(sfx_file);
}
