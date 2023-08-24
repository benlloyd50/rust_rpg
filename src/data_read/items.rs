use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ItemDatabase {
    data: Vec<ItemInfo>,
}

impl ItemDatabase {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    pub fn get(&self, id: u32) -> Option<&ItemInfo> {
        self.data.iter().find(|i| i.identifier.0 == id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemInfo {
    /// Unique id to find the item's static data
    identifier: ItemID,
    pub name: String,
    pub atlas_index: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ItemID(pub u32);
