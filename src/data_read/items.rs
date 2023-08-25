use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ItemID(pub u32);
