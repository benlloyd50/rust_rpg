use std::fmt::Display;

use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};

use crate::{
    components::{Item, Name, Position, Renderable},
    data_read::EntityBuildError,
    z_order::ITEM_Z,
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

    /// Gets the entity by name without ensuring it exists.
    /// This could by panic but can be used when certain a name would exist for an item.
    pub fn get_by_name_unchecked(&self, name: &String) -> &ItemInfo {
        self.data.iter().find(|i| i.name.eq(name)).unwrap()
    }

    pub fn get_by_id(&self, id: u32) -> Option<&ItemInfo> {
        self.data.iter().find(|i| i.identifier.0 == id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemInfo {
    /// Unique id to find the item's static data
    pub identifier: ItemID,
    pub name: String,
    pub atlas_index: usize,
    pub fg: (u8, u8, u8),
    pub pickup_text: Option<String>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ItemID(pub u32);

impl Display for ItemID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let edb = &ENTITY_DB.lock().unwrap();
        let temp = format!("Missing name {}", self.0);
        let name = match edb.items.get_by_id(self.0) {
            Some(info) => &info.name,
            None => &temp,
        };
        write!(f, "{}", name)
    }
}

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
        .with(Renderable::default_bg(raw.atlas_index, raw.fg, ITEM_Z));

    Ok(builder.build())
}
