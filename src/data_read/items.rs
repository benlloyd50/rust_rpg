use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};

use crate::{
    components::{Item, Name, Position, Renderable},
    data_read::EntityBuildError,
};

use super::ENTITY_DB;

#[derive(Deserialize)]
pub struct ItemDatabase {
    data: Vec<ItemInfo>,
}

impl ItemDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    pub fn get_by_name(&self, name: &String) -> Option<&ItemInfo> {
        self.data.iter().find(|i| i.name.eq(name))
    }

    pub fn get_by_id(&self, id: u32) -> Option<&ItemInfo> {
        self.data.iter().find(|i| i.identifier.0 == id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemInfo {
    /// Unique id to find the item's static data
    pub(crate) identifier: ItemID,
    pub(crate) name: String,
    pub(crate) atlas_index: usize,
    pub(crate) fg: (u8, u8, u8),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ItemID(pub u32);

pub fn build_item(
    name: impl ToString,
    pos: Position,
    world: &mut World,
) -> Result<Entity, EntityBuildError> {
    let edb = &ENTITY_DB.lock().unwrap();
    let raw = match edb.items.get_by_name(&name.to_string()) {
        Some(raw) => raw,
        None => {
            eprintln!("No world object found named: {}", name.to_string());
            return Err(EntityBuildError);
        }
    };
    let builder = world
        .create_entity()
        .with(Item)
        .with(Name::new(&raw.name))
        .with(pos)
        .with(Renderable::default_bg(raw.atlas_index, raw.fg));

    Ok(builder.build())
}
