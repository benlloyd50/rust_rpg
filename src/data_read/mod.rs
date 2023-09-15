mod beings;
mod items;
mod levels;
mod world_objs;

/// A tight bunch of important data reading stuff such as the databases and json loading
/// ```rust
///    use crate::data_read::prelude::*;
/// ```
pub mod prelude {
    pub use crate::data_read::beings::build_being;
    pub use crate::data_read::items::{ItemID, build_item};
    pub use crate::data_read::levels::load_simple_ldtk_level;
    pub use crate::data_read::world_objs::build_obj;
    pub use crate::data_read::ENTITY_DB;
}

use lazy_static::lazy_static;
use serde_json::from_str;
use std::{fs, sync::Mutex};

use self::{beings::BeingDatabase, items::ItemDatabase, world_objs::WorldObjectDatabase};

lazy_static! {
    pub static ref ENTITY_DB: Mutex<GameData> = Mutex::new(GameData::new());
}

#[derive(Debug)]
pub struct EntityBuildError;

pub struct GameData {
    pub items: ItemDatabase,
    pub world_objs: WorldObjectDatabase,
    pub beings: BeingDatabase,
}

impl GameData {
    fn new() -> Self {
        Self {
            items: ItemDatabase::empty(),
            world_objs: WorldObjectDatabase::empty(),
            beings: BeingDatabase::empty(),
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

    let contents: String = fs::read_to_string("raws/world_objs.json")
        .expect("Unable to find items.json at `raws/world_objs.json`");
    let world_objs: WorldObjectDatabase =
        from_str(&contents).expect("Bad JSON in items.json fix it");
    game_db.world_objs = world_objs;

    let contents: String = fs::read_to_string("raws/beings.json")
        .expect("Unable to find items.json at `raws/beings.json`");
    let beings: BeingDatabase = from_str(&contents).expect("Bad JSON in beings.json fix it");
    game_db.beings = beings;

    ENTITY_DB.lock().unwrap().load(game_db);
}
