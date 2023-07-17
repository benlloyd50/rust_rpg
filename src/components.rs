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
