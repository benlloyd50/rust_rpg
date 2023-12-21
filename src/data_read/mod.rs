mod animations;
mod audio;
mod beings;
mod items;
mod levels;
mod recipes;
mod world_objs;

/// A tight bunch of important data reading stuff such as the databases and json loading
/// ```rust
///    use crate::data_read::prelude::*;
/// ```
pub mod prelude {
    pub use crate::data_read::animations::ANIMATION_DB;
    pub use crate::data_read::audio::{AUDIOMAN, AUDIO_DB};
    pub use crate::data_read::beings::build_being;
    pub use crate::data_read::levels::{create_map, LDTK_FILE};
    pub use crate::data_read::recipes::RECIPE_DB;
    pub use crate::data_read::world_objs::build_world_obj;
    pub use crate::data_read::ENTITY_DB;
}

use itertools::Itertools;
use lazy_static::lazy_static;
use log::{debug, error, warn};
use prelude::{ANIMATION_DB, AUDIO_DB};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::{
    droptables::{DropQty, Drops, Loot},
    stats::Stats,
};

use self::{
    beings::{BeingDatabase, RawDrops},
    items::ItemDatabase,
    prelude::RECIPE_DB,
    world_objs::WorldObjectDatabase,
};

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
        Self { items: ItemDatabase::empty(), world_objs: WorldObjectDatabase::empty(), beings: BeingDatabase::empty() }
    }

    fn load(&mut self, data: GameData) {
        *self = data;
    }
}

/// Creates global instances of static data present in the `raws/` folder
pub fn initialize_game_databases() {
    debug!("startup: starting to load game databases");
    let mut game_db = GameData::new();

    // the item database must be loaded first since other tables rely on looking up item names to find their ids
    game_db.items = ItemDatabase::load();

    game_db.world_objs = WorldObjectDatabase::load(&game_db);

    game_db.beings = BeingDatabase::load(&game_db);

    ENTITY_DB.lock().unwrap().load(game_db);
    RECIPE_DB.lock().unwrap().load();
    ANIMATION_DB.lock().unwrap().load();
    AUDIO_DB.lock().unwrap().load();

    debug!("startup: finished loading game databases");
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct OptionalStats {
    pub intelligence: Option<usize>,
    pub strength: Option<usize>,
    pub dexterity: Option<usize>,
    pub vitality: Option<usize>,
    pub precision: Option<usize>,
    pub charisma: Option<usize>,
}

impl Stats {
    fn from_optional(some_stats: &OptionalStats) -> Self {
        Self {
            intelligence: some_stats.intelligence.map_or(0, |stat| stat),
            charisma: some_stats.charisma.map_or(0, |stat| stat),
            dexterity: some_stats.dexterity.map_or(0, |stat| stat),
            strength: some_stats.strength.map_or(0, |stat| stat),
            precision: some_stats.precision.map_or(0, |stat| stat),
            vitality: some_stats.vitality.map_or(0, |stat| stat),
        }
    }
}

impl Drops {
    pub(crate) fn from_raw(raw: &RawDrops, game_db: &GameData) -> Self {
        Self {
            drop_chance: raw.drop_chance,
            loot_table: raw
                .loot_table
                .iter()
                .map(|raw_loot| Loot {
                    id: game_db
                        .items
                        .get_by_name(&raw_loot.item)
                        .expect(&format!("{} has no definition in items", raw_loot.item))
                        .identifier,
                    qty: DropQty::from_str(&raw_loot.item_qty),
                    weight: raw_loot.weight,
                })
                .collect(),
        }
    }
}

impl DropQty {
    fn from_str(qty: &str) -> DropQty {
        if let Some((colon, _)) = qty.chars().find_position(|c| c.eq(&':')) {
            let first: String = qty[..colon].chars().collect();
            let second: String = qty[(colon + 1)..].chars().collect();
            let min = first.parse().unwrap_or(0);
            let max = second.parse().unwrap_or(1);
            if min >= max {
                warn!("Drop range defined by {} is invalid: The range is empty. Using Single(1) instead", qty);
                DropQty::Single(1)
            } else {
                debug!("Creating Drop range from {} to {}", min, max);
                DropQty::Range { min, max }
            }
        } else {
            let single_qty = match qty.parse() {
                Ok(qty) => qty,
                Err(_) => {
                    error!("Invalid qty provided, {} for drop qty falling back to 1", qty);
                    1
                }
            };
            DropQty::Single(single_qty)
        }
    }
}
