use bracket_terminal::prelude::ColorPair;
use specs::{Component, VecStorage, Entity};

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub color_pair: ColorPair,
    pub atlas_index: usize,
}

impl Renderable {
    pub fn new(color_pair: ColorPair, atlas_index: usize) -> Self {
        Self {
            color_pair,
            atlas_index,
        }
    }
}

/// Represents a position of anything that exists physically in the game world
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// TODO: This is temporary for testing out breaking things and will be replaced by a more comprehensive stat
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Strength {
    pub amt: u32,
}

/// Prevents gameobjects from passing through it
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Blocking;

/// Defines how breakable an object is, should be used with blocking component to prevent walking through it
/// TODO: seperate health from this component reason: easier to clean up dead entities.
#[derive(Debug, Component)]
#[storage(VecStorage)]
#[allow(dead_code)]
pub struct Breakable {
    pub hp: u32,
    max_hp: u32,
    pub defense: u32,
}

impl Breakable {
    pub fn new(max_hp: u32, defense: u32) -> Self {
        Self {
            hp: max_hp,
            max_hp,
            defense,
        }
    }
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
