use bracket_terminal::prelude::ColorPair;
use specs::{Component, VecStorage};

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
    pub fn new(x: usize, y: usize) -> Self { Self { x, y } }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Blocking;

/// Defines how breakable an object is, should be used with blocking component to prevent walking through it
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Breakable {
    hp: usize,
    max_hp: usize,
    defense: usize,
}

impl Breakable {
    pub fn new(max_hp: usize, defense: usize) -> Self { Self { hp: max_hp, max_hp, defense } }
}

