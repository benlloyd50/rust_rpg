use std::fmt::Display;

use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};

use crate::{
    components::{Equipable, EquipmentSlot, Item, Name, Position, Renderable},
    data_read::EntityBuildError,
    items::ItemQty,
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

    pub fn get_by_name(&self, name: &str) -> Option<&ItemInfo> {
        self.data.iter().find(|i| i.name.eq(name))
    }

    /// Gets the entity by name without ensuring it exists.
    /// This could by panic but can be used when certain a name would exist for an item.
    pub fn get_by_name_unchecked(&self, name: &String) -> &ItemInfo {
        self.data.iter().find(|i| i.name.eq(name)).unwrap()
    }

    pub fn get_by_id(&self, id: ItemID) -> Option<&ItemInfo> {
        self.data.iter().find(|i| i.identifier == id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemInfo {
    /// Unique id to find the item's static data
    pub identifier: ItemID,
    pub name: String,
    pub examine_text: String,
    pub atlas_index: u8,
    pub fg: (u8, u8, u8),
    pub pickup_text: Option<String>,
    pub equipable: Option<String>,
}

#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq, Default, PartialOrd, Ord,
)]
pub struct ItemID(pub u32);

impl Display for ItemID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let edb = &ENTITY_DB.lock().unwrap();
        let missing = format!("Missing name {}", self.0);
        let name = match edb.items.get_by_id(*self) {
            Some(info) => &info.name,
            None => &missing,
        };
        write!(f, "id: {} | name: {}", self.0, name)
    }
}

// TODO: duplicated in items.rs, this one could probably be removed in favor the resource, ItemSpawner
pub fn build_item(
    name: &str,
    pos: Option<Position>,
    world: &mut World,
) -> Result<Entity, EntityBuildError> {
    let edb = &ENTITY_DB.lock().unwrap();
    let raw = match edb.items.get_by_name(name) {
        Some(raw) => raw,
        None => {
            eprintln!("No world object found named: {}", name.to_string());
            return Err(EntityBuildError);
        }
    };
    let mut builder = world
        .create_entity()
        .with(Item::new(raw.identifier, ItemQty(1)))
        .with(Name::new(&raw.name))
        .with(Renderable::default_bg(raw.atlas_index, raw.fg, ITEM_Z));

    if let Some(pos) = pos {
        builder = builder.with(pos);
    }

    if let Some(equipable) = &raw.equipable {
        builder = match equipable.as_str() {
            "Hand" => builder.with(Equipable {
                slot: EquipmentSlot::Hand,
            }),
            "Torso" => builder.with(Equipable {
                slot: EquipmentSlot::Torso,
            }),
            "Head" => builder.with(Equipable {
                slot: EquipmentSlot::Head,
            }),
            "Legs" => builder.with(Equipable {
                slot: EquipmentSlot::Legs,
            }),
            "Feet" => builder.with(Equipable {
                slot: EquipmentSlot::Feet,
            }),
            "Tail" => builder.with(Equipable {
                slot: EquipmentSlot::Tail,
            }),
            _ => {
                eprintln!("{} is not a valid name for an equipment slot", equipable);
                builder
            }
        };
    }

    Ok(builder.build())
}
