use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::{
    components::{AttackBonus, Equipable},
    items::{ItemID, ItemInfo},
};

pub struct ItemDatabase {
    data: Vec<ItemInfo>,
}

#[derive(Deserialize)]
pub struct RawItemDatabase {
    data: Vec<RawItemInfo>,
}

impl ItemDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    pub fn load() -> Self {
        let contents: String = fs::read_to_string("raws/items.json")
            .expect("Unable to find items.json at `raws/items.json`");
        let raw_info_db: RawItemDatabase =
            from_str(&contents).expect("Bad JSON in items.json fix it");
        ItemDatabase {
            data: raw_info_db
                .data
                .iter()
                .map(|info| ItemInfo::from(info))
                .collect(),
        }
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
pub struct RawItemInfo {
    /// Unique id to find the item's static data
    pub identifier: ItemID,
    pub name: String,
    pub examine_text: String,
    pub atlas_index: u8,
    pub fg: (u8, u8, u8),
    pub pickup_text: Option<String>,
    pub equipable: Option<String>,
    pub attack_bonus: Option<usize>,
}

impl ItemInfo {
    fn from(value: &RawItemInfo) -> Self {
        Self {
            identifier: value.identifier,
            name: value.name.clone(),
            examine_text: value.examine_text.clone(),
            atlas_index: value.atlas_index,
            fg: value.fg,
            pickup_text: value.pickup_text.clone(),
            equipable: value
                .equipable
                .clone()
                .map_or(None, |e| Some(Equipable::from_str(&e))),
            attack_bonus: value
                .attack_bonus
                .map_or(None, |bonus| Some(AttackBonus(bonus as i32))),
        }
    }
}
