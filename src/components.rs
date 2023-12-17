pub(crate) use crate::{
    debug::CLEAR,
    fishing::{Cursor, GoalBar},
    items::ItemQty,
    map::xy_to_idx_given_width,
    stats::Stats,
};
use std::{fmt::Display, str::FromStr, time::Duration};

use bracket_terminal::prelude::{ColorPair, Degrees, Point, PointF};
use serde::{Deserialize, Serialize};
#[allow(deprecated)] // deprecated but specs uses it so it's not my choice, maybe this gets changed in new version
use specs::error::NoError;
use specs::{
    saveload::{ConvertSaveload, Marker},
    Component, ConvertSaveload, Entity, NullStorage, VecStorage,
};

use crate::{fishing::ReelBar, indexing::idx_to_point, inventory::UseMenuResult, items::ItemID};

#[derive(Debug, Component, ConvertSaveload, Clone)]
#[storage(VecStorage)]
pub struct Renderable {
    pub color_pair: ColorPair,
    pub atlas_index: u8,
    pub z_priority: u32,
}

/// Marking an entity with this means that it will continue to exist when a level switch changes.
/// This ensures certain things continue as normal since not all entities should be destroyed.
#[derive(Component, Serialize, Deserialize, Clone)]
#[storage(NullStorage)]
pub struct LevelPersistent {}

impl Renderable {
    /// Creates a renderable with a clear bg and specified parts
    pub fn clear_bg(atlas_index: u8, fg: (u8, u8, u8), z_priority: u32) -> Self {
        Self { color_pair: ColorPair::new(fg, CLEAR), atlas_index, z_priority }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub sprite_pos: PointF,
    pub rotation: Degrees,
    pub scale: PointF,
}

impl Transform {
    pub fn new(x: f32, y: f32, degrees: f32, scale_x: f32, scale_y: f32) -> Self {
        Self { sprite_pos: PointF::new(x, y), rotation: Degrees(degrees), scale: PointF::new(scale_x, scale_y) }
    }
}

/// Represents a position of anything that exists physically in the game world
#[derive(Debug, Component, Copy, Clone, PartialEq, Eq, Hash, ConvertSaveload, Serialize)]
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
        idx_to_point(idx, width).into()
    }

    pub fn to_idx(self, width: usize) -> usize {
        xy_to_idx_given_width(self.x, self.y, width)
    }

    pub fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl From<Point> for Position {
    /// May panic if either of the coords of `value` are negative but that should rarely be the case when used in the
    /// proper context. i.e. dont use this when dealing with delta point values (-1, -1)
    fn from(value: Point) -> Self {
        Self::new(value.x as usize, value.y as usize)
    }
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "X:{} Y:{}", self.x, self.y)
    }
}

#[derive(Debug, Component, Copy, Clone, ConvertSaveload)]
#[storage(VecStorage)]
pub struct EntityStats {
    pub set: Stats,
}

pub struct StatsError;

impl EntityStats {
    #[allow(dead_code)]
    pub fn init(
        stat_limit: usize,
        int: usize,
        str: usize,
        dex: usize,
        vit: usize,
        pre: usize,
        cha: usize,
    ) -> Result<Self, StatsError> {
        let stats =
            Stats { intelligence: int, strength: str, dexterity: dex, vitality: vit, precision: pre, charisma: cha };
        if stats.get_total() < stat_limit {
            Ok(Self { set: stats })
        } else {
            Err(StatsError)
        }
    }
}

impl From<Stats> for EntityStats {
    fn from(stats: Stats) -> Self {
        Self { set: stats }
    }
}

/// Prevents gameobjects from passing through it
#[derive(Debug, Component, Default, Clone, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Blocking {}

#[derive(Debug, Component, Default, ConvertSaveload)]
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
        Self { attempts, time_since_last_attempt: Duration::new(0, 0) }
    }
}

#[derive(Component, Default)]
#[storage(NullStorage)]
// TODO: put type of fish on this, ItemID will probably do
pub struct FishOnTheLine {}

#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, ConvertSaveload)]
#[storage(VecStorage)]
pub struct Name(pub String);

const MISSING_ITEM_NAME: &str = "MISSING_ITEM_NAME";

impl Name {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }

    pub fn missing_item_name() -> Self {
        Name::new(MISSING_ITEM_NAME)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Makes the entity walk around in a random cardinal direction
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct RandomWalkerAI {}

/// Makes the entity walk towards a goal which is targeted
#[derive(Component, Clone)]
#[storage(VecStorage)]
pub struct GoalMoverAI {
    pub current: Option<Entity>,
    pub desires: Vec<Name>,
    pub goal_range: usize,
}

impl GoalMoverAI {
    pub fn with_desires(desires: &[Name], goal_range: usize) -> Self {
        Self { current: None, desires: desires.to_vec(), goal_range }
    }
}

#[derive(Debug, Component, Clone, ConvertSaveload)]
#[storage(VecStorage)]
pub struct HealthStats {
    pub hp: usize,
    pub max_hp: usize,
    pub defense: usize,
}

impl HealthStats {
    pub fn new(max_hp: usize, defense: usize) -> Self {
        Self { hp: max_hp, max_hp, defense }
    }

    pub fn add_health(&mut self, amt: usize) {
        self.hp = usize::min(self.hp + amt, self.max_hp);
    }
}

#[derive(Debug, Component, Clone, ConvertSaveload)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct AttackAction {
    pub target: Entity,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct MoveAction {
    pub new_pos: Position,
}

impl MoveAction {
    pub fn new(pos: Position) -> Self {
        Self { new_pos: pos }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct CraftAction {
    pub first_item: Entity,
    pub second_item: Entity,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct EquipAction {
    pub item: Entity,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct HealAction {
    pub amount: usize,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

/// Used to delete an entity when a condition is satisfied
#[derive(Component, Clone, Copy, ConvertSaveload)]
#[storage(VecStorage)]
pub enum DeleteCondition {
    _Timed(Duration),       // Condition is based on deleting after a specificed amount of time
    ActivityFinish(Entity), // Condition is based on when the entity finishes their activity
}

/// Used to signal to other systems that an entity finished their activity
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct FinishedActivity {}

#[derive(Component, Default, ConvertSaveload, Clone)]
#[storage(VecStorage)]
pub struct Item {
    pub id: ItemID,
    pub qty: ItemQty,
}

impl Item {
    pub fn new(id: ItemID, qty: ItemQty) -> Self {
        Self { id, qty }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
#[storage(VecStorage)]
pub struct InBag {
    pub owner: Entity,
}

#[derive(Component, Clone, ConvertSaveload)]
#[storage(VecStorage)]
pub enum Consumable {
    InstantRegen(usize),
    // Effect
}

impl Consumable {
    pub fn from_str(str: &str, amt: usize) -> Self {
        match str {
            "instant_regen" => Self::InstantRegen(amt),
            _ => panic!("Bad string based to consumable."),
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct ConsumeAction {
    pub consuming: Entity,
}

impl ConsumeAction {
    pub fn new(target: &Entity) -> Self {
        Self { consuming: *target }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
#[storage(VecStorage)]
pub struct Equipped {
    pub on: Entity,
}

#[derive(Component, Clone, ConvertSaveload)]
#[storage(VecStorage)]
pub struct Equipable {
    pub slot: EquipmentSlot,
}

impl Equipable {
    pub fn from_str(str: &str) -> Self {
        let slot = match str {
            "Hand" => EquipmentSlot::Hand,
            "Torso" => EquipmentSlot::Torso,
            "Head" => EquipmentSlot::Head,
            "Legs" => EquipmentSlot::Legs,
            "Feet" => EquipmentSlot::Feet,
            "Tail" => EquipmentSlot::Tail,
            _ => {
                eprintln!("{} is not a valid name for an equipment slot, using Head instead", str);
                EquipmentSlot::Head
            }
        };
        Equipable { slot }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Hand,
    Torso,
    Head,
    Legs,
    Feet,
    Tail,
}

#[derive(Component, ConvertSaveload, Clone)]
#[storage(VecStorage)]
pub struct EquipmentSlots {
    pub slots: Vec<EquipmentSlot>,
}

impl EquipmentSlots {
    /// Creates slots based on an average human's bodyparts
    pub fn human() -> Self {
        Self {
            slots: vec![
                EquipmentSlot::Hand,
                EquipmentSlot::Hand,
                EquipmentSlot::Legs,
                EquipmentSlot::Torso,
                EquipmentSlot::Head,
                EquipmentSlot::Feet,
            ],
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct PickupAction {
    pub item: Entity,
}

/// Water ripe for swimming in or boating over or building a pier to fish off
#[derive(Component, Default, Serialize, Deserialize, Clone)]
#[storage(NullStorage)]
pub struct Water {}

/// A delicious treat loved by many animals and other beings...
#[derive(Component, Default, Serialize, Deserialize, Clone)]
#[storage(NullStorage)]
pub struct Grass {}

#[derive(Component, ConvertSaveload, Clone)]
#[storage(VecStorage)]
pub struct Interactor {
    pub mode: InteractorMode,
}

impl Interactor {
    pub fn new(mode: InteractorMode) -> Self {
        Self { mode }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum InteractorMode {
    Reactive,
    Agressive,
}

impl Display for InteractorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = match self {
            Self::Reactive => "Reactive",
            Self::Agressive => "Agressive",
        };
        write!(f, "{}", self_str)
    }
}

#[derive(Component, Clone)]
#[storage(VecStorage)]
pub struct SelectedInventoryItem {
    pub first_item: Entity,
    pub intended_action: Option<UseMenuResult>,
}

#[derive(Component, Clone, ConvertSaveload)]
#[storage(VecStorage)]
pub struct AttackBonus(pub i32);

#[derive(Component)]
#[storage(NullStorage)]
pub struct GameAction {}

#[derive(Component)]
#[storage(VecStorage)]
pub struct FishingMinigame {
    pub cursor: Cursor,
    pub goal_bar: GoalBar,
    pub attempts_left: usize,
    pub reel: ReelBar,
}
