use std::{fs, str::FromStr};

use crate::{
    components::{Blocking, Breakable, Grass, HealthStats as HealthStatsComponent, Name, Position, Renderable},
    droptables::Drops,
    map::{ObjectID, WorldObject},
    saveload::SerializeMe,
    z_order::WORLD_OBJECT_Z,
};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use specs::{
    saveload::{MarkedBuilder, SimpleMarker},
    Builder, Entity, World, WorldExt,
};

use super::{beings::RawDrops, EntityBuildError, GameData, ENTITY_DB};

pub struct WorldObjectDatabase {
    data: Vec<WorldObject>,
}

#[derive(Deserialize)]
pub struct RawWorldObject {
    /// Unique id to find the world object's static data
    identifier: usize,
    name: String,
    atlas_index: u8,
    is_blocking: bool,
    breakable: Option<String>,
    health_stats: Option<HealthStats>,
    grass: Option<String>,
    foreground: Option<(u8, u8, u8)>,
    loot: Option<RawDrops>,
}

#[derive(Deserialize, Serialize, Clone)]
struct HealthStats {
    max_hp: usize,
    defense: usize,
}

impl WorldObjectDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    pub(crate) fn load(game_data: &GameData) -> Self {
        let contents: String = fs::read_to_string("raws/world_objs.json")
            .expect("Unable to find world_objs.json at `raws/world_objs.json`");
        let world_objs: Vec<RawWorldObject> = from_str(&contents).expect("Bad JSON in world_objs.json fix it");
        let data = world_objs
            .iter()
            .map(|raw| WorldObject {
                identifier: ObjectID(raw.identifier),
                name: raw.name.clone(),
                atlas_index: raw.atlas_index,
                is_blocking: raw.is_blocking,
                breakable: raw.breakable.clone(),
                health_stats: raw.health_stats.clone().map(|hs| HealthStatsComponent::new(hs.max_hp, hs.defense)),
                grass: raw.grass.clone(),
                foreground: raw.foreground,
                loot: raw.loot.as_ref().map(|raw| Drops::from_raw(raw, game_data)),
            })
            .collect();
        WorldObjectDatabase { data }
    }

    pub fn get_by_name(&self, name: &String) -> Option<&WorldObject> {
        self.data.iter().find(|i| i.name.eq(name))
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: usize) -> Option<&WorldObject> {
        self.data.iter().find(|i| i.identifier.0 == id)
    }
}

/// Attempts to create the specified entity directly into the world
pub fn build_world_obj(name: impl ToString, pos: Position, world: &mut World) -> Result<Entity, EntityBuildError> {
    let edb = &ENTITY_DB.lock().unwrap();
    let raw = match edb.world_objs.get_by_name(&name.to_string()) {
        Some(raw) => raw,
        None => {
            eprintln!("No world object found named: {}", name.to_string());
            return Err(EntityBuildError);
        }
    };
    let mut builder = world.create_entity().with(Name::new(&raw.name)).with(pos).marked::<SimpleMarker<SerializeMe>>();

    if let Some(foreground) = &raw.foreground {
        builder = builder.with(Renderable::clear_bg(raw.atlas_index, *foreground, WORLD_OBJECT_Z));
    }

    if raw.is_blocking {
        builder = builder.with(Blocking {});
    }

    if let Some(breakable) = &raw.breakable {
        match Breakable::from_str(breakable) {
            Ok(breakable_type) => {
                builder = builder.with(breakable_type);
            }
            Err(_) => {
                eprintln!("Invalid breakable string {} on world object {}", breakable, &raw.name);
                return Err(EntityBuildError);
            }
        }
    }

    if raw.grass.is_some() {
        builder = builder.with(Grass {});
    }

    if let Some(health_stats) = &raw.health_stats {
        builder = builder.with(HealthStatsComponent::new(health_stats.max_hp, health_stats.defense));
    }

    Ok(builder.build())
}
