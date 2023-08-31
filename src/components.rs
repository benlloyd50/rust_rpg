use std::{fmt::Display, str::FromStr, time::Duration, collections::{HashMap, hash_map::Entry}};

use bracket_terminal::prelude::{ColorPair, Point, WHITE};
use specs::{Component, Entity, NullStorage, VecStorage};

use crate::{data_read::{prelude::ItemID, ENTITY_DB}, indexing::idx_to_xy, items::ItemQty};

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub color_pair: ColorPair,
    pub atlas_index: usize,
    pub z_priority: u32,
}

impl Renderable {
    pub fn new(fg: (u8, u8, u8), bg: (u8, u8, u8), atlas_index: usize, z_priority: u32) -> Self {
        Self {
            color_pair: ColorPair::new(fg, bg),
            atlas_index,
            z_priority,
        }
    }

    /// Creates a renderable with a black bg and specified parts
    pub fn default_bg(atlas_index: usize, fg: (u8, u8, u8), z_priority: u32) -> Self {
        Self {
            color_pair: ColorPair::new(fg, WHITE),
            atlas_index,
            z_priority,
        }
    }
}

/// Represents a position of anything that exists physically in the game world
#[derive(Debug, Component, Copy, Clone)]
#[storage(VecStorage)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn from_idx(idx: usize, width: usize) -> Self {
        idx_to_xy(idx, width).into()
    }

    #[allow(dead_code)]
    pub fn to_idx(&self, width: usize) -> usize {
        self.y * width + self.x
    }
}

impl From<Point> for Position {
    /// May panic if either of the coords of `value` are negative but that should rarely be the case when used in the
    /// proper context. i.e. dont use this when dealing with delta point values (-1, -1)
    fn from(value: Point) -> Self {
        Self::new(value.x as usize, value.y as usize)
    }
}

/// TODO: This is temporary for testing out breaking things and will be replaced by a more comprehensive stat
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Strength {
    pub amt: usize,
}

/// Prevents gameobjects from passing through it
#[derive(Debug, Component, Default)]
#[storage(NullStorage)]
pub struct Blocking;

#[derive(Debug, Component, Default)]
#[storage(VecStorage)]
pub struct Fishable {
    pub time_left: Duration,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct FishAction {
    pub target: Position, // mainly just for finding where the fishing rod will be spawned
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct WaitingForFish {
    pub attempts: usize,
    pub time_since_last_attempt: Duration,
}

impl WaitingForFish {
    pub fn new(attempts: usize) -> Self {
        Self {
            attempts,
            time_since_last_attempt: Duration::new(0, 0),
        }
    }
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct FishOnTheLine;

#[derive(Component, Clone, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Name(pub String);

const MISSING_ITEM_NAME: &'static str = "MISSING_ITEM_NAME";
const MISSING_BEING_NAME: &'static str = "MISSING_BEING_NAME";

impl Name {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }

    pub fn missing_item_name() -> Self {
        Name::new(MISSING_ITEM_NAME)
    }

    pub fn missing_being_name() -> Self {
        Name::new(MISSING_BEING_NAME)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Monster;

/// Makes the entity walk around in a random cardinal direction
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct RandomWalkerAI;

#[derive(Debug, Component)]
#[storage(VecStorage)]
#[allow(dead_code)]
pub struct HealthStats {
    pub hp: usize,
    max_hp: usize,
    pub defense: usize,
}

/// An item that will be spawned on the associated entity's death
#[derive(Component)]
#[storage(VecStorage)]
pub struct DeathDrop {
    pub item_id: ItemID,
}

impl DeathDrop {
    pub fn new(item_id: &ItemID) -> Self {
        Self { item_id: *item_id }
    }
}

impl HealthStats {
    pub fn new(max_hp: usize, defense: usize) -> Self {
        Self {
            hp: max_hp,
            max_hp,
            defense,
        }
    }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Breakable {
    pub by: ToolType,
}

impl Breakable {
    pub fn new(by: ToolType) -> Self {
        Self { by }
    }
}

impl FromStr for Breakable {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Hand" => Ok(Breakable::new(ToolType::Hand)),
            "Axe" => Ok(Breakable::new(ToolType::Axe)),
            "Pickaxe" => Ok(Breakable::new(ToolType::Pickaxe)),
            "Shovel" => Ok(Breakable::new(ToolType::Shovel)),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ToolType {
    Hand,
    Pickaxe,
    Axe,
    Shovel,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct BreakAction {
    pub target: Entity,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

/// Used to delete an entity when a condition is satisfied
#[derive(Component, Clone, Copy)]
#[storage(VecStorage)]
pub enum DeleteCondition {
    _Timed(Duration), // Condition is based on deleting after a specificed amount of time
    ActivityFinish(Entity), // Condition is based on when the entity finishes their activity
}

/// Used to signal to other systems that an entity finished their activity
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct FinishedActivity;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Item;


#[derive(Component)]
#[storage(VecStorage)]
pub struct Backpack {
    contents: HashMap<ItemID, ItemQty>,
}

impl Backpack {
    pub fn empty() -> Self {
        Self {
            contents: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }

    pub fn add_into_backpack(&mut self, item_id: ItemID, qty: usize) -> bool {
        match self.contents.entry(item_id) {
            Entry::Occupied(mut o) => { o.get_mut().add(qty);}
            Entry::Vacant(v) => { v.insert(ItemQty::new(qty)); }
        }
        true
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, ItemID, ItemQty> {
        self.contents.iter()
    }


    /// Checks inventory for an item based on name.
    /// This is useful when edb is not needed for other reasons in the calling function.
    /// If you do need edb for other information then use `.contains(ItemID)`
    pub fn contains_named(&self, name: &Name) -> bool {
        let edb = &ENTITY_DB.lock().unwrap();
        let info = edb.items.get_by_name_unchecked(&name.0);
        self.contains(info.identifier)
    }

    /// Checks inventory for an item based on ID.
    pub fn contains(&self, item_id: ItemID) -> bool {
        match self.contents.get(&item_id) {
            Some(o) => {
                o.0 > 0
            }
            None => {
                false
            }
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct PickupAction {
    pub item: Entity,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Water;
