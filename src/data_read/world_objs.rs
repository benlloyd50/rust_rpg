use std::str::FromStr;

use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};
use weighted_rand::builder::*;

use crate::components::Item;
use crate::droptables::WeightedDrop;
use crate::items::ItemQty;
use crate::{
    components::{
        Blocking, Breakable, DeathDrop, Grass, HealthStats as HealthStatsComponent, Name, Position,
        Renderable,
    },
    z_order::WORLD_OBJECT_Z,
};

use super::{EntityBuildError, ENTITY_DB};

#[derive(Deserialize)]
pub struct WorldObjectDatabase {
    data: Vec<WorldObject>,
}

impl WorldObjectDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    pub fn get_by_name(&self, name: &String) -> Option<&WorldObject> {
        self.data.iter().find(|i| i.name.eq(name))
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: u32) -> Option<&WorldObject> {
        self.data.iter().find(|i| i.identifier.0 == id)
    }
}

/// Attempts to create the specified entity directly into the world
pub fn build_obj(
    name: impl ToString,
    pos: Position,
    world: &mut World,
) -> Result<Entity, EntityBuildError> {
    let edb = &ENTITY_DB.lock().unwrap();
    let raw = match edb.world_objs.get_by_name(&name.to_string()) {
        Some(raw) => raw,
        None => {
            eprintln!("No world object found named: {}", name.to_string());
            return Err(EntityBuildError);
        }
    };
    let mut builder = world.create_entity().with(Name::new(&raw.name)).with(pos);

    if let Some(foreground) = &raw.foreground {
        builder = builder.with(Renderable::default_bg(
            raw.atlas_index,
            *foreground,
            WORLD_OBJECT_Z,
        ));
    }

    if raw.is_blocking {
        builder = builder.with(Blocking);
    }

    if let Some(drop) = raw.get_random_drop() {
        let item_id = match edb.items.get_by_name(&drop) {
            Some(info) => &info.identifier,
            None => {
                eprintln!("No item ID found for {} on world obj {}", drop, &raw.name);
                return Err(EntityBuildError);
            }
        };
        builder = builder.with(DeathDrop(Item::new(*item_id, ItemQty(1))));
    }

    if let Some(breakable) = &raw.breakable {
        match Breakable::from_str(breakable) {
            Ok(breakable_type) => {
                builder = builder.with(breakable_type);
            }
            Err(_) => {
                eprintln!(
                    "Invalid breakable string {} on world object {}",
                    breakable, &raw.name
                );
                return Err(EntityBuildError);
            }
        }
    }

    if raw.grass.is_some() {
        builder = builder.with(Grass);
    }

    if let Some(health_stats) = &raw.health_stats {
        builder = builder.with(HealthStatsComponent::new(
            health_stats.max_hp,
            health_stats.defense,
        ));
    }

    Ok(builder.build())
}

#[derive(Deserialize)]
pub struct WorldObject {
    /// Unique id to find the world object's static data
    identifier: ObjectID,
    name: String,
    atlas_index: u8,
    is_blocking: bool,
    drop_table: Option<Vec<WeightedDrop>>,
    breakable: Option<String>,
    health_stats: Option<HealthStats>,
    grass: Option<String>,
    foreground: Option<(u8, u8, u8)>,
}

impl WorldObject {
    fn get_random_drop(&self) -> Option<String> {
        if self.drop_table.is_none() {
            return None;
        }

        if let Some(drops) = &self.drop_table {
            if drops.is_empty() {
                return None;
            }
            let data: Vec<u32> = drops.iter().map(|d| d.chance).collect();
            let builder = WalkerTableBuilder::new(&data);
            let table = builder.build();
            let idx = table.next();
            return Some(drops[idx].item.clone());
        }
        None
    }
}

#[derive(Deserialize, Serialize)]
struct HealthStats {
    max_hp: usize,
    defense: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectID(pub u32);
