use std::fmt::Display;

/// Contains all configs for various game things
/// Note - For now it resides on the `State` variable since we shouldn't have systems modifying it as that
/// involves checking if the player is the entity in the system.
pub struct ConfigMaster {
    pub inventory: InventoryConfig,
}

impl Default for ConfigMaster {
    fn default() -> Self {
        Self {
            inventory: InventoryConfig {
                sort_mode: SortMode::NameABC,
            },
        }
    }
}

pub struct InventoryConfig {
    pub sort_mode: SortMode,
}

pub enum SortMode {
    NameABC,
    _NameZYX,
    IDAsc,
    _IDDesc,
    _Category,
}
impl Display for SortMode {
    /// Limited to 3 characters for nice formatting
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sort_mode = match self {
            SortMode::NameABC => "ABC",
            SortMode::IDAsc => "ID+",
            _ => "UNK",
        }
        .to_string();
        write!(f, "{}", sort_mode)
    }
}
impl InventoryConfig {
    pub fn rotate_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::NameABC => SortMode::IDAsc,
            SortMode::IDAsc => SortMode::NameABC,
            _ => SortMode::NameABC,
        };
    }
}
