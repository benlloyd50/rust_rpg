mod items;
mod levels;

/// A tight bunch of important data reading stuff such as the databases and json loading
/// ```rust
///    use crate::data_read::prelude::*;
/// ```
pub mod prelude {
    pub use crate::data_read::items::ItemID;
    pub use crate::data_read::levels::load_simple_ldtk_level;
    pub use crate::data_read::ENTITY_DB;
}

use lazy_static::lazy_static;
use serde_json::from_str;
use std::{fs, sync::Mutex};

use self::items::ItemDatabase;

lazy_static! {
    pub static ref ENTITY_DB: Mutex<GameData> = Mutex::new(GameData::new());
}

pub struct GameData {
    pub items: ItemDatabase,
}

impl GameData {
    fn new() -> Self {
        Self {
            items: ItemDatabase::empty(),
        }
    }

    fn load(&mut self, data: GameData) {
        *self = data;
    }
}

/// Creates global instances of static data present in the `raws/` folder
pub fn initialize_game_databases() {
    let mut game_db = GameData::new();

    let contents: String = fs::read_to_string("raws/items.json")
        .expect("Unable to find items.json at `raws/items.json`");
    let items: ItemDatabase = from_str(&contents).expect("Bad JSON in items.json fix it");
    game_db.items = items;

    ENTITY_DB.lock().unwrap().load(game_db);
}
